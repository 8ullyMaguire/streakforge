//! The numbers page, for the one person allowed to read it (idea #123).
//!
//! "Is this growing?" is not answerable from the site itself: the leaderboard says who is
//! loudest, `/api/total` says how many loads were logged, and neither separates three regulars
//! from thirty passers-by. This module exposes the three questions the owner asked for —
//! weekly active members, guest→account conversion, and week-1..4 retention — and nothing else.
//!
//! Two rules, both deliberate:
//!
//! * **Trust level 4 only.** The same bit that can hide anyone's post (`is_leader`) can read the
//!   aggregate. There is no "moderator" tier to leak it to, because there is no moderator tier.
//! * **Counts, never people.** Every figure is an aggregate over at least one week; no username,
//!   no per-member row, no "who is at risk of churning". A page that names people would be a
//!   surveillance tool wearing a dashboard's clothes, and this one is meant to answer a product
//!   question and then be closed.
//!
//! Reads are not audited. `audit_log` is a record of *actions* (hiding, erasing, granting), and
//! writing a row every time a leader opens a page would turn it into a browsing log — of the
//! leader. What protects members here is that the data is counts, not that access is logged.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use tower_sessions::Session;

use crate::api::require_user;
use crate::error::{ApiError, ApiResult};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct Overview {
    pub members: i64,
    pub active_1d: i64,
    pub active_7d: i64,
    pub new_7d: i64,
    pub guests_created: i64,
    pub guests_converted: i64,
    /// `None` when no guest has ever been created — a rate over an empty denominator is not 0,
    /// it is undefined, and the page prints "—" for it.
    pub conversion_rate: Option<f64>,
    pub logs_1d: i64,
    pub logs_7d: i64,
    pub median_logs_per_active_7d: f64,
    pub verified_cage_secs: i64,
    pub affirmations_live: i64,
}

#[derive(Debug, Serialize)]
pub struct WeekRow {
    /// ISO date of the Monday that starts the week (UTC), as `YYYY-MM-DD`.
    pub week_start: String,
    pub active_members: i64,
    pub new_members: i64,
    pub logs: i64,
    pub guests: i64,
    pub guests_converted: i64,
}

#[derive(Debug, Serialize)]
pub struct CohortRow {
    pub cohort_start: String,
    pub cohort_size: i64,
    /// `None` while that week has not happened yet; the page prints "—", never 0%.
    pub w1: Option<i64>,
    pub w2: Option<i64>,
    pub w3: Option<i64>,
    pub w4: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminNumbers {
    pub overview: Overview,
    pub weekly: Vec<WeekRow>,
    pub retention: Vec<CohortRow>,
    /// RFC 3339 — the page says how old the numbers are, because "this week" moves.
    pub generated_at: String,
    /// The definitions, shipped with the numbers so the page cannot paraphrase them into
    /// something kinder (a "retention" that quietly counts anyone who ever logged would read
    /// as 60% forever).
    pub notes: Vec<String>,
}

/// `GET /api/admin/numbers` — trust level 4 only.
pub async fn numbers(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<AdminNumbers>> {
    let user = require_user(&session).await?;

    // `is_leader` is the level-4 bit; asking the column rather than `trust_level()` keeps this
    // one query away from the ladder, which is a different question ("may I moderate") that the
    // moderation module already answers.
    let is_leader: bool = sqlx::query_scalar("select is_leader from profiles where id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    if !is_leader {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "The numbers page belongs to the site's leader. Ask a leader if something there matters to you.",
        ));
    }

    // Every numeric column is cast in SQL (bigint / float8) — the INT4-into-i64 500 is this
    // codebase's most repeated runtime bug, and a cast costs nothing.
    let (members, active_1d, active_7d, new_7d, guests_created, guests_converted, conversion_rate, logs_1d, logs_7d, median, verified, deck): (
        i64, i64, i64, i64, i64, i64, Option<f64>, i64, i64, f64, i64, i64,
    ) = sqlx::query_as(
        "select members::bigint, active_1d::bigint, active_7d::bigint, new_7d::bigint,
                guests_created::bigint, guests_converted::bigint, conversion_rate::float8,
                logs_1d::bigint, logs_7d::bigint, median_logs_per_active_7d::float8,
                verified_cage_secs::bigint, affirmations_live::bigint
           from public.admin_overview()",
    )
    .fetch_one(&state.pool)
    .await?;

    let weekly: Vec<WeekRow> = sqlx::query_as::<_, (String, i64, i64, i64, i64, i64)>(
        "select week_start::text, active_members::bigint, new_members::bigint, logs::bigint,
                guests::bigint, guests_converted::bigint
           from public.admin_weekly(12) order by week_start",
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(week_start, active_members, new_members, logs, guests, guests_converted)| WeekRow {
        week_start,
        active_members,
        new_members,
        logs,
        guests,
        guests_converted,
    })
    .collect();

    let retention: Vec<CohortRow> = sqlx::query_as::<
        _,
        (String, i64, Option<i64>, Option<i64>, Option<i64>, Option<i64>),
    >(
        "select cohort_start::text, cohort_size::bigint,
                w1::bigint, w2::bigint, w3::bigint, w4::bigint
           from public.admin_retention(8) order by cohort_start desc",
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(cohort_start, cohort_size, w1, w2, w3, w4)| CohortRow {
        cohort_start,
        cohort_size,
        w1,
        w2,
        w3,
        w4,
    })
    .collect();

    Ok(Json(AdminNumbers {
        overview: Overview {
            members,
            active_1d,
            active_7d,
            new_7d,
            guests_created,
            guests_converted,
            conversion_rate,
            logs_1d,
            logs_7d,
            median_logs_per_active_7d: median,
            verified_cage_secs: verified,
            affirmations_live: deck,
        },
        weekly,
        retention,
        generated_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        notes: vec![
            "A member is a non-guest account. Guests and folded tombstones are counted separately."
                .into(),
            "Active means at least one log of any kind in the window — the question is whether someone used the site, not what they did with it.".into(),
            "Weeks are ISO weeks, Monday 00:00 UTC. The current week is included and incomplete."
                .into(),
            "A converted guest is counted in the week the guest first appeared, not the week it registered.".into(),
            "Retention W1..W4 is the share of a signup week still logging in the n-th week after it. A dash means that week has not happened yet, not that everyone left.".into(),
            "Median logs per active member is a median over members, not an average over logs: one heavy logger must not move the typical member.".into(),
        ],
    }))
}
