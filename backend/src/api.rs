// API handlers — logging (rate-limited), stats, leaderboards, feed, profiles.
// Rate limits (server-enforced), all keyed on the UTC calendar day so they
// agree with user_streak_kind and with the Devotion Index multiplier:
//   - loads wasted (kind = 'habit'): WLW_DAILY_LIMIT per UTC day (default 5) —
//     a daily budget, not a one-shot cadence
//   - denials (kind = 'denial'): 1 per UTC day. A denial day is a stand;
//     "I denied five times today" is not a thing
//   - affirmations (kind = 'affirmation'): DAILY_LIMIT per UTC day, their own
//     budget so drill reps neither consume nor pollute the waste allowance
//   - a waste (habit) and a denial (WLD) are exclusive in ONE direction on the
//     same UTC day: a recorded waste spends the denial's day (a whiteboi who cums
//     cannot claim a denial), but a recorded denial does not touch the waste
//     budget — a relapse after a stand is the event the site counts
// `daily_budget()` is the single source of truth for all three; check_rate_limits
// and compute_stats both read it so the rule can't drift between enforcement and
// what the UI is told.

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
/// Cadence for a kind that is one-per-UTC-day: denials.
/// Matches user_streak_kind (UTC-day streaks) and the Devotion Index
/// multiplier, so "submit once a day to grow your streak" is literally true.
pub const DAILY_WINDOW_HOURS: i64 = 24;

