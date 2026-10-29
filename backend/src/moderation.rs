//! Moderation through computed trust levels and quorum.
//!
//! Owner (2026-10-29): "all moderation through discourse-style trust levels or quorum".
//!
//! What this module refuses to have: a moderator role that can hide things quietly.
//! The only stored privilege is `profiles.is_leader` (trust level 4), every use of
//! it writes a `moderation_actions` row, and everything below it happens by
//! arithmetic on other people's independent flags.
//!
//! Two rules are worth stating where they are enforced:
//!
//!   * a flag count is not a verdict — 3 flags with at least one from trust level 2
//!     hide the target, and the target can come back on 2 restore votes;
//!   * hiding is about VISIBLE CONTENT, not about rewriting history: a hidden log
//!     still counts toward its author's own score. Deleting or zeroing it would let
//!     a flag campaign edit someone's record.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::Session;

use crate::api::require_user;
use crate::error::{ApiError, ApiResult};
use crate::AppState;

/// Distinct flags that hide a target.
pub const FLAG_QUORUM: i64 = 3;
/// Distinct restore votes that bring one back.
pub const RESTORE_QUORUM: i64 = 2;
/// Level that may flag.
pub const FLAG_LEVEL: i32 = 1;
/// Level that may see the queue and vote to restore.
pub const MODERATOR_LEVEL: i32 = 2;
/// Level that may act alone.
pub const LEADER_LEVEL: i32 = 4;

fn rfc3339(ts: OffsetDateTime) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn valid_kind(kind: &str) -> bool {
    kind == "log" || kind == "cage_check"
}

/// One rung as `/api/trust` publishes it. The page prints these numbers directly, so a
/// requirement can never be hardcoded in the UI and drift from the function that decides.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TrustRung {
    pub level: i32,
    pub min_age_hours: i32,
    pub min_active_days: i32,
    pub min_contributions: i32,
}

#[derive(Debug, Serialize)]
pub struct TrustResponse {
    pub level: i32,
    pub active_days: i32,
    pub contributions: i32,
    pub upheld_flags: i32,
    /// Contributions still needed for the next automatic level; null at 3+ because
    /// level 4 is granted, not earned.
    pub next_level_at: Option<i32>,
    pub is_leader: bool,
    /// `small` (fewer than 10 active members) or `grown`. The ladder bends on this word,
    /// and the page says which band it is standing on.
    pub band: String,
    pub active_accounts: i32,
    /// Every rung of the current band, lowest first.
    pub thresholds: Vec<TrustRung>,
}

#[derive(Debug, Deserialize)]
pub struct FlagRequest {
    pub target_kind: String,
    pub target_id: i64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub flags: i64,
    pub from_tl2: i64,
    pub hidden: bool,
}

#[derive(Debug, Serialize)]
pub struct QueueItem {
    pub target_kind: String,
    pub target_id: i64,
    pub flags: i64,
    pub from_tl2: i64,
    pub hidden: bool,
    pub preview: Option<String>,
    pub author: Option<String>,
    pub first_flag_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub target_kind: String,
    pub target_id: i64,
    /// "hide" or "restore"
    pub action: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub hidden: bool,
    /// Restore votes recorded for this target, including the caller's.
    pub restore_votes: i64,
}

