//! Cage checks — a claim, a seal, and peers who vouch.
//!
//! Owner (2026-10-29): "add cage check verified by users themselves; total verified
//! cage hours and similar metrics".
//!
//! This module is the whole of the honesty story for that sentence:
//!
//!   * the server never receives an image, so "verified" cannot mean "a machine
//!     looked at it" — it means other members said so;
//!   * every check carries a SEAL CODE issued at creation time, so a check that was
//!     copied off someone else's screen shows a code that belongs to a different
//!     row, and a verifier looking at two checks can see that;
//!   * hours only count while `status = 'verified'`, and a dispute from a trusted
//!     member takes them out again.
//!
//! The rules (quorum sizes, the 72h cap) are constants here AND in the migration's
//! comments, because a rule that lives only in a comment is a rule that drifts.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::Session;

use crate::api::{lock_info, require_user, LockInfoResponse};
use crate::error::{ApiError, ApiResult};
use crate::AppState;

/// A single check can never claim more than three days. Without the cap, silence
/// would become a way to bank hours.
pub const MAX_CLAIM_SECS: i64 = 259_200;
/// Distinct confirming members needed for a check to count.
pub const CONFIRM_QUORUM: i64 = 2;
/// One confirmation from a member at this level is enough on its own: with three
/// active accounts, requiring two trusted members would stall the queue forever.
pub const LONE_VERIFIER_LEVEL: i32 = 3;
/// Trust level required to attest at all.
pub const ATTEST_LEVEL: i32 = 1;

fn rfc3339(ts: OffsetDateTime) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// 8 hex chars: short enough to write on a card next to the cage, wide enough
/// that guessing another row's code is not a shortcut to anything.
fn seal_code() -> String {
    use rand::Rng;
    let n: u32 = rand::thread_rng().gen();
    format!("{n:08x}")
}

