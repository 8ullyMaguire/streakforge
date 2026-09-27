// API handlers — logging (rate-limited), stats, leaderboards, feed, profiles.
// Rate limits (server-enforced), all keyed on the UTC calendar day so they
// agree with user_streak_kind and with the Devotion Index multiplier:
//   - max 1 log per UTC day per user, per kind (habit / affirmation / denial)
//   - a waste (habit) and a denial (WLD) are mutually exclusive on the same day:
//     recording one invalidates the other's eligibility that day
//   - DAILY_LIMIT is retained only as a backstop against accidental duplicates;
//     the cadence is the 1-per-UTC-day rule above.

use crate::auth::SessionUser;
use crate::error::{ApiError, ApiResult};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use time::OffsetDateTime;
use tower_sessions::Session;

pub const DAILY_LIMIT: i64 = 5;
/// Cadence: one submission per UTC calendar day, per kind.
/// Matches user_streak_kind (UTC-day streaks) and the Devotion Index
/// multiplier, so "submit once a day to grow your streak" is literally true.
pub const DAILY_WINDOW_HOURS: i64 = 24;

// ---- Helpers ----

pub async fn require_user(session: &Session) -> ApiResult<SessionUser> {
    session
        .get::<SessionUser>(crate::auth::SESSION_USER_KEY)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::unauthorized("Sign in required"))
}

// ---- DTOs ----

#[derive(Debug, Deserialize)]
pub struct LogRequest {
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub stats: StatsResponse,
}

#[derive(Debug, Serialize)]
pub struct DrillResponse {
    pub stats: StatsResponse,
}

/// Chastity-lock state for the denial page / profile.
#[derive(Debug, Serialize, Clone)]
pub struct LockInfoResponse {
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<String>,
    /// current lock duration in seconds (0 when unlocked)
    pub current_duration_secs: i64,
    /// total time locked (all sessions) in seconds
    pub total_locked_secs: i64,
    /// longest single lock in seconds
    pub longest_lock_secs: i64,
    /// consecutive lock periods (allowing a <=24h unlock gap)
    pub current_lock_streak: i64,
    pub longest_lock_streak: i64,
}

#[derive(Debug, Serialize)]
pub struct DenialResponse {
    pub stats: StatsResponse,
    pub lock: LockInfoResponse,
    /// seconds until the next denial can be reported, or null if allowed now
    pub next_denial_allowed_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatsResponse {
    pub today_count: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub week_count: i64,
    pub alltime_count: i64,
    pub recent_logs: Vec<FeedItem>,
    pub last_60m: i64,
    pub can_log: bool,
    pub next_allowed_at: Option<String>,
    /// Seconds until the daily cadence resets (next UTC midnight).
    /// 0 when allowed now. The UI ticks this down locally so the countdown
    /// advances without a reload.
    pub next_allowed_in_secs: i64,
    /// True when today's submission is blocked because the OTHER exclusive kind
    /// already ran (a waste blocks a denial, a denial blocks a waste).
    pub blocked_by_exclusivity: bool,
    /// Whiteboi Devotion Index for this user (all-time), decomposed.
    pub devotion: DevotionIndexResponse,
}

/// Score = ((WLW + WLD + Game Bonus) x Score Multiplier) - Racism Penalty.
/// `racism_penalty` is always 0 today: there is no reporting data source for it.
#[derive(Debug, Serialize, Clone)]
pub struct DevotionIndexResponse {
    pub wlw: i64,
    pub wld: i64,
    pub game_bonus: i64,
    pub multiplier: f64,
    pub racism_penalty: f64,
    pub score: f64,
    /// Streak of the currently active exclusive kind (the most recent WLW or WLD).
    pub active_kind: String,
    pub active_streak: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct FeedItem {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub logged_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub user_id: uuid::Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    /// Whiteboi Devotion Index for the window: ((WLW + WLD + Game Bonus) x mult) - penalty
    pub score: f64,
    /// components, so the board can show how a score was reached
    pub wlw: i64,
    pub wld: i64,
    pub game_bonus: i64,
    pub multiplier: f64,
    pub racism_penalty: f64,
    /// active exclusive streak (most recent WLW or WLD) and which kind it is
    pub streak: i64,
    pub active_kind: String,
    /// legacy flat points, retained for the profile/feed shapes that use it
    pub points: i64,
    pub waste_count: i64,
    pub denial_count: i64,
    pub affirmation_count: i64,
    pub last_log_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub period: String,
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Debug, Serialize)]
pub struct UserOfTheDay {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    /// weighted points: denial = 10 pts, waste = 1 pt
    pub points: i64,
    pub first_log_at: String,
    pub alltime_count: i64,
}

#[derive(Debug, Serialize)]
pub struct FeedResponse {
    pub items: Vec<FeedItem>,
    pub next_cursor: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: uuid::Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_url: Option<String>,
    pub created_at: String,
    pub streak: i64,
    pub longest_streak: i64,
    pub today_count: i64,
    pub week_count: i64,
    pub alltime_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub social_url: Option<String>,
}

// ---- Rate limiting ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogKind {
    Habit,
    Affirmation,
    Denial,
}

