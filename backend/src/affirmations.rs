//! The drill deck — curated lines, member lines, and the voice that admits them.
//!
//! Owner (2026-10-29): "allow users to add entries to the affirmation drill".
//!
//! Before this module the deck was a TypeScript array, which meant the only person who
//! could add a line was the person who could deploy. The deck is now a table with three
//! states, and the rules are deliberately the site's existing currency — trust, not
//! an editor:
//!
//! * a member **submits** a line; it starts `pending`;
//! * a **level-2 voice** admits it (`live`) — or the author's own level does, at the
//!   moment of writing, because a member who has already earned level 2 has earned the
//!   benefit of the doubt;
//! * the author (or any level-2 voice) can **hide** it, and the author can **delete**
//!   it outright. A curated line (no author) can only be touched by a leader.
//!
//! Two things it refuses to do: let a submission skip review because the author is
//! impatient (level 0/1 is the queue), and let the deck carry the same line twice
//! (`affirmations_one_live_per_line`, case- and whitespace-insensitive). The unique
//! index is the rule, not a pre-check, so two simultaneous admissions resolve in the
//! database instead of in a race.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;

use crate::api::require_user;
use crate::error::{ApiError, ApiResult};
use crate::AppState;

/// Longest line the drill will read out. The 140-character note limit next door is
/// about the feed; a mantra is a sentence, and 200 is one sentence with room for a
/// comma list.
pub const MAX_LEN: usize = 200;
pub const MIN_LEN: usize = 3;

/// Submissions are the one public surface with no natural rate limit, so they get a
/// pair: one at a time, three a day. Enough to build a deck with friends, not enough
/// to drown it.
pub const SUBMITS_PER_HOUR: i64 = 1;
pub const SUBMITS_PER_DAY: i64 = 3;

/// The level that admits and hides other people's lines. Level 2 is the ladder's first
/// "this person contributes" rung; under the small-site band (§0019) it is two days of
/// logging away, which is the whole point of that band.
pub const REVIEW_LEVEL: i32 = 2;
/// The level that may touch a curated line. Same number as everywhere else on the site.
pub const LEADER_LEVEL: i32 = 4;

