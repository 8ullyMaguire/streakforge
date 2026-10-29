//! The member's own data: take it away, or have it erased.
//!
//! The terms say both are possible; until now neither was. These two handlers are
//! the whole of the promise, and the shape of each is chosen so the promise can be
//! checked by reading the code rather than by trusting the copy:
//!
//! * **Export** is one SQL statement that builds the document server-side from a
//!   list of the member's own rows. A table added later that the export misses
//!   shows up as a missing key in the test that compares the export's section
//!   names against `pg_constraint`, so completeness is a test rather than a habit.
//! * **Erasure** is a password-gated transaction that deletes the profile (every
//!   content table references it `on delete cascade`), clears the member's
//!   sessions, and leaves two kinds of residue, both deliberate: anonymous audit
//!   rows saying an account was erased, and `moderation_actions` with a null actor
//!   (`on delete set null`), because a decision that was taken against someone
//!   else's content must not vanish when its author does.
//!
//! What is NOT erased, and is stated on `/privacy`: a folded guest tombstone is
//! a different profile (it was folded into this one, and it holds no content of
//! its own), and `moderation_actions` keeps the action, not the actor.

use axum::extract::State;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_sessions::Session;

use crate::api::require_user;
use crate::audit;
use crate::auth::{
    client_ip, ip_failures, record_login_failure, throttle_verdict, username_failures,
    username_failures_in_day, verify_password,
};
use crate::error::{ApiError, ApiResult};
use crate::AppState;

/// The window named in the refusal. The ceiling itself is the sign-in's
/// (`auth::throttle_verdict`), because it is the same password being guessed.
const ERASE_WINDOW_SECS: i64 = 900;

#[derive(Debug, Deserialize)]
pub struct EraseRequest {
    pub password: String,
    /// Typed by hand. A confirmation dialog is a click away from a mistake; a
    /// typed word is not, and this is the one irreversible action on the site.
    pub confirm: String,
}

/// `GET /api/account/export` — everything the site holds about the caller, as one
/// JSON document, served as a download.
pub async fn export(State(state): State<AppState>, session: Session) -> ApiResult<Response> {
    let user = require_user(&session).await?;

    let doc: Value = sqlx::query_scalar(
        r#"
        select json_build_object(
          'format', 'streakforge.account.v1',
          'exported_at', now(),
          'account', (
            select json_build_object(
              'id', p.id,
              'username', p.username,
              'display_name', p.display_name,
              'avatar_url', p.avatar_url,
              'social_url', p.social_url,
              'provider', p.provider,
              'is_guest', p.is_guest,
              'created_at', p.created_at,
              'consented_at', p.consented_at,
              'consent_version', p.consent_version,
              'logs_public', p.logs_public,
              'merged_into', p.merged_into
            ) from public.profiles p where p.id = $1
          ),
          'logs', (
            select coalesce(json_agg(json_build_object(
              'id', l.id, 'kind', l.kind, 'note', l.note,
              'logged_at', l.logged_at, 'log_date', l.log_date,
              'created_at', l.created_at,
              'time_to_cum_secs', l.time_to_cum_secs,
              'is_catch_up', l.is_catch_up, 'catch_up_on', l.catch_up_on,
              'hidden_at', l.hidden_at
            ) order by l.id), '[]'::json)
            from public.habit_logs l where l.user_id = $1
          ),
          'locks', (
            select coalesce(json_agg(json_build_object(
              'id', s.id, 'locked_at', s.locked_at, 'unlocked_at', s.unlocked_at,
              'reason', s.reason
            ) order by s.id), '[]'::json)
            from public.lock_sessions s where s.user_id = $1
          ),
          'cage_checks', (
            select coalesce(json_agg(json_build_object(
              'id', c.id, 'checked_at', c.checked_at, 'check_date', c.check_date,
              'seal_code', c.seal_code, 'note', c.note, 'claimed_secs', c.claimed_secs,
              'status', c.status, 'hidden_at', c.hidden_at
            ) order by c.id), '[]'::json)
            from public.cage_checks c where c.user_id = $1
          ),
          'cage_attestations_given', (
            select coalesce(json_agg(json_build_object(
              'check_id', a.check_id, 'verdict', a.verdict, 'created_at', a.created_at
            ) order by a.check_id), '[]'::json)
            from public.cage_attestations a where a.attester_id = $1
          ),
          'flags_filed', (
            select coalesce(json_agg(json_build_object(
              'target_kind', f.target_kind, 'target_id', f.target_id,
              'reason', f.reason, 'created_at', f.created_at
            ) order by f.id), '[]'::json)
            from public.flags f where f.reporter_id = $1
          ),
          'moderation_actions_taken', (
            select coalesce(json_agg(json_build_object(
              'target_kind', m.target_kind, 'target_id', m.target_id,
              'action', m.action, 'note', m.note, 'created_at', m.created_at
            ) order by m.id), '[]'::json)
            from public.moderation_actions m where m.actor_id = $1
          ),
          'audit_events', (
            select coalesce(json_agg(json_build_object(
              'action', a.action, 'detail', a.detail, 'at', a.at
            ) order by a.id), '[]'::json)
            from public.audit_log a where a.profile_id = $1
          ),
          'sessions', (
            select count(*) from tower_sessions.session
            where encode(data, 'escape') like '%' || $1::text || '%'
          )
        )
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;

    let filename = format!(
        "streakforge-{}-{}.json",
        user.username,
        time::OffsetDateTime::now_utc().date()
    );
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
            // The document is the member's own data; no cache should keep a copy.
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        Json(doc),
    )
        .into_response())
}