impl LogKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogKind::Habit => "habit",
            LogKind::Affirmation => "affirmation",
            LogKind::Denial => "denial",
        }
    }
}

impl Default for LogKind {
    fn default() -> Self {
        LogKind::Habit
    }
}

pub fn parse_kind(s: Option<&str>) -> LogKind {
    match s {
        Some("affirmation") => LogKind::Affirmation,
        Some("denial") => LogKind::Denial,
        _ => LogKind::Habit,
    }
}

/// Next UTC midnight — the boundary the daily cadence resets on.
/// Countdowns are computed against this so the UI and the server agree.
pub fn next_utc_midnight(now: OffsetDateTime) -> OffsetDateTime {
    (now.date() + time::Duration::days(1)).midnight().assume_utc()
}

/// Enforce the daily (UTC) cadence. `pub` so integration tests can assert the
/// rule directly instead of only inferring it from an HTTP response.
pub async fn check_rate_limits(
    pool: &PgPool,
    user_id: uuid::Uuid,
    kind: LogKind,
) -> ApiResult<()> {
    let now = OffsetDateTime::now_utc();
    let today_start = now.date().midnight().assume_utc();

    // Counts for the current UTC day, per kind.
    let (habits_today, denials_today, kind_today): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit'  and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = 'denial' and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = $3        and log_date = $2)",
    )
        .bind(user_id)
        .bind(today_start.date())
        .bind(kind.as_str())
        .fetch_one(pool)
        .await?;

    let next = next_utc_midnight(now);
    let next_str = next
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    // WLWs and WLDs are mutually exclusive: recording a waste today makes a
    // denial today ineligible, and vice versa. Enforced as a hard block, since
    // the Devotion Index treats the two streaks as competing.
    if kind == LogKind::Denial && habits_today >= 1 {
        return Err(ApiError::too_many_requests(
            "You already recorded a load wasted today. WLWs and WLDs are mutually exclusive — a whiteboi who cums cannot claim a denial. Come back tomorrow.",
        ));
    }
    if kind == LogKind::Habit && denials_today >= 1 {
        return Err(ApiError::too_many_requests(
            "You already recorded a denial today. WLWs and WLDs are mutually exclusive — a denial day stands. Come back tomorrow.",
        ));
    }

    // One submission per UTC day for WLW/WLD. This is the cadence that grows a
    // streak, so it is the primary rule.
    //
    // Affirmations are exempt: the drill keeps its own budget (see DAILY_LIMIT
    // below) precisely so reps neither consume nor pollute the WLW/WLD
    // allowance, and a strict 1/day would gut it.
    if kind != LogKind::Affirmation && kind_today >= 1 {
        return Err(ApiError::too_many_requests(format!(
            "Already recorded a {} today. Submit once a day (UTC) to grow your streak — you can again at {}.",
            kind.as_str(),
            next_str
        )));
    }
    if kind == LogKind::Affirmation && kind_today >= DAILY_LIMIT {
        return Err(ApiError::too_many_requests(format!(
            "Daily limit reached ({} per day). Come back tomorrow.",
            DAILY_LIMIT
        )));
    }
    Ok(())
}