/// Submissions allowed per UTC day for `kind`.
///
/// Single source of truth: `check_rate_limits` enforces it and `compute_stats`
/// reports it, so the limit the UI shows is the limit the server applies.
/// `wlw_daily_limit` is the operator's `WLW_DAILY_LIMIT` (default 5).
pub fn daily_budget(kind: LogKind, wlw_daily_limit: i64) -> i64 {
    match kind {
        LogKind::Habit => wlw_daily_limit,
        LogKind::Denial => 1,
        LogKind::Affirmation => DAILY_LIMIT,
    }
}

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
    /// Owner's definition (2026-10-29): how long the session ran, from starting to
    /// release. Optional on every path — the one-click landing button must not
    /// demand a number, and an empty report is not a missing value to guess at.
    #[serde(default)]
    pub time_to_cum_secs: Option<i64>,
    /// Which UTC day the log belongs to: `"today"` (the default) or `"yesterday"`.
    /// Yesterday is the catch-up window — one per member per filing day, only when
    /// that day is otherwise empty, so a forgotten log repairs the streak instead
    /// of inventing history. Anything else is a 400.
    #[serde(default)]
    pub day: Option<String>,
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
    /// Submissions this kind allows per UTC day (the server's own budget:
    /// `WLW_DAILY_LIMIT` for a waste, 1 for a denial, `DAILY_LIMIT` for reps).
    /// Sent rather than duplicated in the UI so the number shown is the number
    /// enforced.
    pub daily_limit: i64,
    /// Submissions still available for this kind today, i.e.
    /// `daily_limit - today_count`, floored at 0. Drives "3 of 5 left" and the
    /// enabled state together with `blocked_by_exclusivity`.
    pub remaining_today: i64,
    /// Seconds until the next log is PERMITTED: 0 when one is allowed now.
    /// This is about the lock, not about the day -- see `secs_until_reset`.
    pub next_allowed_in_secs: i64,
    /// Seconds until the daily budget refills (next UTC midnight), regardless of
    /// whether a log is allowed right now.
    ///
    /// WHY THIS IS A SECOND FIELD (docs/100_IDEAS.md #15 asks the dashboard to
    /// say "3 of 5 left, resets in 4h12m"): `next_allowed_in_secs` is zeroed
    /// whenever `can_log` is true, which is the normal state for most of the day,
    /// so the one countdown the item is about was the one case it could not
    /// report. The two questions are genuinely different -- "may I log now" and
    /// "when does the allowance come back" -- and they were sharing an answer.
    pub secs_until_reset: i64,
    /// True when today's submission is blocked by the one-way exclusivity rule:
    /// a recorded waste blocks a denial for the rest of the UTC day. Never true
    /// for a waste (a denial does not block one) or for a drill.
    pub blocked_by_exclusivity: bool,
    /// The caller's own time-to-cum numbers (owner's definition: start → release).
    pub time_to_cum: TimeToCumMine,
    /// Each kind's exclusive streak, side by side — the one-way rule made legible.
    pub kind_streaks: KindStreaks,
    /// The multiplier tiers, with which ones the run has already reached.
    pub milestones: Vec<MilestoneTier>,
    /// True when skipping today costs the multiplier the member already earned.
    pub at_risk: AtRisk,
    /// The catch-up window, as the server sees it: whether yesterday is empty (the
    /// only case a catch-up may fill) and whether one has already been filed today.
    /// Sent rather than inferred in the UI, because the rule is a database fact.
    pub catch_up: CatchUpState,
    /// Whiteboi Devotion Index for this user (all-time), decomposed.
    pub devotion: DevotionIndexResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct TimeToCumMine {
    pub samples: i64,
    pub avg_secs: f64,
    pub median_secs: f64,
    pub min_secs: i64,
    pub max_secs: i64,
    pub last_secs: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct KindStreaks {
    pub habit: i64,
    pub denial: i64,
    pub affirmation: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct MilestoneTier {
    pub days: i64,
    pub multiplier: f64,
    pub reached: bool,
}

/// Whether the member may file a log for yesterday, and why not.
#[derive(Debug, Serialize, Clone)]
pub struct CatchUpState {
    pub available: bool,
    pub filed_today: bool,
    pub yesterday_count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct AtRisk {
    /// The run is worth something today and today has no log in it: miss it and the
    /// multiplier falls back to 1.0 at UTC midnight.
    pub at_risk: bool,
    pub secs_left: i64,
    pub multiplier: f64,
    pub streak: i64,
    pub next_tier_in: Option<i64>,
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
    /// Every log ever made, never decayed. A record, not a rankable score:
    /// nothing sorts by it and no multiplier applies to it.
    pub lifetime_total: i64,
    /// True when the user has logs but none that still count, i.e. 30+ days of
    /// silence and the score has been wiped. Drives the UI hint.
    pub decayed: bool,
    /// Whole days of silence left before the score is wiped, i.e.
    /// `30 - days since the newest log`, floored at 0. `None` for a user who has
    /// never logged: there is nothing to lose yet. 0 means the era has already
    /// ended (same state as `decayed`, told from the countdown's direction so the
    /// dashboard can say "log to restore it" rather than "0 days left").
    pub days_until_decay: Option<i64>,
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
    /// True when the row was filed late for the day it describes. Labelled in the
    /// caller's own history and on the feed, because a backdated row that looks
    /// ordinary is a small lie the site does not need to tell.
    #[serde(default)]
    pub catch_up: bool,
}

/// A log as the PUBLIC feed shows it: no note.
///
/// A separate type rather than a `skip_serializing_if` on `FeedItem`, because
/// the difference is a privacy rule and not a serialisation preference. The
/// stats endpoint hands a user their own logs (notes included, authenticated
/// and self-scoped); `activity_feed()` no longer returns a note at all, so this
/// type cannot grow one back by accident.
#[derive(Debug, Serialize, Clone)]
pub struct PublicFeedItem {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    /// 'habit' or 'denial' — the feed carries both since 2026-10-29; a denial is
    /// the 10x event and hiding it from the square was a hole, not a policy.
    pub kind: String,
    /// The confession. Public by default (owner, 2026-10-29) with a per-profile
    /// opt-out; the SQL decides, so this layer cannot leak what the reader's setting
    /// excludes.
    pub note: Option<String>,
    pub logged_at: String,
    /// Filed late for the day it describes — the wall shows it, because the wall
    /// claims to be what happened and a backdated row is part of the record.
    pub catch_up: bool,
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
    /// Legacy mirror of `wld`; the 💧 badge on both board pages reads it.
    /// `points` / `waste_count` / `affirmation_count` were dropped with the
    /// 10x weighting they encoded — nothing in the frontend read them.
    pub denial_count: i64,
    /// Never-decaying all-time log count. Shown as a record alongside a score
    /// that the 30-day decay rule can reset.
    pub lifetime_total: i64,
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
    pub items: Vec<PublicFeedItem>,
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
    /// Whether this member's confessions appear in the public feed. True by default
    /// (owner, 2026-10-29); the switch lives in settings.
    pub logs_public: bool,
    /// Whether this member's reported durations feed the public averages (0021).
    /// False by default — the only private-by-default switch on the site.
    pub share_time_to_cum: bool,
    /// The computed trust level (0–4). Part of the profile's numbers block, because a
    /// profile that shows only counts cannot be read: how much can be trusted of this
    /// person is the first thing a stranger wants to know.
    pub trust_level: i32,
    /// Peer-verified cage time, from the same function the cage page reads.
    pub verified_cage_secs: i64,
    /// How many durations this member has reported — always shown.
    pub time_to_cum_samples: i64,
    /// Their average, and only when they share it (0021). `None` is "not shared", which
    /// the page prints as words rather than as a zero.
    pub time_to_cum_avg_secs: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub social_url: Option<String>,
    pub logs_public: Option<bool>,
    pub share_time_to_cum: Option<bool>,
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
///
/// The date is taken in UTC, not in the caller's offset: `now.date()` is the date
/// on the CALLER's clock, so a +02 caller at 01:30 on the 15th (23:30 UTC on the
/// 14th) would get the 16th's midnight — a day late, and a disagreement with the
/// SQL scanners, which bucket strictly by UTC. Every caller passes `now_utc()`
/// today, which is why the difference was latent; the offset conversion makes the
/// promise in this doc true for any input.
pub fn next_utc_midnight(now: OffsetDateTime) -> OffsetDateTime {
    (now.to_offset(time::UtcOffset::UTC).date() + time::Duration::days(1))
        .midnight()
        .assume_utc()
}

/// Enforce the daily (UTC) cadence. `pub` so integration tests can assert the
/// rule directly instead of only inferring it from an HTTP response.
///
/// `wlw_daily_limit` is the operator's `WLW_DAILY_LIMIT` (default 5): loads
/// wasted are a daily BUDGET, denials stay one per day, affirmations keep their
/// own `DAILY_LIMIT`.
pub async fn check_rate_limits(
    pool: &PgPool,
    user_id: uuid::Uuid,
    kind: LogKind,
    wlw_daily_limit: i64,
) -> ApiResult<()> {
    check_rate_limits_on(
        pool,
        user_id,
        kind,
        wlw_daily_limit,
        OffsetDateTime::now_utc().date(),
    )
    .await
}

/// The same rule, judged against a named UTC date.
///
/// `for_date` exists for the catch-up window: a log filed for yesterday answers to
/// yesterday's budget and yesterday's exclusivity rule, not today's.
pub async fn check_rate_limits_on(
    pool: &PgPool,
    user_id: uuid::Uuid,
    kind: LogKind,
    wlw_daily_limit: i64,
    for_date: time::Date,
) -> ApiResult<()> {
    let now = OffsetDateTime::now_utc();
    // The day being written to, not necessarily today: a catch-up log answers to
    // the budget and the exclusivity rule of the day it describes. Judging a
    // backdated waste by today's counter would let a member file the sixth waste
    // of yesterday through the catch-up door.
    let today_start = for_date.midnight().assume_utc();

    // Counts for the current UTC day, per kind. `habits_today` is the only one
    // the exclusivity rule reads now — see the direction note below.
    let (habits_today, kind_today): (i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = $3       and log_date = $2)",
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

    // WLWs and WLDs are exclusive in ONE direction, and the direction is the
    // whole rule (owner, 2026-10-29):
    //
    //   waste  -> denial : BLOCKED. A whiteboi who cums cannot claim he was
    //                       denied. The denial day is spent the moment a load is
    //                       wasted, however much of the waste budget is left.
    //   denial -> waste : ALLOWED. A reported denial is not a promise, and a load
    //                      wasted after a stand is exactly the failure this site
    //                      exists to count. Refusing it hid the relapse and left
    //                      the waste counter at zero for the rest of the day.
    //
    // The waste budget is untouched by a denial either way: five is five. The
    // Devotion Index still treats the two as competing streaks
    // (`exclusive_streak` follows the most recent kind), so switching kind is
    // never a free win — it only costs the active streak.
    if kind == LogKind::Denial && habits_today >= 1 {
        return Err(ApiError::too_many_requests(
            "You already recorded a load wasted today. WLWs and WLDs are one way: a whiteboi who cums cannot claim a denial. Come back tomorrow.",
        ));
    }

    // The day's budget for this kind. This is the primary rule: it is what
    // grows a streak (waste) or what a stand is measured in (denial).
    let budget = daily_budget(kind, wlw_daily_limit);
    if kind_today >= budget {
        let msg = match kind {
            // Names the clock, not just the verdict: a 429 with no time in it
            // reads as "the button is broken".
            LogKind::Habit => format!(
                "Daily limit reached ({budget} loads wasted per UTC day). You can waste again at {next_str}."
            ),
            LogKind::Affirmation => format!(
                "Daily limit reached ({budget} per day). Come back tomorrow."
            ),
            LogKind::Denial => format!(
                "Already recorded a {} today. Submit once a day (UTC) to grow your streak — you can again at {next_str}.",
                kind.as_str()
            ),
        };
        return Err(ApiError::too_many_requests(msg));
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

    // Two things can spend the denial's day: a denial already reported, or a
    // load wasted today (the one-way rule — waste blocks the denial, not the
    // reverse). Either way the next opportunity is the next UTC midnight.
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
    // GS-1: at the MAU cap a guest keeps reading but stops writing (§1).
    crate::guest::guard_write(&state, &user).await?;
    let note = body.note.unwrap_or_default();
    let note = note.trim();
    if note.chars().count() > 140 {
        return Err(ApiError::bad_request("Note must be 140 characters or fewer"));
    }
    let kind = parse_kind(body.kind.as_deref());
    let ttc = match body.time_to_cum_secs {
        None => None,
        Some(v) if (0..=86_400).contains(&v) => Some(v as i32),
        Some(_) => {
            return Err(ApiError::bad_request(
                "time_to_cum_secs must be between 0 and 86400",
            ))
        }
    };

    // ---- Which day is this? ------------------------------------------------
    // `today` is the ordinary path. `yesterday` is the catch-up window, and it is
    // deliberately narrow: it fills a GAP, so it is refused when that day already
    // has any log at all. A member who logged once yesterday cannot add a second
    // one from today; that would be a claim with nothing behind it, and the budget
    // is per day, not per mood.
    let today = time::OffsetDateTime::now_utc().date();
    let (target_date, is_catch_up) = match body.day.as_deref().unwrap_or("today") {
        "today" | "" => (today, false),
        "yesterday" => (
            today.previous_day().ok_or_else(|| {
                ApiError::bad_request("There is no yesterday before the first day")
            })?,
            true,
        ),
        other => {
            return Err(ApiError::bad_request(format!(
                "day must be \"today\" or \"yesterday\", not {other:?}"
            )))
        }
    };

    if is_catch_up {
        let already: i64 = sqlx::query_scalar(
            "select count(*) from habit_logs where user_id = $1 and log_date = $2",
        )
        .bind(user.id)
        .bind(target_date)
        .fetch_one(&state.pool)
        .await?;
        if already > 0 {
            return Err(ApiError::conflict(
                "That day already has a log. A catch-up fills a gap — it cannot add to a day you kept.",
            ));
        }
    }

    check_rate_limits_on(
        &state.pool,
        user.id,
        kind,
        state.cfg.wlw_daily_limit,
        target_date,
    )
    .await?;

    // A catch-up is stamped at midday of the day it describes: the real time is
    // unknown, and midday keeps it unambiguously inside that UTC date.
    let logged_at: Option<OffsetDateTime> = if is_catch_up {
        Some(target_date.midnight().assume_utc() + time::Duration::hours(12))
    } else {
        None
    };

    let insert = sqlx::query(
        "insert into habit_logs
            (user_id, note, kind, time_to_cum_secs, logged_at, is_catch_up, catch_up_on)
         values ($1, $2, $3, $4, coalesce($5, now()), $6, $7)",
    )
    .bind(user.id)
    .bind(note)
    .bind(kind.as_str())
    .bind(ttc)
    .bind(logged_at)
    .bind(is_catch_up)
    .bind(if is_catch_up { Some(today) } else { None })
    .execute(&state.pool)
    .await;

    if let Err(e) = insert {
        // The one-per-filing-day index is the rule; a lost race must say so rather
        // than surface as a 500.
        if matches!(&e, sqlx::Error::Database(db) if db.constraint()
            == Some("habit_logs_one_catch_up_per_filing_day"))
        {
            return Err(ApiError::conflict(
                "You already filed a catch-up log today. One a day is the rule.",
            ));
        }
        return Err(ApiError::from(e));
    }

    let stats = compute_stats(&state.pool, user.id, kind, state.cfg.wlw_daily_limit).await?;
    Ok((StatusCode::CREATED, Json(LogResponse { stats })))
}

/// How long after writing a log its author may still edit or remove it.
///
/// Fifteen minutes is the undo window, and it exists because the landing page now
/// logs on a single click: a rule that makes a mis-click permanent would be a good
/// reason not to offer the single click at all.
pub const LOG_EDIT_WINDOW_SECS: i64 = 900;

#[derive(Debug, Deserialize)]
pub struct UpdateLogRequest {
    /// Sets the duration, or clears it when null. PATCH is a set, not a merge: the
    /// only field it carries is this one.
    #[serde(default)]
    pub time_to_cum_secs: Option<i64>,
}

/// Load a log the caller owns and is still allowed to touch, or fail with the
/// reason the UI needs to print.
async fn owned_recent_log(
    pool: &PgPool,
    user_id: uuid::Uuid,
    log_id: i64,
) -> ApiResult<(uuid::Uuid, OffsetDateTime)> {
    // `created_at`, not `logged_at`: a catch-up describes yesterday but was written
    // now, and the window is about the writing.
    let row: Option<(uuid::Uuid, OffsetDateTime)> =
        sqlx::query_as("select user_id, created_at from habit_logs where id = $1")
            .bind(log_id)
            .fetch_optional(pool)
            .await?;
    let Some((owner, logged_at)) = row else {
        return Err(ApiError::not_found("No such log"));
    };
    if owner != user_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "That log is not yours",
        ));
    }
    let age = (OffsetDateTime::now_utc() - logged_at).whole_seconds();
    if age > LOG_EDIT_WINDOW_SECS {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            format!(
                "Logs can only be changed in the first {} minutes",
                LOG_EDIT_WINDOW_SECS / 60
            ),
        ));
    }
    Ok((owner, logged_at))
}