#[derive(Debug, Serialize)]
pub struct CageMetrics {
    pub checks: i64,
    pub claimed_secs: i64,
    pub verified_secs: i64,
    pub pending_secs: i64,
    pub disputed_secs: i64,
    pub verification_rate: f64,
    pub avg_secs: f64,
    pub current_verified_day_streak: i64,
    pub longest_verified_day_streak: i64,
    pub last_check_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CageCheckView {
    pub id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub checked_at: String,
    pub seal_code: String,
    pub note: Option<String>,
    pub claimed_secs: i64,
    pub status: String,
    pub confirms: i64,
    pub disputes: i64,
    /// This caller's own verdict on the check, if they already gave one.
    pub my_verdict: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CageResponse {
    pub lock: LockInfoResponse,
    pub metrics: CageMetrics,
    /// Today's check, if the members already made one — the UI uses it to show the
    /// seal they were given rather than offering a second button that would 429.
    pub today: Option<CageCheckView>,
    pub recent: Vec<CageCheckView>,
    pub queue_count: i64,
    pub total_verified_secs: i64,
}

#[derive(Debug, Deserialize)]
pub struct CheckRequest {
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttestRequest {
    pub verdict: String,
}

type ViewRow = (
    i64,
    String,
    Option<String>,
    OffsetDateTime,
    String,
    Option<String>,
    i64,
    String,
    i64,
    i64,
    Option<String>,
);

const VIEW_COLS: &str = "c.id, p.username, p.avatar_url, c.checked_at, c.seal_code, c.note,
     c.claimed_secs::bigint, c.status,
     (select count(*) from public.cage_attestations a
        where a.check_id = c.id and a.verdict = 'confirm')::bigint,
     (select count(*) from public.cage_attestations a
        where a.check_id = c.id and a.verdict = 'dispute')::bigint,
     (select a.verdict from public.cage_attestations a
        where a.check_id = c.id and a.attester_id = $1)";

fn view(row: ViewRow) -> CageCheckView {
    let (id, username, avatar_url, checked_at, seal_code, note, claimed_secs, status, confirms, disputes, my_verdict) = row;
    CageCheckView {
        id,
        username,
        avatar_url,
        checked_at: rfc3339(checked_at),
        seal_code,
        note,
        claimed_secs,
        status,
        confirms,
        disputes,
        my_verdict,
    }
}

async fn metrics(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> ApiResult<CageMetrics> {
    let (
        checks,
        claimed_secs,
        verified_secs,
        pending_secs,
        disputed_secs,
        verification_rate,
        avg_secs,
        current_verified_day_streak,
        longest_verified_day_streak,
        last_check_at,
    ): (i64, i64, i64, i64, i64, f64, f64, i64, i64, Option<OffsetDateTime>) =
        sqlx::query_as(
            "select checks, claimed_secs, verified_secs, pending_secs, disputed_secs,
                    verification_rate::float8, avg_secs::float8,
                    current_verified_day_streak::bigint, longest_verified_day_streak::bigint,
                    last_check_at
             from public.cage_check_hours($1)",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(CageMetrics {
        checks,
        claimed_secs,
        verified_secs,
        pending_secs,
        disputed_secs,
        verification_rate,
        avg_secs,
        current_verified_day_streak,
        longest_verified_day_streak,
        last_check_at: last_check_at.map(rfc3339),
    })
}

async fn my_check_today(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> ApiResult<Option<CageCheckView>> {
    let row: Option<ViewRow> = sqlx::query_as(&format!(
        "select {VIEW_COLS}
         from public.cage_checks c join public.profiles p on p.id = c.user_id
         where c.user_id = $1 and c.check_date = (now() at time zone 'utc')::date
         limit 1"
    ))
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(view))
}

/// GET /api/cage — lock state, my verified hours, today's check, the queue size.
pub async fn get_cage(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<CageResponse>> {
    let user = require_user(&session).await?;
    let lock = lock_info(&state.pool, user.id).await?;
    let metrics = metrics(&state.pool, user.id).await?;
    let today = my_check_today(&state.pool, user.id).await?;
    let recent: Vec<ViewRow> = sqlx::query_as(&format!(
        "select {VIEW_COLS}
         from public.cage_checks c join public.profiles p on p.id = c.user_id
         where c.user_id = $1 and c.hidden_at is null
         order by c.checked_at desc limit 10"
    ))
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    let queue_count: i64 = sqlx::query_scalar(
        "select count(*) from public.cage_checks
         where status = 'pending' and hidden_at is null and user_id <> $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let total_verified_secs: i64 =
        sqlx::query_scalar("select public.total_verified_cage_hours()")
            .fetch_one(&state.pool)
            .await?;
    Ok(Json(CageResponse {
        lock,
        metrics,
        today,
        recent: recent.into_iter().map(view).collect(),
        queue_count,
        total_verified_secs,
    }))
}

/// POST /api/cage-check — record today's check. Requires an open lock: a cage check
/// with no cage is a claim about nothing.
pub async fn create_check(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<CheckRequest>,
) -> ApiResult<(StatusCode, Json<CageCheckView>)> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;

    let note = body.note.unwrap_or_default();
    let note = note.trim();
    if note.chars().count() > 140 {
        return Err(ApiError::bad_request("Note must be 140 characters or fewer"));
    }
    let note = if note.is_empty() { None } else { Some(note.to_string()) };

    // The lock the check is about. One query, because the claimed hours are
    // measured from its start when there is no earlier check.
    let open: Option<(i64, OffsetDateTime)> = sqlx::query_as(
        "select id, locked_at from public.lock_sessions
         where user_id = $1 and unlocked_at is null
         order by locked_at desc limit 1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;
    let Some((lock_id, locked_at)) = open else {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "No open lock — lock yourself before checking the cage",
        ));
    };

    let previous: Option<OffsetDateTime> =
        sqlx::query_scalar("select max(checked_at) from public.cage_checks where user_id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    let from = previous.unwrap_or(locked_at).max(locked_at);
    let now = OffsetDateTime::now_utc();
    let claimed = (now - from).whole_seconds().clamp(0, MAX_CLAIM_SECS);

    let seal = seal_code();
    let id: i64 = match sqlx::query_scalar(
        "insert into public.cage_checks (user_id, lock_id, seal_code, note, claimed_secs)
         values ($1, $2, $3, $4, $5) returning id",
    )
    .bind(user.id)
    .bind(lock_id)
    .bind(&seal)
    .bind(&note)
    .bind(claimed as i32)
    .fetch_one(&state.pool)
    .await
    {
        Ok(id) => id,
        // One check per UTC day is a unique index, and the race between two clicks
        // is the ordinary way to reach it — answer in the same words as the check.
        Err(sqlx::Error::Database(db)) if db.constraint().is_some_and(|c| c.contains("cage_checks_user_id_check_date_key")) => {
            return Err(ApiError::too_many_requests(
                "A cage check is already recorded today (UTC). One a day is what makes the hours mean something.",
            ));
        }
        Err(other) => return Err(other.into()),
    };

    let row: ViewRow = sqlx::query_as(&format!(
        "select {VIEW_COLS}
         from public.cage_checks c join public.profiles p on p.id = c.user_id
         where c.id = $2"
    ))
    .bind(user.id)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(view(row))))
}

/// GET /api/cage-check/queue — other members' pending checks. Excludes my own and
/// hidden rows; ordering is oldest-first so nothing rots at the back.
pub async fn queue(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<Vec<CageCheckView>>> {
    let user = require_user(&session).await?;
    let rows: Vec<ViewRow> = sqlx::query_as(&format!(
        "select {VIEW_COLS}
         from public.cage_checks c join public.profiles p on p.id = c.user_id
         where c.status = 'pending' and c.hidden_at is null and c.user_id <> $1
         order by c.checked_at asc limit 20"
    ))
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows.into_iter().map(view).collect()))
}

/// POST /api/cage-check/{id}/attest — vouch for (or dispute) someone else's check.
///
/// The quorum is applied here rather than in a trigger so the caller can be told
/// what their attestation did; the SQL layer stays a set of plain statements.
pub async fn attest(
    State(state): State<AppState>,
    session: Session,
    Path(check_id): Path<i64>,
    Json(body): Json<AttestRequest>,
) -> ApiResult<Json<CageCheckView>> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;