fn next_allowed_at(kind_today: i64) -> Option<String> {
    if kind_today >= 1 {
        let next = next_utc_midnight(OffsetDateTime::now_utc());
        Some(
            next.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        )
    } else {
        None
    }
}

/// For the denial page: when can this user next report a denial?
/// Returns seconds from now, or None if they can deny right now.
async fn denial_next_allowed(pool: &PgPool, user_id: uuid::Uuid) -> ApiResult<Option<i64>> {
    let now = OffsetDateTime::now_utc();
    let today = now.date();

    // One denial per UTC calendar day: the next opportunity is the next midnight.
    let (denials_today, wastes_today): (i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'denial' and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit'  and log_date = $2)",
    )
    .bind(user_id)
    .bind(today)
    .fetch_one(pool)
    .await?;

    if denials_today >= 1 || wastes_today >= 1 {
        let secs = (next_utc_midnight(now) - now).whole_seconds().max(0);
        return Ok(Some(secs));
    }
    Ok(None)
}

// ---- Handlers ----

pub async fn log_habit(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<LogRequest>,
) -> ApiResult<(StatusCode, Json<LogResponse>)> {
    let user = require_user(&session).await?;
    let note = body.note.unwrap_or_default();
    let note = note.trim();
    if note.chars().count() > 140 {
        return Err(ApiError::bad_request("Note must be 140 characters or fewer"));
    }
    let kind = parse_kind(body.kind.as_deref());

    check_rate_limits(&state.pool, user.id, kind).await?;

    sqlx::query("insert into habit_logs (user_id, note, kind) values ($1, $2, $3)")
        .bind(user.id)
        .bind(note)
        .bind(kind.as_str())
        .execute(&state.pool)
        .await?;

    let stats = compute_stats(&state.pool, user.id, kind).await?;
    Ok((StatusCode::CREATED, Json(LogResponse { stats })))
}

pub async fn get_stats(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<StatsResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Habit).await?;
    Ok(Json(stats))
}

/// Drill stats — affirmation-specific counters + rate-limit state.
pub async fn get_drill(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<DrillResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Affirmation).await?;
    Ok(Json(DrillResponse { stats }))
}

/// Denial page — denial-kind stats + chastity lock state.
pub async fn get_denial(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<DenialResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Denial).await?;
    let lock = lock_info(&state.pool, user.id).await?;
    let next_denial_allowed_in = denial_next_allowed(&state.pool, user.id).await?;
    Ok(Json(DenialResponse { stats, lock, next_denial_allowed_in }))
}