/// PATCH /api/logs/{id} — the post-click half of the one-click flow: the waste is
/// already recorded, this adds the number without a second form in the way.
pub async fn update_log(
    State(state): State<AppState>,
    session: Session,
    Path(log_id): Path<i64>,
    Json(body): Json<UpdateLogRequest>,
) -> ApiResult<Json<LogResponse>> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;
    owned_recent_log(&state.pool, user.id, log_id).await?;

    let ttc = match body.time_to_cum_secs {
        None => None,
        Some(v) if (0..=86_400).contains(&v) => Some(v as i32),
        Some(_) => {
            return Err(ApiError::bad_request(
                "time_to_cum_secs must be between 0 and 86400",
            ))
        }
    };
    sqlx::query("update habit_logs set time_to_cum_secs = $2 where id = $1")
        .bind(log_id)
        .bind(ttc)
        .execute(&state.pool)
        .await?;

    let stats = compute_stats(&state.pool, user.id, LogKind::Habit, state.cfg.wlw_daily_limit).await?;
    Ok(Json(LogResponse { stats }))
}

/// DELETE /api/logs/{id} — undo, inside the same window.
///
/// The row is really deleted, not hidden: undo means "this did not happen", and a
/// tombstone would keep it in the daily budget the user is trying to get back.
pub async fn delete_log(
    State(state): State<AppState>,
    session: Session,
    Path(log_id): Path<i64>,
) -> ApiResult<StatusCode> {
    let user = require_user(&session).await?;
    crate::guest::guard_write(&state, &user).await?;
    owned_recent_log(&state.pool, user.id, log_id).await?;
    sqlx::query("delete from habit_logs where id = $1")
        .bind(log_id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_stats(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<StatsResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Habit, state.cfg.wlw_daily_limit).await?;
    Ok(Json(stats))
}

/// Drill stats — affirmation-specific counters + rate-limit state.
pub async fn get_drill(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<DrillResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Affirmation, state.cfg.wlw_daily_limit).await?;
    Ok(Json(DrillResponse { stats }))
}

