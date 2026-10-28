//! GS-1 — guest access: a guest is a real identity, not a shadow.
//!
//! Estate standard §1. The mechanism is deliberately unglamorous: a guest is a
//! `profiles` row with `is_guest = true` and **no credential at all**. Every
//! authorship column in this schema already points at `profiles(id)`, so a parallel
//! guests table would mean two sources of truth for "who did this" — the cost the
//! standard says has already been paid once in this estate and is not paid again.
//!
//! What makes the identity exist is the **session**, which is the cookie. So the
//! ordering in the standard is not decoration: `POST /api/guest` is a mutation, so
//! it goes through GS-3's consent gate — the gate really does gate the identity.
//!
//! The door closes on MAU (§1): when the count of accounts that acted inside the
//! window reaches `guest.mau_limit`, new identities are refused and guest writes
//! stop. Existing guest data stays readable and stays foldable — refusing to *start*
//! is not the same as taking away what someone already made.
//!
//! Parity (§1): a guest is a level-1 user and nothing more. On this site that means
//! it can log, lock and edit its profile like anyone else, and cannot sign in with a
//! password because there is nothing to sign in with.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::{SessionUser, SESSION_USER_KEY};
use crate::error::{ApiError, ApiResult};
use crate::AppState;

pub const MINTED_ACTION: &str = "guest.minted";

/// The audit action recorded when a guest's rows move to a new account.
pub const FOLDED_ACTION: &str = "guest.folded";

#[derive(Debug, Serialize)]
pub struct GuestStatus {
    pub enabled: bool,
    pub is_guest: bool,
    /// Distinct accounts that acted inside the window, guests and members together.
    pub mau: i64,
    pub limit: i64,
    pub window_days: i64,
    pub at_cap: bool,
}

/// A guest's username: a stable prefix and 8 hex characters, matching the site's
/// `^[a-zA-Z0-9_]{2,30}$` rule. The prefix is how a person reading the board knows
/// the account has no password behind it.
pub fn username_for(suffix: &str) -> String {
    format!("guest_{suffix}")
}

/// Salted hash of the address a guest identity was minted from.
///
/// Hashed rather than stored raw: the standard asks for "one identity per cookie + IP
/// fingerprint" and the rule needs *a* value to compare, not the address itself. The
/// salt is the session secret, so the column is useless to anyone who steals only the
/// database.
pub fn fingerprint(headers: &HeaderMap, secret: &str) -> String {
    let ip = crate::auth::client_ip(headers);
    let agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"|");
    hasher.update(ip.as_bytes());
    hasher.update(b"|");
    hasher.update(agent.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Distinct accounts that acted inside the configured window.
pub async fn monthly_active(pool: &PgPool, window_days: i64) -> sqlx::Result<i64> {
    sqlx::query_scalar("select public.monthly_active_accounts($1)")
        .bind(window_days as i32)
        .fetch_one(pool)
        .await
}

/// Is the door closed? `>=` rather than `>`: at exactly the limit the next identity
/// would exceed it, so the limit is the cap and not one past it.
pub fn is_at_cap(mau: i64, limit: i64) -> bool {
    limit > 0 && mau >= limit
}

/// Refuse a guest write when the door is closed.
///
/// Called by every mutating handler a guest can reach. It is a function rather than a
/// middleware because the rule is deliberately narrower than "authenticated" —
/// guests keep *reading* at the cap (§1's grace), so a blanket layer would take away
/// exactly what the standard says must survive.
pub async fn guard_write(state: &AppState, user: &SessionUser) -> ApiResult<()> {
    if !user.is_guest || !state.cfg.guest_enabled {
        return Ok(());
    }
    let mau = monthly_active(&state.pool, state.cfg.guest_mau_window_days).await?;
    if is_at_cap(mau, state.cfg.guest_mau_limit) {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "The site is at its guest limit — new activity needs an account",
        ));
    }
    Ok(())
}

/// GET /api/guest — what the client needs to decide whether to offer the door.
pub async fn status(session: Session, State(state): State<AppState>) -> ApiResult<Json<GuestStatus>> {
    let user = current(&session).await?;
    let mau = monthly_active(&state.pool, state.cfg.guest_mau_window_days).await?;
    Ok(Json(GuestStatus {
        enabled: state.cfg.guest_enabled && state.cfg.guest_mau_limit > 0,
        is_guest: user.as_ref().is_some_and(|u| u.is_guest),
        mau,
        limit: state.cfg.guest_mau_limit,
        window_days: state.cfg.guest_mau_window_days,
        at_cap: is_at_cap(mau, state.cfg.guest_mau_limit),
    }))
}

/// POST /api/guest — mint a guest identity for this session, or return the one it
/// already holds.
///
/// Idempotent on purpose: a double click must not mint two identities, and neither
/// must a retry after a timeout the client never saw succeed.
pub async fn mint(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> ApiResult<(StatusCode, Json<SessionUser>)> {
    if !state.cfg.guest_enabled || state.cfg.guest_mau_limit <= 0 {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Guest access is closed on this site",
        ));
    }
    if let Some(existing) = current(&session).await? {
        let code = if existing.is_guest {
            StatusCode::OK
        } else {
            return Err(ApiError::conflict(
                "This session already has an account — nothing to mint",
            ));
        };
        return Ok((code, Json(existing)));
    }

    let mau = monthly_active(&state.pool, state.cfg.guest_mau_window_days).await?;
    if is_at_cap(mau, state.cfg.guest_mau_limit) {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "The site is at its guest limit — register to take part",
        ));
    }

    let print = fingerprint(&headers, &state.cfg.session_secret);
    let user = insert_guest(&state.pool, &print).await?;

    // Best effort, like every other audit write: the identity exists even if the
    // record of its creation failed, and the next action writes one anyway.
    if let Err(err) = crate::audit::record(
        &state.pool,
        Some(user.id),
        MINTED_ACTION,
        json!({ "mau": mau, "limit": state.cfg.guest_mau_limit }),
    )
    .await
    {
        tracing::warn!(error = %err, "guest minted but the audit row was not written");
    }

    session
        .insert(SESSION_USER_KEY, &user)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// The session's user, if there is one. Not an error when there is not.
pub async fn current(session: &Session) -> ApiResult<Option<SessionUser>> {
    session
        .get::<SessionUser>(SESSION_USER_KEY)
        .await
        .map_err(ApiError::from)
}

async fn insert_guest(pool: &PgPool, fingerprint: &str) -> ApiResult<SessionUser> {
    // Five attempts at a random suffix: a collision is 1 in 4 billion per try, and a
    // loop that cannot end is worse than an error the person can act on.
    for _ in 0..5 {
        let suffix = format!("{:08x}", rand::random::<u32>());
        let username = username_for(&suffix);
        let row = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            insert into profiles (username, is_guest, guest_fingerprint, provider)
            values ($1, true, $2, 'local')
            on conflict (username) do nothing
            returning id, username
            "#,
        )
        .bind(&username)
        .bind(fingerprint)
        .fetch_optional(pool)
        .await?;

        if let Some((id, name)) = row {
            return Ok(SessionUser {
                id,
                username: name,
                display_name: None,
                avatar_url: None,
                provider: "local".into(),
                social_url: None,
                is_guest: true,
            });
        }
    }
    Err(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "Could not mint a guest identity — try again",
    ))
}