#[derive(Debug, Serialize)]
pub struct DeckItem {
    pub id: i64,
    pub body: String,
    pub source: String,
    /// `curated`, a member's name, or `you`.
    pub author: String,
    pub mine: bool,
    pub status: String,
    /// RFC 3339, so the page can print "added 2 days ago" without guessing the zone.
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeckResponse {
    pub deck: Vec<DeckItem>,
    /// The caller's own lines that are not live yet (pending, hidden).
    pub mine: Vec<DeckItem>,
    /// Pending lines from other members — empty unless the caller is level 2 or above.
    pub review: Vec<DeckItem>,
    pub can_review: bool,
    pub level: i32,
    pub review_level: i32,
    /// How many submissions the caller has left today; null when signed out.
    pub remaining_today: Option<i64>,
    pub max_len: usize,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub id: i64,
    pub body: String,
    pub status: String,
    /// True when the line went straight into the deck instead of the queue.
    pub live: bool,
    pub remaining_today: i64,
}

fn rfc3339(ts: time::OffsetDateTime) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Collapse every run of whitespace (including newlines) to one space and trim the
/// ends. A mantra is a single line: a pasted paragraph with line breaks would break the
/// drill's typing check and the deck's layout, and neither failure is explainable to the
/// person who pasted it.
fn tidy(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn level_of(state: &AppState, user_id: uuid::Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar("select level from public.trust_level($1)")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
}

async fn submitted_since(state: &AppState, user_id: uuid::Uuid, secs: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "select count(*) from public.affirmations
          where author_id = $1 and created_at > now() - make_interval(secs => $2)",
    )
    .bind(user_id)
    .bind(secs as f64)
    .fetch_one(&state.pool)
    .await
}

fn row_to_item(row: &sqlx::postgres::PgRow, me: Option<uuid::Uuid>) -> DeckItem {
    let author_id: Option<uuid::Uuid> = row.get("author_id");
    let username: Option<String> = row.get("username");
    let mine = me.is_some() && author_id == me;
    DeckItem {
        id: row.get("id"),
        body: row.get("body"),
        source: row.get("source"),
        author: if mine {
            "you".to_string()
        } else {
            username.unwrap_or_else(|| "curated".to_string())
        },
        mine,
        status: row.get("status"),
        created_at: rfc3339(row.get("created_at")),
    }
}

/// `GET /api/affirmations` — the deck, the caller's own queue, and (level 2+) the review
/// queue. Public: the deck is read out loud on a public page, so requiring a session to
/// see it would be theatre.
pub async fn list(State(state): State<AppState>, session: Session) -> ApiResult<Json<DeckResponse>> {
    let me = require_user(&session).await.ok().map(|u| u.id);
    let level = match me {
        Some(id) => level_of(&state, id).await?,
        None => 0,
    };
    let can_review = level >= REVIEW_LEVEL;

    let rows = sqlx::query(
        "select a.id, a.body, a.source, a.status, a.created_at, a.author_id, p.username
           from public.affirmations a
           left join public.profiles p on p.id = a.author_id
          where a.status = 'live'
             or ($1::uuid is not null and a.author_id = $1)
             or ($2 and a.status = 'pending')
          order by (a.status = 'live') desc, a.created_at desc, a.id desc",
    )
    .bind(me)
    .bind(can_review)
    .fetch_all(&state.pool)
    .await?;

    let mut deck = Vec::new();
    let mut mine = Vec::new();
    let mut review = Vec::new();
    for row in &rows {
        let item = row_to_item(row, me);
        if item.status == "live" {
            deck.push(item);
        } else if item.mine {
            mine.push(item);
        } else {
            review.push(item);
        }
    }

    let remaining_today = match me {
        Some(id) => Some((SUBMITS_PER_DAY - submitted_since(&state, id, 86_400).await?).max(0)),
        None => None,
    };

    Ok(Json(DeckResponse {
        deck,
        mine,
        review,
        can_review,
        level,
        review_level: REVIEW_LEVEL,
        remaining_today,
        max_len: MAX_LEN,
    }))
}

/// `POST /api/affirmations` — submit a line. `live` if the author is level 2+, otherwise
/// it waits in the queue that `/api/affirmations` shows to level-2 members.
pub async fn submit(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<SubmitRequest>,
) -> ApiResult<(StatusCode, Json<SubmitResponse>)> {
    let user = require_user(&session).await?;
    let text = tidy(&body.body);

    if text.chars().count() < MIN_LEN {
        return Err(ApiError::bad_request(
            "A mantra needs at least three characters and at least one word.",
        ));
    }
    if text.chars().count() > MAX_LEN {
        return Err(ApiError::bad_request(format!(
            "Keep it under {MAX_LEN} characters — the drill reads it out loud."
        )));
    }

    // Per hour first, so the message names the shorter wait when both are spent.
    if submitted_since(&state, user.id, 3_600).await? >= SUBMITS_PER_HOUR {
        return Err(ApiError::too_many_requests(
            "One line an hour. The deck is read, not shouted at.",
        )
        .with_retry_after(3_600));
    }
    let today = submitted_since(&state, user.id, 86_400).await?;
    if today >= SUBMITS_PER_DAY {
        return Err(ApiError::too_many_requests(format!(
            "{SUBMITS_PER_DAY} a day is the limit. Come back tomorrow."
        ))
        .with_retry_after(86_400));
    }

    let level = level_of(&state, user.id).await?;
    let live = level >= REVIEW_LEVEL;

    let inserted = sqlx::query(
        "insert into public.affirmations (author_id, body, source, status, reviewed_at, reviewed_by)
         values ($1, $2, 'member', case when $3 then 'live' else 'pending' end,
                 case when $3 then now() else null end,
                 case when $3 then $1::uuid else null end)
         returning id, status",
    )
    .bind(user.id)
    .bind(&text)
    .bind(live)
    .fetch_one(&state.pool)
    .await;

    let (id, status): (i64, String) = match inserted {
        Ok(row) => (row.get("id"), row.get("status")),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => {
            return Err(ApiError::conflict(
                "That line is already in the deck — same words, different order of events.",
            ));
        }
        Err(e) => return Err(e.into()),
    };

    let _ = crate::audit::record(
        &state.pool,
        Some(user.id),
        "affirmation.submitted",
        serde_json::json!({ "id": id, "status": status, "live": live }),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(SubmitResponse {
            id,
            body: text,
            status: status.clone(),
            live: status == "live",
            remaining_today: (SUBMITS_PER_DAY - today - 1).max(0),
        }),
    ))
}

