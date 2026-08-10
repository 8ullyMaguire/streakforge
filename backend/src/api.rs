// API handlers — logging (rate-limited), stats, leaderboards, feed, profiles.
// Rate limits (server-enforced):
//   - max 1 log per 60 minutes per user
//   - max 5 logs per UTC calendar day per user

use crate::auth::SessionUser;
use crate::error::{ApiError, ApiResult};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use time::OffsetDateTime;
use tower_sessions::Session;

pub const HOURLY_LIMIT: i64 = 1;
pub const DAILY_LIMIT: i64 = 5;

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
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub stats: StatsResponse,
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
    pub count: i64,
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
    pub count: i64,
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
}

// ---- Rate limiting ----

async fn check_rate_limits(pool: &PgPool, user_id: uuid::Uuid) -> ApiResult<()> {
    let now = OffsetDateTime::now_utc();
    let hour_ago = now - time::Duration::minutes(60);
    let today_start = now.date();
    let today_start_dt = today_start.midnight().assume_utc();

    let (last_60m, today_count): (i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1 and logged_at >= $3)",
    )
    .bind(user_id)
    .bind(hour_ago)
    .bind(today_start_dt)
    .fetch_one(pool)
    .await?;

    if last_60m >= HOURLY_LIMIT {
        // when can they log again? next full hour boundary
        let next = hour_ago + time::Duration::minutes(60);
        let msg = format!(
            "Hourly limit reached (1 per hour). You can log again at {}.",
            next.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
        );
        return Err(ApiError::too_many_requests(msg));
    }
    if today_count >= DAILY_LIMIT {
        return Err(ApiError::too_many_requests(format!(
            "Daily limit reached ({} per day). Come back tomorrow.",
            DAILY_LIMIT
        )));
    }
    Ok(())
}

fn next_allowed_at(user: &SessionUser, today_count: i64, last_60m: i64) -> Option<String> {
    if last_60m >= HOURLY_LIMIT {
        let next = OffsetDateTime::now_utc() + time::Duration::minutes(60);
        Some(
            next.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        )
    } else if today_count >= DAILY_LIMIT {
        let next = (OffsetDateTime::now_utc().date() + time::Duration::days(1))
            .midnight()
            .assume_utc();
        Some(
            next.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        )
    } else {
        None
    }
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

    check_rate_limits(&state.pool, user.id).await?;

    sqlx::query("insert into habit_logs (user_id, note) values ($1, $2)")
        .bind(user.id)
        .bind(note)
        .execute(&state.pool)
        .await?;

    let stats = compute_stats(&state.pool, user.id).await?;
    Ok((StatusCode::CREATED, Json(LogResponse { stats })))
}