    let verdict = body.verdict.trim().to_lowercase();
    if verdict != "confirm" && verdict != "dispute" {
        return Err(ApiError::bad_request("verdict must be 'confirm' or 'dispute'"));
    }

    let level: i32 = sqlx::query_scalar("select level from public.trust_level($1)")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    if level < ATTEST_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Attesting needs trust level 1 — a day old and three logs",
        ));
    }

    let owner: Option<uuid::Uuid> =
        sqlx::query_scalar("select user_id from public.cage_checks where id = $1")
            .bind(check_id)
            .fetch_optional(&state.pool)
            .await?;
    let Some(owner) = owner else {
        return Err(ApiError::not_found("No such cage check"));
    };
    if owner == user.id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "You cannot verify your own cage — that is the whole point",
        ));
    }

    sqlx::query(
        "insert into public.cage_attestations (check_id, attester_id, verdict)
         values ($1, $2, $3)
         on conflict (check_id, attester_id) do update set verdict = excluded.verdict",
    )
    .bind(check_id)
    .bind(user.id)
    .bind(&verdict)
    .execute(&state.pool)
    .await?;

    if verdict == "dispute" && level >= 2 {
        sqlx::query(
            "update public.cage_checks set status = 'disputed', verified_at = null where id = $1",
        )
        .bind(check_id)
        .execute(&state.pool)
        .await?;
    } else if verdict == "confirm" {
        // Two independent level-1+ voices, or one level-3+ voice.
        let (confirms, trusted): (i64, i64) = sqlx::query_as(
            "select
               count(*)::bigint,
               count(*) filter (where (select tl.level from public.trust_level(a.attester_id) tl) >= $2)::bigint
             from public.cage_attestations a
             where a.check_id = $1 and a.verdict = 'confirm'",
        )
        .bind(check_id)
        .bind(LONE_VERIFIER_LEVEL)
        .fetch_one(&state.pool)
        .await?;
        if confirms >= CONFIRM_QUORUM || trusted >= 1 {
            sqlx::query(
                "update public.cage_checks set status = 'verified', verified_at = now() where id = $1",
            )
            .bind(check_id)
            .execute(&state.pool)
            .await?;
        }
    }

    let row: ViewRow = sqlx::query_as(&format!(
        "select {VIEW_COLS}
         from public.cage_checks c join public.profiles p on p.id = c.user_id
         where c.id = $2"
    ))
    .bind(user.id)
    .bind(check_id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(view(row)))
}