/// POST /api/lock — start (or restart) a chastity lock session.
pub async fn lock(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<(axum::http::StatusCode, Json<LockInfoResponse>)> {
    let user = require_user(&session).await?;
    // If there's an open session already, close it first (re-lock).
    sqlx::query("update lock_sessions set unlocked_at = now() where user_id = $1 and unlocked_at is null")
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    sqlx::query("insert into lock_sessions (user_id) values ($1)")
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    let lock = lock_info(&state.pool, user.id).await?;
    Ok((axum::http::StatusCode::CREATED, Json(lock)))
}

/// POST /api/unlock — end the current lock session.
pub async fn unlock(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<UnlockRequest>,
) -> ApiResult<Json<LockInfoResponse>> {
    let user = require_user(&session).await?;
    let reason = body.reason.unwrap_or_default();
    let reason = reason.trim();
    if reason.chars().count() > 60 {
        return Err(ApiError::bad_request("Reason max 60 chars"));
    }
    let reason = if reason.is_empty() { None } else { Some(reason.to_string()) };
    sqlx::query("update lock_sessions set unlocked_at = now(), reason = $2 where user_id = $1 and unlocked_at is null")
        .bind(user.id)
        .bind(&reason)
        .execute(&state.pool)
        .await?;
    let lock = lock_info(&state.pool, user.id).await?;
    Ok(Json(lock))
}

/// GET /api/lock — current lock state (for navbar/profile).
pub async fn get_lock(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<LockInfoResponse>> {
    let user = require_user(&session).await?;
    let lock = lock_info(&state.pool, user.id).await?;
    Ok(Json(lock))
}

async fn lock_info(pool: &PgPool, user_id: uuid::Uuid) -> ApiResult<LockInfoResponse> {
    let (locked, locked_at, current_dur, total_locked, longest): (
        bool,
        Option<OffsetDateTime>,
        Option<i64>,
        i64,
        i64,
    ) = sqlx::query_as(
        "select
           locked,
           locked_at,
           case when current_duration is null then null else extract(epoch from current_duration)::bigint end,
           extract(epoch from total_locked)::bigint,
           extract(epoch from longest_lock)::bigint
         from lock_info($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let (cur_streak, longest_streak): (i64, i64) = sqlx::query_as(
        "select current_streak::bigint, longest_streak::bigint from lock_streak($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(LockInfoResponse {
        locked,
        locked_at: locked_at.map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()),
        current_duration_secs: current_dur.unwrap_or(0),
        total_locked_secs: total_locked,
        longest_lock_secs: longest,
        current_lock_streak: cur_streak,
        longest_lock_streak: longest_streak,
    })
}

async fn compute_stats(pool: &PgPool, user_id: uuid::Uuid, kind: LogKind) -> ApiResult<StatsResponse> {
    let now = OffsetDateTime::now_utc();
    let hour_ago = now - time::Duration::minutes(60);
    let today_start = now.date().midnight().assume_utc();
    let week_start = week_start_utc(now);

    let (today_count, week_count, alltime_count): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = $4 and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and kind = $4 and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1 and kind = $4)",
    )
    .bind(user_id)
    .bind(week_start)
    .bind(today_start)
    .bind(kind.as_str())
    .fetch_one(pool)
    .await?;

    let last_60m: i64 = sqlx::query_scalar(
        "select count(*) from habit_logs where user_id = $1 and kind = $3 and logged_at >= $2",
    )
    .bind(user_id)
    .bind(hour_ago)
    .bind(kind.as_str())
    .fetch_one(pool)
    .await?;

    let (current_streak, longest_streak): (i64, i64) = sqlx::query_as(
        "select current_streak::bigint, longest_streak::bigint from user_streak_kind($1, $2)",
    )
    .bind(user_id)
    .bind(kind.as_str())
    .fetch_one(pool)
    .await?;

    let recent: Vec<FeedItem> = sqlx::query_as::<_, (i64, uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime, Option<String>)>(
        "select l.id, l.user_id, p.username, p.display_name, p.avatar_url, l.logged_at, l.note
         from habit_logs l join profiles p on p.id = l.user_id
         where l.user_id = $1 and l.kind = $3 order by l.logged_at desc limit 20",
    )
    .bind(user_id)
    .bind(today_start)
    .bind(kind.as_str())
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, uid, uname, dname, av, ts, note)| FeedItem {
        id,
        user_id: uid,
        username: uname,
        display_name: dname,
        avatar_url: av,
        logged_at: ts.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        note,
    })
    .collect();

    let (habits_today, denials_today, kind_today): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit'  and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = 'denial' and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = $3        and log_date = $2)",
    )
    .bind(user_id)
    .bind(today_start.date())
    .bind(kind.as_str())
    .fetch_one(pool)
    .await?;

    // WLWs and WLDs are mutually exclusive. An affirmation is neither, so it is
    // never blocked by exclusivity and never blocks anything.
    let blocked_by_exclusivity = match kind {
        LogKind::Habit => denials_today >= 1,
        LogKind::Denial => habits_today >= 1,
        LogKind::Affirmation => false,
    };

    // Mirrors check_rate_limits exactly: affirmations have their own budget and
    // are not subject to the 1-per-UTC-day cadence.
    let can_log = match kind {
        LogKind::Affirmation => kind_today < DAILY_LIMIT,
        _ => kind_today < 1 && !blocked_by_exclusivity,
    };
    // Seconds until the next UTC midnight. The UI ticks this down client-side,
    // so the countdown moves without a reload and flips can_log at zero.
    let next_in_secs = if can_log {
        0
    } else {
        (next_utc_midnight(now) - now).whole_seconds().max(0)
    };

    // Whiteboi Devotion Index (all-time) for this user.
    let (wlw, wld, game_bonus, mult, penalty, score): (i64, i64, i64, f64, f64, f64) = sqlx::query_as(
        "select wlw::bigint, wld::bigint, game_bonus::bigint,
                multiplier::float8, racism_penalty::float8, score::float8
         from devotion_index_alltime($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let (active_kind, active_streak): (String, i64) = sqlx::query_as(
        "select active_kind::text, streak::bigint from exclusive_streak($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(StatsResponse {
        today_count,
        current_streak,
        longest_streak,
        week_count,
        alltime_count,
        recent_logs: recent,
        last_60m,
        can_log,
        next_allowed_at: if can_log { None } else { next_allowed_at(kind_today) },
        next_allowed_in_secs: next_in_secs,
        blocked_by_exclusivity,
        devotion: DevotionIndexResponse {
            wlw,
            wld,
            game_bonus,
            multiplier: mult,
            racism_penalty: penalty,
            score,
            active_kind,
            active_streak,
        },
    })
}