/// `POST /api/affirmations/{id}/approve` — level 2+ admits a pending line.
pub async fn approve(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let user = require_user(&session).await?;
    let level = level_of(&state, user.id).await?;
    if level < REVIEW_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            format!("Level {REVIEW_LEVEL} admits lines to the deck. You are level {level}."),
        ));
    }

    let updated = sqlx::query(
        "update public.affirmations
            set status = 'live', reviewed_at = now(), reviewed_by = $2, hidden_at = null
          where id = $1 and status = 'pending'",
    )
    .bind(id)
    .bind(user.id)
    .execute(&state.pool)
    .await;

    match updated {
        Ok(r) if r.rows_affected() == 1 => {
            let _ = crate::audit::record(
                &state.pool,
                Some(user.id),
                "affirmation.approved",
                serde_json::json!({ "id": id }),
            )
            .await;
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(_) => Err(ApiError::not_found(
            "No pending line with that id — it was already admitted, or withdrawn.",
        )),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => Err(
            ApiError::conflict("An identical line is already live in the deck."),
        ),
        Err(e) => Err(e.into()),
    }
}

/// `POST /api/affirmations/{id}/hide` — the author, or any level-2 voice, takes a line
/// out of the deck. A hidden line is not deleted: the author can still see it in their
/// own list, and the row keeps the record of who hid it.
pub async fn hide(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let user = require_user(&session).await?;
    let level = level_of(&state, user.id).await?;

    let author: Option<Option<uuid::Uuid>> =
        sqlx::query_scalar("select author_id from public.affirmations where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    let Some(author_id) = author else {
        return Err(ApiError::not_found("No line with that id."));
    };

    let is_author = author_id == Some(user.id);
    if !is_author && level < REVIEW_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Only the person who wrote a line, or a level 2 member, can hide it.",
        ));
    }
    if author_id.is_none() && level < LEADER_LEVEL {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "That line is curated. Only the owner of the site can hide it.",
        ));
    }

    let r = sqlx::query(
        "update public.affirmations set status = 'hidden', hidden_at = now()
          where id = $1 and status <> 'hidden'",
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    if r.rows_affected() == 0 {
        return Err(ApiError::conflict("That line is already hidden."));
    }
    let _ = crate::audit::record(
        &state.pool,
        Some(user.id),
        "affirmation.hidden",
        serde_json::json!({ "id": id, "by_author": is_author }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/affirmations/{id}` — the author withdraws their own line for good; a
/// leader may delete any line. Deletion is the only way a line leaves the deck without a
/// trace of it remaining, which is why it is the author's own right and nobody else's.
pub async fn delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let user = require_user(&session).await?;
    let level = level_of(&state, user.id).await?;

    let author: Option<Option<uuid::Uuid>> =
        sqlx::query_scalar("select author_id from public.affirmations where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    let Some(author_id) = author else {
        return Err(ApiError::not_found("No line with that id."));
    };

    let allowed = author_id == Some(user.id) || level >= LEADER_LEVEL;
    if !allowed {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "You can delete your own lines. Curated ones belong to the site.",
        ));
    }

    sqlx::query("delete from public.affirmations where id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    let _ = crate::audit::record(
        &state.pool,
        Some(user.id),
        "affirmation.deleted",
        serde_json::json!({ "id": id, "own": author_id == Some(user.id) }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
