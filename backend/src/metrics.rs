//! Metrics that the site shows but does not score: time to cum, and the boards
//! the brainstorm asked for (most denied, consistency, lock streak).
//!
//! "Time to cum" is the OWNER's definition (2026-10-29): "time it takes from start
//! jerking to release" — a duration the author reports per log, not the gap between
//! releases. That is why the column lives on `habit_logs` and why an empty report is
//! normal: the number is optional everywhere.
//!
//! The two averages are both published on purpose. `avg_secs` is pooled over logs
//! (one person who logs five times weighs five times more), `avg_of_user_avgs_secs`
//! gives every person one vote. Averages of averages are usually a mistake; here the
//! owner asked for both, so both are named rather than one being implied.

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use tower_sessions::Session;

use crate::auth::SESSION_USER_KEY;
use crate::error::ApiResult;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct TimeToCumStats {
    pub samples: i64,
    pub avg_secs: f64,
    pub median_secs: f64,
    pub min_secs: i64,
    pub max_secs: i64,
    pub last_secs: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TimeToCumAggregate {
    pub samples: i64,
    pub avg_secs: f64,
    pub median_secs: f64,
    pub avg_of_user_avgs_secs: f64,
    pub reporters: i64,
}

#[derive(Debug, Serialize)]
pub struct EdgeEntry {
    pub username: String,
    pub avatar_url: Option<String>,
    pub samples: i64,
    pub avg_secs: f64,
}

#[derive(Debug, Serialize)]
pub struct TimeToCumResponse {
    pub aggregate: TimeToCumAggregate,
    /// The caller's own numbers, when signed in. `None` for an anonymous read.
    pub mine: Option<TimeToCumStats>,
    pub board: Vec<EdgeEntry>,
}

/// GET /api/metrics/time-to-cum — public aggregate and board, plus the caller's own
/// figures when there is a session. Public on purpose: the landing page and the
/// metrics page both want the aggregate without asking anyone to sign in.
pub async fn time_to_cum(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<TimeToCumResponse>> {
    let (samples, avg_secs, median_secs, avg_of_user_avgs_secs, reporters): (i64, f64, f64, f64, i64) =
        sqlx::query_as(
            "select samples::bigint, avg_secs::float8, median_secs::float8,
                    avg_of_user_avgs_secs::float8, reporters::bigint
             from public.time_to_cum_aggregate()",
        )
        .fetch_one(&state.pool)
        .await?;

    let board: Vec<(String, Option<String>, i64, f64)> = sqlx::query_as(
        "select username, avatar_url, samples::bigint, avg_secs::float8
         from public.time_to_cum_board(10)",
    )
    .fetch_all(&state.pool)
    .await?;

    let me = session
        .get::<crate::auth::SessionUser>(SESSION_USER_KEY)
        .await
        .ok()
        .flatten();
    let mine = match me {
        None => None,
        Some(user) => {
            let (samples, avg_secs, median_secs, min_secs, max_secs, last_secs): (
                i64,
                f64,
                f64,
                i32,
                i32,
                Option<i32>,
            ) = sqlx::query_as(
                "select samples::bigint, avg_secs::float8, median_secs::float8,
                        min_secs, max_secs, last_secs
                 from public.time_to_cum_stats($1)",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            Some(TimeToCumStats {
                samples,
                avg_secs,
                median_secs,
                min_secs: min_secs as i64,
                max_secs: max_secs as i64,
                last_secs: last_secs.map(|v| v as i64),
            })
        }
    };

    Ok(Json(TimeToCumResponse {
        aggregate: TimeToCumAggregate {
            samples,
            avg_secs,
            median_secs,
            avg_of_user_avgs_secs,
            reporters,
        },
        mine,
        board: board
            .into_iter()
            .map(|(username, avatar_url, samples, avg_secs)| EdgeEntry {
                username,
                avatar_url,
                samples,
                avg_secs,
            })
            .collect(),
    }))
}

#[derive(Debug, Serialize)]
pub struct DeniedEntry {
    pub username: String,
    pub avatar_url: Option<String>,
    pub denials: i64,
}

#[derive(Debug, Serialize)]
pub struct ConsistencyEntry {
    pub username: String,
    pub avatar_url: Option<String>,
    pub days: i64,
    pub pct: f64,
}

#[derive(Debug, Serialize)]
pub struct LockEntry {
    pub username: String,
    pub avatar_url: Option<String>,
    pub current_streak: i64,
    pub longest_streak: i64,
}

#[derive(Debug, Serialize)]
pub struct BoardsResponse {
    /// Last 30 days, by denial count — the behaviour the score pays 10x for.
    pub most_denied: Vec<DeniedEntry>,
    /// Last 30 days, by percentage of days with any log.
    pub consistency: Vec<ConsistencyEntry>,
    pub locks: Vec<LockEntry>,
}

/// GET /api/metrics/boards — the three boards that were missing. Public, like the
/// main board: a ranking nobody can see is a ranking nobody competes for.
pub async fn boards(State(state): State<AppState>) -> ApiResult<Json<BoardsResponse>> {
    let denied: Vec<(String, Option<String>, i64)> = sqlx::query_as(
        "select username, avatar_url, denials::bigint from public.most_denied_board(30, 10)",
    )
    .fetch_all(&state.pool)
    .await?;
    let consistency: Vec<(String, Option<String>, i64, f64)> = sqlx::query_as(
        "select username, avatar_url, days::bigint, pct::float8 from public.consistency_board(30, 10)",
    )
    .fetch_all(&state.pool)
    .await?;
    let locks: Vec<(String, Option<String>, i32, i32)> = sqlx::query_as(
        "select username, avatar_url, current_streak, longest_streak
         from public.lock_streak_board(10)",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(BoardsResponse {
        most_denied: denied
            .into_iter()
            .map(|(username, avatar_url, denials)| DeniedEntry {
                username,
                avatar_url,
                denials,
            })
            .collect(),
        consistency: consistency
            .into_iter()
            .map(|(username, avatar_url, days, pct)| ConsistencyEntry {
                username,
                avatar_url,
                days,
                pct,
            })
            .collect(),
        locks: locks
            .into_iter()
            .map(|(username, avatar_url, current_streak, longest_streak)| LockEntry {
                username,
                avatar_url,
                current_streak: current_streak as i64,
                longest_streak: longest_streak as i64,
            })
            .collect(),
    }))
}