fn week_start_utc(now: OffsetDateTime) -> OffsetDateTime {
    let days = now.weekday().number_days_from_monday() as i64;
    (now.date() - time::Duration::days(days)).midnight().assume_utc()
}

// ---- Leaderboards ----

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Path(period): Path<String>,
) -> ApiResult<Json<LeaderboardResponse>> {
    let period = match period.as_str() {
        "daily" => "daily",
        "weekly" => "weekly",
        "monthly" => "monthly",
        "alltime" => "alltime",
        _ => return Err(ApiError::bad_request("Unknown period")),
    };
    // The Devotion Index function returns (score, wlw, wld, game_bonus,
    // multiplier, streak, active_kind, last_log_at). The flat `points` and the
    // per-kind counts are derived here for the profile/feed compatibility fields.
    type Row = (
        uuid::Uuid, String, Option<String>, Option<String>,   // identity
        f64, i64, i64, i64, f64, f64,                        // score + components
        i64, String, Option<OffsetDateTime>,                 // streak, kind, last
    );
    let rows: Vec<Row> = match period {
        "daily" => sqlx::query_as("select * from daily_leaderboard_weighted(50)").fetch_all(&state.pool).await?,
        "weekly" => sqlx::query_as("select * from weekly_leaderboard_weighted(50)").fetch_all(&state.pool).await?,
        "monthly" => sqlx::query_as("select * from monthly_leaderboard_weighted(50)").fetch_all(&state.pool).await?,
        _ => sqlx::query_as("select * from alltime_leaderboard_weighted(50)").fetch_all(&state.pool).await?,
    };
    let entries = rows
        .into_iter()
        .enumerate()
        .map(|(i, (uid, uname, dname, av, score, wlw, wld, game_bonus, mult, penalty, streak, kind, last))| {
            // flat points kept for the legacy field: denial 10, waste 1, 3 affs 1
            let points = wld * 10 + wlw;
            LeaderboardEntry {
                rank: (i + 1) as i64,
                user_id: uid,
                username: uname,
                display_name: dname,
                avatar_url: av,
                score,
                wlw,
                wld,
                game_bonus,
                multiplier: mult,
                racism_penalty: penalty,
                streak,
                active_kind: kind,
                points,
                waste_count: wlw,
                denial_count: wld,
                affirmation_count: game_bonus * 3,
                last_log_at: last
                    .map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()),
            }
        })
        .collect();
    Ok(Json(LeaderboardResponse { period: period.into(), entries }))
}