async fn level_of(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> ApiResult<i32> {
    let level: i32 = sqlx::query_scalar("select level from public.trust_level($1)")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(level)
}

/// GET /api/trust — my level, its inputs, and what is left to the next one.
pub async fn get_trust(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<TrustResponse>> {
    let user = require_user(&session).await?;
    let (level, active_days, contributions, upheld_flags, next_level_at): (
        i32,
        i32,
        i32,
        i32,
        Option<i32>,
    ) = sqlx::query_as(
        "select level, active_days, contributions, upheld_flags, next_level_at
         from public.trust_level($1)",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let is_leader: bool = sqlx::query_scalar("select is_leader from public.profiles where id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    // The rungs come from the same function `trust_level()` reads, so the page cannot
    // print a requirement the level is not actually judged against.
    let thresholds: Vec<TrustRung> = sqlx::query_as(
        "select level, min_age_hours, min_active_days, min_contributions
           from public.trust_thresholds() order by level",
    )
    .fetch_all(&state.pool)
    .await?;
    let (band, active_accounts): (String, i32) = sqlx::query_as(
        "select t.band, public.active_accounts() as n from public.trust_thresholds() t limit 1",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(TrustResponse {
        level,
        active_days,
        contributions,
        upheld_flags,
        next_level_at,
        is_leader,
        band,
        active_accounts,
        thresholds,
    }))
}

/// POST /api/flags — flag a log or a cage check. Idempotent per reporter: a second
/// flag from the same person is a conflict, not a second voice.
pub async fn flag(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<FlagRequest>,
) -> ApiResult<Json<FlagResponse>> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;

    if !valid_kind(&body.target_kind) {
        return Err(ApiError::bad_request("target_kind must be 'log' or 'cage_check'"));
    }
    let reason = body.reason.trim();
    if reason.is_empty() || reason.chars().count() > 140 {
        return Err(ApiError::bad_request("Reason must be 1–140 characters"));
    }
    let level = level_of(&state.pool, user.id).await?;
    if level < FLAG_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Flagging needs trust level 1 — a day old and three logs",
        ));
    }

    // The target must exist; a flag on a deleted row is a 404, not an orphan.
    let exists: bool = if body.target_kind == "log" {
        sqlx::query_scalar::<_, bool>("select exists(select 1 from public.habit_logs where id = $1)")
            .bind(body.target_id)
            .fetch_one(&state.pool)
            .await?
    } else {
        sqlx::query_scalar::<_, bool>("select exists(select 1 from public.cage_checks where id = $1)")
            .bind(body.target_id)
            .fetch_one(&state.pool)
            .await?
    };
    if !exists {
        return Err(ApiError::not_found("No such target"));
    }

    let inserted = sqlx::query(
        "insert into public.flags (target_kind, target_id, reporter_id, reason)
         values ($1, $2, $3, $4)
         on conflict (reporter_id, target_kind, target_id) do nothing",
    )
    .bind(&body.target_kind)
    .bind(body.target_id)
    .bind(user.id)
    .bind(reason)
    .execute(&state.pool)
    .await?
    .rows_affected();
    if inserted == 0 {
        return Err(ApiError::conflict("You already flagged this"));
    }

    let (flags, from_tl2): (i64, i64) =
        sqlx::query_as("select flags::bigint, from_tl2::bigint from public.flag_quorum($1, $2)")
            .bind(&body.target_kind)
            .bind(body.target_id)
            .fetch_one(&state.pool)
            .await?;

    let mut hidden = false;
    if flags >= FLAG_QUORUM && from_tl2 >= 1 {
        sqlx::query("select public.hide_target($1, $2)")
            .bind(&body.target_kind)
            .bind(body.target_id)
            .execute(&state.pool)
            .await?;
        sqlx::query(
            "insert into public.moderation_actions (actor_id, action, target_kind, target_id, note)
             values (null, 'auto_hide', $1, $2, $3)",
        )
        .bind(&body.target_kind)
        .bind(body.target_id)
        .bind(format!("{flags} flags ({from_tl2} from trust level 2+)"))
        .execute(&state.pool)
        .await?;
        hidden = true;
    }

    Ok(Json(FlagResponse {
        flags,
        from_tl2,
        hidden,
    }))
}

/// GET /api/moderation/queue — trust level 2+ only: what has been flagged, with
/// the counts that decide it.
pub async fn queue(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<Vec<QueueItem>>> {
    let user = require_user(&session).await?;
    let level = level_of(&state.pool, user.id).await?;
    if level < MODERATOR_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "The moderation queue is for trust level 2",
        ));
    }

    // One row per target: the flags table is the input, the two content tables are
    // joined only for the preview a moderator needs to judge it.
    let rows: Vec<(String, i64, i64, i64, bool, Option<String>, Option<String>, OffsetDateTime)> =
        sqlx::query_as(
            "select f.target_kind, f.target_id,
                    count(*)::bigint as flags,
                    count(*) filter (
                      where (select tl.level from public.trust_level(f.reporter_id) tl) >= 2
                    )::bigint as from_tl2,
                    coalesce(bool_or(
                      case when f.target_kind = 'log'
                           then (select l.hidden_at is not null from public.habit_logs l where l.id = f.target_id)
                           else (select c.hidden_at is not null from public.cage_checks c where c.id = f.target_id)
                      end), false) as hidden,
                    max(coalesce(l.note, c.note)) as preview,
                    max(coalesce(lp.username, cp.username)) as author,
                    min(f.created_at) as first_flag_at
             from public.flags f
             left join public.habit_logs l on f.target_kind = 'log' and l.id = f.target_id
             left join public.profiles lp on lp.id = l.user_id
             left join public.cage_checks c on f.target_kind = 'cage_check' and c.id = f.target_id
             left join public.profiles cp on cp.id = c.user_id
             group by f.target_kind, f.target_id
             order by flags desc, first_flag_at asc
             limit 50",
        )
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(target_kind, target_id, flags, from_tl2, hidden, preview, author, first)| QueueItem {
                target_kind,
                target_id,
                flags,
                from_tl2,
                hidden,
                preview,
                author,
                first_flag_at: rfc3339(first),
            })
            .collect(),
    ))
}