pub async fn get_stats(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Json<StatsResponse>> {
    let user = require_user(&session).await?;
    let stats = compute_stats(&state.pool, user.id).await?;
    Ok(Json(stats))
}

async fn compute_stats(pool: &PgPool, user_id: uuid::Uuid) -> ApiResult<StatsResponse> {
    let now = OffsetDateTime::now_utc();
    let hour_ago = now - time::Duration::minutes(60);
    let today_start = now.date().midnight().assume_utc();
    let week_start = week_start_utc(now);

    let (today_count, week_count, alltime_count): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1)",
    )
    .bind(user_id)
    .bind(week_start)
    .fetch_one(pool)
    .await?;

    let last_60m: i64 = sqlx::query_scalar(
        "select count(*) from habit_logs where user_id = $1 and logged_at >= $2",
    )
    .bind(user_id)
    .bind(hour_ago)
    .fetch_one(pool)
    .await?;

    let (current_streak, longest_streak): (i64, i64) = sqlx::query_as(
        "select current_streak::bigint, longest_streak::bigint from user_streak($1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let recent: Vec<FeedItem> = sqlx::query_as::<_, (i64, uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime, Option<String>)>(
        "select l.id, l.user_id, p.username, p.display_name, p.avatar_url, l.logged_at, l.note
         from habit_logs l join profiles p on p.id = l.user_id
         where l.user_id = $1 order by l.logged_at desc limit 20",
    )
    .bind(user_id)
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

    let can_log = last_60m < HOURLY_LIMIT && today_count < DAILY_LIMIT;
    Ok(StatsResponse {
        today_count,
        current_streak,
        longest_streak,
        week_count,
        alltime_count,
        recent_logs: recent,
        last_60m,
        can_log,
        next_allowed_at: if can_log { None } else { next_allowed_at(&SessionUser{ id: user_id, username: String::new(), display_name: None, avatar_url: None, provider: String::new() }, today_count, last_60m) },
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
        "alltime" => "alltime",
        _ => return Err(ApiError::bad_request("Unknown period")),
    };
    let rows: Vec<(uuid::Uuid, String, Option<String>, Option<String>, i64, Option<OffsetDateTime>)> =
        match period {
            "daily" => sqlx::query_as("select * from daily_leaderboard(50)").fetch_all(&state.pool).await?,
            "weekly" => sqlx::query_as("select * from weekly_leaderboard(50)").fetch_all(&state.pool).await?,
            _ => sqlx::query_as("select * from alltime_leaderboard(50)").fetch_all(&state.pool).await?,
        };
    let entries = rows
        .into_iter()
        .enumerate()
        .map(|(i, (uid, uname, dname, av, count, last))| LeaderboardEntry {
            rank: (i + 1) as i64,
            user_id: uid,
            username: uname,
            display_name: dname,
            avatar_url: av,
            count,
            last_log_at: last
                .map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()),
        })
        .collect();
    Ok(Json(LeaderboardResponse { period: period.into(), entries }))
}

pub async fn get_user_of_the_day(
    State(state): State<AppState>,
) -> ApiResult<Json<Option<UserOfTheDay>>> {
    let row: Option<(uuid::Uuid, String, Option<String>, Option<String>, i64, OffsetDateTime, i64)> =
        sqlx::query_as("select * from user_of_the_day()")
            .fetch_optional(&state.pool)
            .await?;
    Ok(Json(row.map(|(uid, uname, dname, av, count, first, all)| UserOfTheDay {
        user_id: uid,
        username: uname,
        display_name: dname,
        avatar_url: av,
        count,
        first_log_at: first
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        alltime_count: all,
    })))
}

pub async fn get_total(State(state): State<AppState>) -> ApiResult<Json<HashMap<&'static str, i64>>> {
    let total: i64 = sqlx::query_scalar("select count(*) from habit_logs").fetch_one(&state.pool).await?;
    Ok(Json(HashMap::from([("total", total)])))
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
    let row: Option<(uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime)> =
        sqlx::query_as(
            "select id, username, display_name, avatar_url, created_at from profiles where username = $1",
        )
        .bind(&username)
        .fetch_optional(&state.pool)
        .await?;
    let Some((id, uname, dname, av, created)) = row else {
        return Err(ApiError::not_found("Profile not found"));
    };
    let (streak, longest): (i64, i64) = sqlx::query_as("select current_streak::bigint, longest_streak::bigint from user_streak($1)")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    let today_start = OffsetDateTime::now_utc().date().midnight().assume_utc();
    let week_start = week_start_utc(OffsetDateTime::now_utc());
    let (today_count, week_count, alltime_count): (i64, i64, i64) = sqlx::query_as(
        "select
           (select count(*) from habit_logs where user_id = $1 and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1)",
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

    let updated = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, Option<String>, OffsetDateTime)>(
        "update profiles set username = $1, display_name = $2, avatar_url = $3, updated_at = now()
         where id = $4
         returning id, username, display_name, avatar_url, created_at",
    )
    .bind(&username)
    .bind(&display_name)
    .bind(&avatar_url)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, uname, dname, av, created)) = updated else {
        return Err(ApiError::not_found("Profile not found"));
    };

    // refresh session user
    let mut session = session;
    let new_user = SessionUser {
        id,
        username: uname.clone(),
        display_name: dname.clone(),
        avatar_url: av.clone(),
        provider: user.provider,
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
           (select count(*) from habit_logs where user_id = $1 and log_date = (now() at time zone 'utc')::date),
           (select count(*) from habit_logs where user_id = $1 and logged_at >= $2),
           (select count(*) from habit_logs where user_id = $1)",
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