pub async fn get_user_of_the_day(
    State(state): State<AppState>,
) -> ApiResult<Json<Option<UserOfTheDay>>> {
    let row: Option<(uuid::Uuid, String, Option<String>, Option<String>, i64, OffsetDateTime, i64)> =
        sqlx::query_as(
            "select p.id, p.username, p.display_name, p.avatar_url,
                    (
                      count(*) filter (where l.kind = 'denial') * 10
                      + count(*) filter (where l.kind = 'habit')
                      + floor(count(*) filter (where l.kind = 'affirmation') / 3)
                    )::bigint as points,
                    min(l.logged_at) as first_log_at,
                    (select count(*) from public.habit_logs a where a.user_id = p.id and a.kind in ('habit','denial','affirmation')) as alltime_count
             from public.habit_logs l
             join public.profiles p on p.id = l.user_id
             where l.logged_at >= now() - interval '24 hours'
               and l.kind in ('habit', 'denial', 'affirmation')
             group by p.id
             order by points desc, min(l.logged_at) asc
             limit 1",
        )
        .fetch_optional(&state.pool)
        .await?;
    Ok(Json(row.map(|(uid, uname, dname, av, points, first, all)| UserOfTheDay {
        user_id: uid,
        username: uname,
        display_name: dname,
        avatar_url: av,
        points,
        first_log_at: first
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        alltime_count: all,
    })))
}

pub async fn get_total(State(state): State<AppState>) -> ApiResult<Json<HashMap<&'static str, i64>>> {
    let total: i64 = sqlx::query_scalar("select count(*) from habit_logs where kind = 'habit'").fetch_one(&state.pool).await?;
    let denied: i64 = sqlx::query_scalar("select count(*) from habit_logs where kind = 'denial'").fetch_one(&state.pool).await?;
    Ok(Json(HashMap::from([("total", total), ("denied", denied)])))
}

// ---- Feed ----

pub async fn get_feed(
    State(state): State<AppState>,
    Query(params): Query<FeedParams>,
) -> ApiResult<Json<FeedResponse>> {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let cursor = params.cursor.unwrap_or(0);
    let rows: Vec<(i64, uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime, Option<String>)> =
        sqlx::query_as("select * from activity_feed($1, $2::int)")
            .bind(cursor)
            .bind(limit)
            .fetch_all(&state.pool)
            .await?;
    let items: Vec<FeedItem> = rows
        .into_iter()
        .map(|(id, uid, uname, dname, av, ts, note)| FeedItem {
            id,
            user_id: uid,
            username: uname,
            display_name: dname,
            avatar_url: av,
            logged_at: ts.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            note,
        })
        .collect();
    let next_cursor = if items.len() as i64 >= limit { items.last().map(|i| i.id) } else { None };
    Ok(Json(FeedResponse { items, next_cursor }))
}

#[derive(Debug, Deserialize)]
pub struct FeedParams {
    pub cursor: Option<i64>,
    pub limit: Option<i64>,
}

// ---- Profiles ----