/// POST /api/moderation/action — hide (level 4 only) or restore (2 votes at level
/// 2+, or one leader). Every call writes an audit row, including the ones that do
/// not reach quorum yet.
pub async fn action(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<ActionRequest>,
) -> ApiResult<Json<ActionResponse>> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;
    if !valid_kind(&body.target_kind) {
        return Err(ApiError::bad_request("target_kind must be 'log' or 'cage_check'"));
    }
    let action = body.action.trim().to_lowercase();
    if action != "hide" && action != "restore" {
        return Err(ApiError::bad_request("action must be 'hide' or 'restore'"));
    }
    let level = level_of(&state.pool, user.id).await?;

    if action == "hide" {
        if level < LEADER_LEVEL {
            return Err(ApiError::new(
                StatusCode::FORBIDDEN,
                "Hiding needs trust level 4 — everyone else works through flags",
            ));
        }
        sqlx::query("select public.hide_target($1, $2)")
            .bind(&body.target_kind)
            .bind(body.target_id)
            .execute(&state.pool)
            .await?;
        record_action(&state, Some(user.id), "hide", &body.target_kind, body.target_id, body.note.as_deref()).await?;
        return Ok(Json(ActionResponse {
            hidden: true,
            restore_votes: 0,
        }));
    }

    if level < MODERATOR_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Restore votes are for trust level 2",
        ));
    }

    // The vote itself is the audit row, which is also what the quorum counts — one
    // statement, so a failed audit cannot leave a vote uncounted.
    record_action(&state, Some(user.id), "restore", &body.target_kind, body.target_id, body.note.as_deref()).await?;
    let votes: i64 = sqlx::query_scalar(
        "select count(distinct actor_id)::bigint from public.moderation_actions
         where target_kind = $1 and target_id = $2 and action = 'restore'
           and created_at > now() - interval '30 days'",
    )
    .bind(&body.target_kind)
    .bind(body.target_id)
    .fetch_one(&state.pool)
    .await?;

    let hidden = if votes >= RESTORE_QUORUM || level >= LEADER_LEVEL {
        sqlx::query("select public.restore_target($1, $2)")
            .bind(&body.target_kind)
            .bind(body.target_id)
            .execute(&state.pool)
            .await?;
        false
    } else {
        // Still hidden: the caller is told how many votes exist so the UI can say
        // "1 of 2" rather than pretending the click did nothing.
        let currently_hidden: bool = if body.target_kind == "log" {
            sqlx::query_scalar("select hidden_at is not null from public.habit_logs where id = $1")
                .bind(body.target_id)
                .fetch_one(&state.pool)
                .await?
        } else {
            sqlx::query_scalar("select hidden_at is not null from public.cage_checks where id = $1")
                .bind(body.target_id)
                .fetch_one(&state.pool)
                .await?
        };
        currently_hidden
    };

    Ok(Json(ActionResponse {
        hidden,
        restore_votes: votes,
    }))
}

async fn record_action(
    state: &AppState,
    actor: Option<uuid::Uuid>,
    action: &str,
    kind: &str,
    target: i64,
    note: Option<&str>,
) -> ApiResult<()> {
    sqlx::query(
        "insert into public.moderation_actions (actor_id, action, target_kind, target_id, note)
         values ($1, $2, $3, $4, $5)",
    )
    .bind(actor)
    .bind(action)
    .bind(kind)
    .bind(target)
    .bind(note)
    .execute(&state.pool)
    .await?;
    Ok(())
}