/// Denial page — denial-kind stats + chastity lock state.
pub async fn get_denial(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<DenialResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id, LogKind::Denial, state.cfg.wlw_daily_limit).await?;
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
    // GS-1: at the MAU cap a guest keeps reading but stops writing (§1).
    crate::guest::guard_write(&state, &user).await?;
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
    // GS-1: at the MAU cap a guest keeps reading but stops writing (§1).
    crate::guest::guard_write(&state, &user).await?;
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

pub async fn lock_info(pool: &PgPool, user_id: uuid::Uuid) -> ApiResult<LockInfoResponse> {
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

async fn compute_stats(
    pool: &PgPool,
    user_id: uuid::Uuid,
    kind: LogKind,
    wlw_daily_limit: i64,
) -> ApiResult<StatsResponse> {
    let now = OffsetDateTime::now_utc();
    let hour_ago = now - time::Duration::minutes(60);
    let today_start = now.date().midnight().assume_utc();
    let week_start = week_start_utc(now);

    // Yesterday, for the catch-up window: how many logs that day holds (any kind,
    // because a catch-up fills a gap rather than topping up a day), and whether
    // one was already filed today.
    let (yesterday_count, catch_up_filed): (i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs
             where user_id = $1 and log_date = (now() at time zone 'utc')::date - 1),
           (select count(*) from habit_logs
             where user_id = $1 and is_catch_up
               and catch_up_on = (now() at time zone 'utc')::date)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

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

    // Newest-WRITTEN first, not newest-dated: a catch-up filed a minute ago
    // describes yesterday and must still be at the top, or the 15-minute undo
    // would be attached to a different row than the one just created.
    let recent: Vec<FeedItem> = sqlx::query_as::<_, (i64, uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime, Option<String>, bool)>(
        "select id, user_id, username, display_name, avatar_url, logged_at, note, catch_up
         from my_recent_logs($1, $2::text, 20)",
    )
    .bind(user_id)
    .bind(kind.as_str())
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, uid, uname, dname, av, ts, note, catch_up)| FeedItem {
        id,
        user_id: uid,
        username: uname,
        display_name: dname,
        avatar_url: av,
        logged_at: ts.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        note,
        catch_up,
    })
    .collect();

    let (habits_today, kind_today): (i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and kind = 'habit' and log_date = $2),
           (select count(*) from habit_logs where user_id = $1 and kind = $3       and log_date = $2)",
    )
    .bind(user_id)
    .bind(today_start.date())
    .bind(kind.as_str())
    .fetch_one(pool)
    .await?;

    // The exclusivity rule is one-way (see check_rate_limits), and this flag has
    // to mirror it exactly or the UI disables a button the server would accept:
    // a waste today spends the DENIAL's day, but a denial today does not touch
    // the waste budget. So a habit is never exclusive-blocked.
    let blocked_by_exclusivity = kind == LogKind::Denial && habits_today >= 1;

    // Mirrors check_rate_limits exactly, off the same daily_budget(): loads
    // wasted are a budget (WLW_DAILY_LIMIT, default 5), a denial is one per day,
    // affirmations keep their own DAILY_LIMIT.
    let budget = daily_budget(kind, wlw_daily_limit);
    let remaining_today = (budget - kind_today).max(0);
    let can_log = remaining_today > 0 && !blocked_by_exclusivity;
    // Seconds until the next UTC midnight. The UI ticks this down client-side,
    // so the countdown moves without a reload and flips can_log at zero.
    let next_in_secs = if can_log {
        0
    } else {
        (next_utc_midnight(now) - now).whole_seconds().max(0)
    };
    // The same clock, WITHOUT the `can_log` gate: the budget refills at UTC
    // midnight whether or not the user is allowed to log this second.
    let secs_until_reset = (next_utc_midnight(now) - now).whole_seconds().max(0);

    // Whiteboi Devotion Index for this user, under the 30-day decay rule: logs
    // older than 30 days no longer count, so an inactive user reads 0 here while
    // `lifetime_total` preserves their history.
    let (wlw, wld, game_bonus, mult, penalty, score, lifetime_total, decayed, days_until_decay): (
        i64, i64, i64, f64, f64, f64, i64, bool, Option<i64>,
    ) = sqlx::query_as(
        "select wlw::bigint, wld::bigint, game_bonus::bigint,
                multiplier::float8, racism_penalty::float8, score::float8,
                lifetime_total::bigint, decayed, days_until_decay::bigint
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

    // Per-kind streaks, so the one-way exclusivity rule is visible instead of
    // inferred from a single combined number.
    let (habit_streak, denial_streak, affirmation_streak): (i64, i64, i64) = sqlx::query_as(
        "select
           (select current_streak from user_streak_kind($1, 'habit'))::bigint,
           (select current_streak from user_streak_kind($1, 'denial'))::bigint,
           (select current_streak from user_streak_kind($1, 'affirmation'))::bigint",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let (participation_streak,): (i64,) =
        sqlx::query_as("select current_streak::bigint from user_participation_streak($1)")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    let logged_today: bool = sqlx::query_scalar(
        "select exists(select 1 from habit_logs where user_id = $1 and log_date = (now() at time zone 'utc')::date)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let (ttc_samples, ttc_avg, ttc_median, ttc_min, ttc_max, ttc_last): (
        i64,
        f64,
        f64,
        i32,
        i32,
        Option<i32>,
    ) = sqlx::query_as(
        "select samples::bigint, avg_secs::float8, median_secs::float8, min_secs, max_secs, last_secs
         from time_to_cum_stats($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // The tiers `devotion_multiplier` actually pays, in one place: a UI that invents
    // its own thresholds is a UI that promises a multiplier the SQL will not give.
    const TIERS: [(i64, f64); 5] = [(7, 1.25), (14, 1.5), (30, 2.0), (60, 2.5), (100, 3.0)];
    let milestones: Vec<MilestoneTier> = TIERS
        .iter()
        .map(|(days, multiplier)| MilestoneTier {
            days: *days,
            multiplier: *multiplier,
            reached: participation_streak >= *days,
        })
        .collect();
    let secs_left = (next_utc_midnight(now) - now).whole_seconds().max(0);
    let at_risk = AtRisk {
        // At risk means: the run is already worth a multiplier, and today is one of
        // the days that would have to hold it. A member who has logged today is
        // safe until tomorrow; a member with no multiplier has nothing to lose.
        at_risk: mult > 1.0 && participation_streak > 0 && !logged_today,
        secs_left,
        multiplier: mult,
        streak: participation_streak,
        next_tier_in: TIERS
            .iter()
            .find(|(days, _)| *days > participation_streak)
            .map(|(days, _)| *days - participation_streak),
    };

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
        daily_limit: budget,
        remaining_today,
        next_allowed_in_secs: next_in_secs,
        secs_until_reset,
        blocked_by_exclusivity,
        catch_up: CatchUpState {
            available: yesterday_count == 0 && catch_up_filed == 0,
            filed_today: catch_up_filed > 0,
            yesterday_count,
        },
        time_to_cum: TimeToCumMine {
            samples: ttc_samples,
            avg_secs: ttc_avg,
            median_secs: ttc_median,
            min_secs: ttc_min as i64,
            max_secs: ttc_max as i64,
            last_secs: ttc_last.map(|v| v as i64),
        },
        kind_streaks: KindStreaks {
            habit: habit_streak,
            denial: denial_streak,
            affirmation: affirmation_streak,
        },
        milestones,
        at_risk,
        devotion: DevotionIndexResponse {
            wlw,
            wld,
            game_bonus,
            multiplier: mult,
            racism_penalty: penalty,
            score,
            lifetime_total,
            decayed,
            days_until_decay,
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
    // Order must match COLS below exactly:
    //   identity | score, wlw, wld, game_bonus, multiplier | streak, kind, last | penalty
    type Row = (
        uuid::Uuid, String, Option<String>, Option<String>,   // identity
        f64, i64, i64, i64, f64,                              // score + components
        i64, String, Option<OffsetDateTime>,                 // streak, kind, last
        f64,                                                 // racism_penalty (always 0)
        i64,                                                 // lifetime_total
    );
    // Column list is explicit rather than `select *`: the leaderboard functions
    // return `score` and `multiplier` as SQL `numeric` (exact arithmetic, and
    // `round(...,2)` needs it), which sqlx cannot decode into f64. `select *`
    // therefore 500s at runtime. Cast at the SQL boundary instead of adding a
    // decimal dependency to the crate.
    //
    // The board function does NOT return racism_penalty (only devotion_index
    // does), so it is filled with the literal 0 it always evaluates to.
    let rows: Vec<Row> = {
        // The board function does not return racism_penalty, so the literal 0
        // it always evaluates to is supplied here. It MUST sit in the position
        // matching the Row tuple, not be appended: COLS already ends with
        // lifetime_total, and a trailing literal shifts every later column.
        const COLS: &str = "user_id, username, display_name, avatar_url, \
             score::float8, wlw::bigint, wld::bigint, game_bonus::bigint, \
             multiplier::float8, streak::bigint, active_kind, last_log_at, \
             0::float8, lifetime_total::bigint";
        let q = |fn_name: &str| format!("select {COLS} from {fn_name}");
        match period {
            "daily" => sqlx::query_as(&q("daily_leaderboard_weighted(50)"))
                .fetch_all(&state.pool)
                .await?,
            "weekly" => sqlx::query_as(&q("weekly_leaderboard_weighted(50)"))
                .fetch_all(&state.pool)
                .await?,
            "monthly" => sqlx::query_as(&q("monthly_leaderboard_weighted(50)"))
                .fetch_all(&state.pool)
                .await?,
            // Default when no period is given: the monthly board.
            _ => sqlx::query_as(&q("alltime_leaderboard_weighted(50)"))
                .fetch_all(&state.pool)
                .await?,
        }
    };
    let entries = rows
        .into_iter()
        .enumerate()
        .map(
            |(i, (uid, uname, dname, av, score, wlw, wld, game_bonus, mult, streak, kind, last, penalty, lifetime_total))| {
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
                // Legacy mirror of wld, kept because both board pages render a
                // 💧 badge from it. The authoritative field is `wld`.
                denial_count: wld,
                lifetime_total,
                last_log_at: last
                    .map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()),
            }
        },
        )
        .collect();
    Ok(Json(LeaderboardResponse { period: period.into(), entries }))
}

#[derive(Debug, Serialize)]
pub struct KpiTotalsResponse {
    pub total_wasted: i64,
    pub total_denied: i64,
    pub total_affirmations: i64,
    pub total_users: i64,
    pub active_24h: i64,
    pub new_7d: i64,
    pub currently_locked: i64,
    pub total_lock_hours: f64,
    pub denial_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct KpiTrendPoint {
    pub day: String,
    pub wasted: i64,
    pub denied: i64,
    pub affirmations: i64,
}

#[derive(Debug, Serialize)]
pub struct KpiResponse {
    pub totals: KpiTotalsResponse,
    pub trend: Vec<KpiTrendPoint>,
    pub top_weekly: Vec<LeaderboardEntry>,
}

/// GET /api/kpi — public community KPIs. No auth, matching the landing page's
/// public counters: "the community is alive" is the feature, and a gate would
/// defeat it.
///
/// The plan for this endpoint said the mapper is a thin shape like
/// get_leaderboard and needs no test of its own. That is true of the happy path
/// and false of the part that actually breaks: the top-weekly board. It is read
/// with the SAME explicit column list as get_leaderboard, not `select *`,
/// because the board functions return `score` and `multiplier` as SQL numeric
/// and sqlx cannot decode those into f64 -- `select *` here would 500 at
/// runtime, on every request, for a page whose whole job is to load. The
/// literal 0 for racism_penalty must sit in the tuple position the Row type
/// expects, not be appended, or every later column shifts.
pub async fn get_kpi(State(state): State<AppState>) -> ApiResult<Json<KpiResponse>> {
    let (
        total_wasted,
        total_denied,
        total_affirmations,
        total_users,
        active_24h,
        new_7d,
        currently_locked,
        total_lock_hours,
        denial_rate,
    ): (i64, i64, i64, i64, i64, i64, i64, f64, f64) = sqlx::query_as("select * from kpi_totals()")
        .fetch_one(&state.pool)
        .await?;

    // Fixed at 30 days, matching the plan's decision that the trend window is a
    // coarse daily view. Not a query parameter: a public endpoint that lets a
    // caller ask for 10_000 days is a way to run an expensive scan on demand.
    let trend_rows: Vec<(String, i64, i64, i64)> =
        sqlx::query_as("select * from kpi_trend(30)")
            .fetch_all(&state.pool)
            .await?;
    let trend = trend_rows
        .into_iter()
        .map(|(day, wasted, denied, affirmations)| KpiTrendPoint {
            day,
            wasted,
            denied,
            affirmations,
        })
        .collect();

    // Same shape and same casts as get_leaderboard's "weekly" branch. Rank is
    // assigned by position, so the ORDER BY inside the function is what decides
    // the numbering.
    type WeeklyRow = (
        uuid::Uuid, String, Option<String>, Option<String>,   // identity
        f64, i64, i64, i64, f64,                              // score + components
        i64, String, Option<OffsetDateTime>,                 // streak, kind, last
        f64,                                                 // racism_penalty (always 0)
        i64,                                                 // lifetime_total
    );
    const WEEKLY_COLS: &str = "user_id, username, display_name, avatar_url, \
         score::float8, wlw::bigint, wld::bigint, game_bonus::bigint, \
         multiplier::float8, streak::bigint, active_kind, last_log_at, \
         0::float8, lifetime_total::bigint";
    let weekly: Vec<WeeklyRow> =
        sqlx::query_as(&format!("select {WEEKLY_COLS} from weekly_leaderboard_weighted(5)"))
            .fetch_all(&state.pool)
            .await?;
    let top_weekly = weekly
        .into_iter()
        .enumerate()
        .map(|(i, (uid, uname, dname, av, score, wlw, wld, game_bonus, mult, streak, kind, last, penalty, lifetime_total))| {
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
                denial_count: wld,
                lifetime_total,
                last_log_at: last.map(|t| {
                    t.format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default()
                }),
            }
        })
        .collect();

    Ok(Json(KpiResponse {
        totals: KpiTotalsResponse {
            total_wasted,
            total_denied,
            total_affirmations,
            total_users,
            active_24h,
            new_7d,
            currently_locked,
            total_lock_hours,
            denial_rate,
        },
        trend,
        top_weekly,
    }))
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

/// GET /api/today — what the community has done today (UTC).
///
/// Public and unauthenticated, like /api/kpi: every number here is already
/// derivable from that page, and the point of the line is that a solitary
/// practice has a crowd visible next to it. The counts come from `today_counts()`
/// rather than being recomputed here, so this endpoint and the database agree on
/// what "today" means by construction (the generated `log_date` column).
#[derive(Debug, Serialize)]
pub struct TodayResponse {
    pub denials_today: i64,
    pub wastes_today: i64,
    pub loggers_today: i64,
}

pub async fn get_today(State(state): State<AppState>) -> ApiResult<Json<TodayResponse>> {
    let (denials_today, wastes_today, loggers_today): (i64, i64, i64) =
        sqlx::query_as("select denials_today::bigint, wastes_today::bigint, loggers_today::bigint from today_counts()")
            .fetch_one(&state.pool)
            .await?;
    Ok(Json(TodayResponse {
        denials_today,
        wastes_today,
        loggers_today,
    }))
}

// ---- Health ----

/// A health check that can fail.
///
/// Before this existed, `GET /health` returned 200 on a broken database: nothing
/// matched the path, so the SPA fallback served index.html and the status was a
/// cheerful 200 describing nothing. A check that cannot fail is not a check, so
/// this one round-trips a query and reports the failure as 503.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let db_up = sqlx::query_scalar::<_, i32>("select 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();
    let status = if db_up { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (
        status,
        Json(HealthResponse {
            status: if db_up { "ok" } else { "degraded" },
            database: if db_up { "ok" } else { "unreachable" },
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
}

// ---- Feed ----

pub async fn get_feed(
    State(state): State<AppState>,
    Query(params): Query<FeedParams>,
) -> ApiResult<Json<FeedResponse>> {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let cursor = params.cursor.unwrap_or(0);
    // Named columns, not `select *`: the function's OUT list is what the public
    // sees, and this query is the second place that decides. Both agree that a note
    // is public when its author left `logs_public` on (2026-10-29) — the decision
    // lives in `activity_feed`, so this layer cannot publish past the setting.
    let rows: Vec<(
        i64,
        uuid::Uuid,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        OffsetDateTime,
        bool,
    )> = sqlx::query_as(
        "select id, user_id, username, display_name, avatar_url, kind, note, logged_at, catch_up
             from activity_feed($1, $2::int)",
    )
    .bind(cursor)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<PublicFeedItem> = rows
        .into_iter()
        .map(|(id, uid, uname, dname, av, kind, note, ts, catch_up)| PublicFeedItem {
            id,
            user_id: uid,
            username: uname,
            display_name: dname,
            avatar_url: av,
            kind,
            note,
            logged_at: ts.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            catch_up,
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
    let row: Option<(uuid::Uuid, String, Option<String>, Option<String>, Option<String>, OffsetDateTime, bool, bool)> =
        sqlx::query_as(
            "select id, username, display_name, avatar_url, social_url, created_at, logs_public, share_time_to_cum
               from profiles where username = $1",
        )
        .bind(&username)
        .fetch_optional(&state.pool)
        .await?;
    let Some((id, uname, dname, av, social_url, created, logs_public, share_time_to_cum)) = row else {
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

    // The numbers block: one round trip, three functions the rest of the site already
    // trusts. `time_to_cum_stats` is the member's own figure, so it is always counted;
    // whether it may be PUBLISHED is `share_time_to_cum`, and that is the only branch here.
    let (trust_level, verified_cage_secs, ttc_samples, ttc_avg): (i32, i64, i64, f64) =
        sqlx::query_as(
            "select (select level from public.trust_level($1)) as level,
                    (select verified_secs::bigint from public.cage_check_hours($1)) as verified,
                    (select samples::bigint from public.time_to_cum_stats($1)) as ttc_samples,
                    (select avg_secs::float8 from public.time_to_cum_stats($1)) as ttc_avg",
        )
        .bind(id)
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
        logs_public,
        share_time_to_cum,
        trust_level,
        verified_cage_secs,
        time_to_cum_samples: ttc_samples,
        time_to_cum_avg_secs: if share_time_to_cum && ttc_samples > 0 {
            Some(ttc_avg)
        } else {
            None
        },
    }))
}

pub async fn update_profile(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<Json<ProfileResponse>> {
    let user = require_user(&session).await?;
    // GS-1: at the MAU cap a guest keeps reading but stops writing (§1).
    crate::guest::guard_write(&state, &user).await?;

    let mut username = user.username.clone();
    let mut display_name = user.display_name.clone();
    let mut avatar_url = user.avatar_url.clone();
    let mut social_url = user.social_url.clone();
    // Publication is a setting, not a one-way door: whatever it is now, the client
    // may flip it in the same request that edits anything else.
    let mut logs_public: bool = sqlx::query_scalar("select logs_public from profiles where id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    if let Some(p) = body.logs_public {
        logs_public = p;
    }
    // Same shape for the durations: a setting the client may flip in the same request it
    // edits anything else, and turning it OFF takes effect on the next read of the averages
    // because the filter is inside the SQL function rather than in a handler.
    let mut share_time_to_cum: bool =
        sqlx::query_scalar("select share_time_to_cum from profiles where id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    if let Some(v) = body.share_time_to_cum {
        share_time_to_cum = v;
    }

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

    let updated = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, Option<String>, Option<String>, OffsetDateTime, bool, bool)>(
        "update profiles set username = $1, display_name = $2, avatar_url = $3, social_url = $4,
                logs_public = $5, share_time_to_cum = $6, updated_at = now()
         where id = $7
         returning id, username, display_name, avatar_url, social_url, created_at, logs_public,
                   share_time_to_cum",
    )
    .bind(&username)
    .bind(&display_name)
    .bind(&avatar_url)
    .bind(&social_url)
    .bind(logs_public)
    .bind(share_time_to_cum)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, uname, dname, av, social_url, created, logs_public, share_time_to_cum)) = updated else {
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
        is_guest: user.is_guest,
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

    let (trust_level, verified_cage_secs, ttc_samples, ttc_avg): (i32, i64, i64, f64) =
        sqlx::query_as(
            "select (select level from public.trust_level($1)) as level,
                    (select verified_secs::bigint from public.cage_check_hours($1)) as verified,
                    (select samples::bigint from public.time_to_cum_stats($1)) as ttc_samples,
                    (select avg_secs::float8 from public.time_to_cum_stats($1)) as ttc_avg",
        )
        .bind(id)
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
        logs_public,
        share_time_to_cum,
        trust_level,
        verified_cage_secs,
        time_to_cum_samples: ttc_samples,
        time_to_cum_avg_secs: if share_time_to_cum && ttc_samples > 0 {
            Some(ttc_avg)
        } else {
            None
        },
    }))
}

fn valid_username(u: &str) -> bool {
    !u.is_empty()
        && u.chars().count() <= 30
        && u.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}