pub async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ProfileResponse>> {
    let row: Option<(uuid::Uuid, String, Option<String>, Option<String>, Option<String>, OffsetDateTime)> =
        sqlx::query_as(
            "select id, username, display_name, avatar_url, social_url, created_at from profiles where username = $1",
        )
        .bind(&username)
        .fetch_optional(&state.pool)
        .await?;
    let Some((id, uname, dname, av, social_url, created)) = row else {
        return Err(ApiError::not_found("Profile not found"));
    };
    let (streak, longest): (i64, i64) = sqlx::query_as("select current_streak::bigint, longest_streak::bigint from user_streak($1)")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    let week_start = week_start_utc(OffsetDateTime::now_utc());
    let (today_count, week_count, alltime_count): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit')",
    )
    .bind(id)
    .bind(week_start)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ProfileResponse {
        id,
        username: uname,
        display_name: dname,
        avatar_url: av,
        social_url,
        created_at: created.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        streak,
        longest_streak: longest,
        today_count,
        week_count,
        alltime_count,
    }))
}

pub async fn update_profile(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<Json<ProfileResponse>> {
    let user = require_user(&session).await?;

    let mut username = user.username.clone();
    let mut display_name = user.display_name.clone();
    let mut avatar_url = user.avatar_url.clone();
    let mut social_url = user.social_url.clone();

    if let Some(u) = body.username {
        let u = u.trim().to_string();
        if !valid_username(&u) {
            return Err(ApiError::bad_request(
                "Username must be 2–30 chars, letters/digits/underscore only",
            ));
        }
        username = u;
    }
    if let Some(d) = body.display_name {
        let d = d.trim().to_string();
        if d.chars().count() > 50 {
            return Err(ApiError::bad_request("Display name max 50 chars"));
        }
        display_name = if d.is_empty() { None } else { Some(d) };
    }
    if let Some(a) = body.avatar_url {
        let a = a.trim().to_string();
        if !a.is_empty() && !(a.starts_with("https://") || a.starts_with("http://")) {
            return Err(ApiError::bad_request("Avatar URL must start with http(s)://"));
        }
        avatar_url = if a.is_empty() { None } else { Some(a) };
    }
    if let Some(s) = body.social_url {
        let s = s.trim().to_string();
        if !s.is_empty() && !(s.starts_with("https://") || s.starts_with("http://")) {
            return Err(ApiError::bad_request("Social URL must start with http(s)://"));
        }
        if s.chars().count() > 500 {
            return Err(ApiError::bad_request("Social URL max 500 chars"));
        }
        social_url = if s.is_empty() { None } else { Some(s) };
    }

    let updated = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, Option<String>, Option<String>, OffsetDateTime)>(
        "update profiles set username = $1, display_name = $2, avatar_url = $3, social_url = $4, updated_at = now()
         where id = $5
         returning id, username, display_name, avatar_url, social_url, created_at",
    )
    .bind(&username)
    .bind(&display_name)
    .bind(&avatar_url)
    .bind(&social_url)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, uname, dname, av, social_url, created)) = updated else {
        return Err(ApiError::not_found("Profile not found"));
    };

    // refresh session user
    let new_user = SessionUser {
        id,
        username: uname.clone(),
        display_name: dname.clone(),
        avatar_url: av.clone(),
        provider: user.provider,
        social_url: social_url.clone(),
    };
    session
        .insert(crate::auth::SESSION_USER_KEY, &new_user)
        .await
        .map_err(ApiError::from)?;

    let (streak, longest): (i64, i64) = sqlx::query_as("select current_streak::bigint, longest_streak::bigint from user_streak($1)")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    let week_start = week_start_utc(OffsetDateTime::now_utc());
    let (today_count, week_count, alltime_count): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit')",
    )
    .bind(id)
    .bind(week_start)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ProfileResponse {
        id,
        username: uname,
        display_name: dname,
        avatar_url: av,
        social_url,
        created_at: created.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        streak,
        longest_streak: longest,
        today_count,
        week_count,
        alltime_count,
    }))
}

fn valid_username(u: &str) -> bool {
    !u.is_empty()
        && u.chars().count() <= 30
        && u.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}