/// `POST /api/account/erase` — delete the account and everything it owns.
///
/// Requires the password (the one secret that proves it is the member and not a
/// borrowed session) and the word DELETE typed out. Both refusals are cheap and
/// both are tested.
pub async fn erase(
    State(state): State<AppState>,
    session: Session,
    headers: axum::http::HeaderMap,
    Json(body): Json<EraseRequest>,
) -> ApiResult<StatusCode> {
    let user = require_user(&session).await?;

    if body.confirm != "DELETE" {
        return Err(ApiError::bad_request(
            "Type DELETE to confirm. Nothing was erased.",
        ));
    }

    // The password is the secret being guessed, so it answers to the SAME throttle
    // as a sign-in — not a second, laxer one written for this endpoint. A wrong
    // password here must cost an attacker exactly what a wrong password at /login
    // costs, or erasure becomes the cheaper oracle.
    let ip = client_ip(&headers);
    let (user_fails, user_retry_secs) = username_failures(&state.pool, &user.username).await?;
    let ip_fails = ip_failures(&state.pool, &ip).await?;
    let user_fails_in_day = username_failures_in_day(&state.pool, &user.username).await?;
    if throttle_verdict(user_fails, ip_fails, user_retry_secs, user_fails_in_day).is_some() {
        return Err(ApiError::too_many_requests(format!(
            "Too many wrong passwords. Try again in {} minutes.",
            ERASE_WINDOW_SECS / 60
        )));
    }

    let hash: Option<String> =
        sqlx::query_scalar("select password_hash from public.profiles where id = $1")
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

    let ok = match &hash {
        Some(h) => verify_password(&body.password, h),
        // An account with no password (provider-only) cannot confirm by password;
        // refusing is the honest answer rather than erasing on a session alone.
        None => false,
    };
    if !ok {
        // Recorded exactly as a failed sign-in is, so the throttle above counts it.
        // `login_attempts.ip` is NOT NULL, which is why this goes through the same
        // helper rather than a hand-written insert with a null address.
        let _ = record_login_failure(&state.pool, &user.username, &ip).await;
        audit::record(
            &state.pool,
            Some(user.id),
            "account.erase_refused",
            json!({ "reason": "bad_password" }),
        )
        .await
        .ok();
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Wrong password. Nothing was erased.",
        ));
    }

    // One transaction: the counts, the session cleanup, the scrub and the delete.
    let mut tx = state.pool.begin().await?;

    let logs: i64 = sqlx::query_scalar("select count(*) from public.habit_logs where user_id = $1")
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;
    let locks: i64 =
        sqlx::query_scalar("select count(*) from public.lock_sessions where user_id = $1")
            .bind(user.id)
            .fetch_one(&mut *tx)
            .await?;
    let checks: i64 =
        sqlx::query_scalar("select count(*) from public.cage_checks where user_id = $1")
            .bind(user.id)
            .fetch_one(&mut *tx)
            .await?;

    // Sessions are msgpack blobs, so the id is matched as text rather than by a
    // JSON field: `tower_sessions.session.data` is bytea, not jsonb.
    sqlx::query(
        "delete from tower_sessions.session
          where encode(data, 'escape') like '%' || $1::text || '%'",
    )
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    // The audit trail keeps the consent record's WHEN (that is the record a
    // deletion request is answered from) but not its contents, which could name
    // the person. The row keeps `action` and `at`; `detail` is emptied.
    sqlx::query("update public.audit_log set detail = '{}'::jsonb where profile_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

    // `profiles` cascades to habit_logs, lock_sessions, cage_checks,
    // cage_attestations and flags; `moderation_actions.actor_id` and
    // `audit_log.profile_id` are `on delete set null` by design.
    let deleted = sqlx::query("delete from public.profiles where id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() != 1 {
        tx.rollback().await.ok();
        return Err(ApiError::not_found("That account no longer exists"));
    }

    // Anonymous, and the only thing that survives: how much was erased, not who.
    sqlx::query(
        "insert into public.audit_log (profile_id, action, detail)
         values (null, 'account.erased', $1)",
    )
    .bind(json!({ "logs": logs, "locks": locks, "cage_checks": checks }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // The cookie is already useless (the profile it pointed at is gone); tearing
    // the session down makes it obviously so.
    session.flush().await.map_err(ApiError::from)?;
    session.delete().await.map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Purge rows whose retention window has passed.
///
/// The terms describe a bounded history, so something has to implement the bound.
/// Deliberately narrow: folded guest tombstones older than the window (they exist
/// only to answer "where did my guest history go", and the answer stops being
/// useful after a year), login attempts past the throttle window, and audit rows
/// that are neither a consent record nor an erasure — the two kinds a deletion
/// request is answered from. Returns what it removed so a caller can log it.
pub async fn purge_expired(pool: &PgPool, tombstone_days: i32) -> Result<Value, sqlx::Error> {
    let tombstones: i64 = sqlx::query_scalar(
        "with gone as (
           delete from public.profiles
            where is_guest and merged_into is not null
              and created_at < now() - make_interval(days => $1)
            returning id)
         select count(*) from gone",
    )
    .bind(tombstone_days.max(1))
    .fetch_one(pool)
    .await?;

    let attempts: i64 = sqlx::query_scalar(
        "with gone as (
           delete from public.login_attempts
            where attempted_at < now() - interval '30 days'
            returning 1)
         select count(*) from gone",
    )
    .fetch_one(pool)
    .await?;

    let audit_rows: i64 = sqlx::query_scalar(
        "with gone as (
           delete from public.audit_log
            where at < now() - interval '365 days'
              and action not in ('consent.accepted', 'account.erased')
            returning 1)
         select count(*) from gone",
    )
    .fetch_one(pool)
    .await?;

    Ok(json!({
        "guest_tombstones": tombstones,
        "login_attempts": attempts,
        "audit_rows": audit_rows,
    }))
}
