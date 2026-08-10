// Auth — session-based with local username/password registration + login.
// Bot-dissuasion for the public register/login forms:
//   - honeypot field (hidden in the form; bots that autofill it get rejected)
//   - form timing (the form must have been opened 3s..10min before submit)
//   - JS challenge: client fetches a nonce from GET /api/auth/nonce, computes
//     proof = sha256(nonce + username + password) truncated to 16 hex chars
//     (16 hex chars = 64 bits of work the client must actually compute), and
//     the server recomputes it. Proof-of-work style — a bot that skips the
//     challenge cannot produce a valid proof without running the JS.
// Passwords are stored as Argon2id PHC strings. No SMTP, no email.

use crate::error::{ApiError, ApiResult};
use crate::AppState;
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_sessions::Session;
use uuid::Uuid;

pub const SESSION_USER_KEY: &str = "user";

/// Minimum wall-clock time a human needs to open a form and submit it.
pub const FORM_MIN_OPEN_MS: u128 = 3_000;
/// Reject forms that claim to have been open for longer than this.
pub const FORM_MAX_OPEN_MS: u128 = 600_000; // 10 minutes

/// Argon2id parameters (OWASP-recommended baseline).
const ARGON2_M_COST: u32 = 19_456; // ~19 MiB
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_url: Option<String>,
}

// ---- Password hashing (Argon2id) ----

pub fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, None)
            .map_err(|e| {
                tracing::error!("argon2 params: {e}");
                ApiError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Password hashing unavailable",
                )
            })?,
    );
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            tracing::error!("argon2 hash failed: {e}");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Password hashing failed",
            )
        })
}

/// Constant-time verification against a stored PHC string. Returns false for
/// any malformed hash rather than erroring, so login can't distinguish
/// "unknown user" from "bad hash".
pub fn verify_password(password: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

// ---- JS challenge (proof-of-work) ----

/// Server-side half of the challenge: recompute what the client must have
/// computed. proof must equal sha256(nonce || username || password)[0..16]
/// (first 16 hex chars = 64 bits).
pub fn compute_challenge_proof(nonce: &str, username: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    hasher.update(username.as_bytes());
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    digest[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

pub fn challenge_proof_valid(proof: &str, nonce: &str, username: &str, password: &str) -> bool {
    if proof.len() != 16 || !proof.bytes().all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    // constant-time comparison of the hex string
    let expected = compute_challenge_proof(nonce, username, password);
    let a = proof.as_bytes();
    let b = expected.as_bytes();
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn fresh_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---- Bot-dissuasion validation ----

#[derive(Debug, Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
    /// ms epoch when the form was opened (client clock)
    pub form_opened_at: Option<i64>,
    /// honeypot field — must be empty (bots fill every input)
    #[serde(default)]
    pub website: String,
    pub challenge_proof: Option<String>,
    pub challenge_nonce: Option<String>,
}

/// Common validation for register + login forms.
/// Returns the trimmed username on success.
fn validate_auth_form(body: &AuthForm) -> ApiResult<String> {
    let username = body.username.trim().to_string();

    // Honeypot: bots fill hidden fields.
    if !body.website.trim().is_empty() {
        return Err(ApiError::bad_request("Invalid form submission"));
    }

    // Form timing: must have been open between 3s and 10min.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            tracing::error!("clock error: {e}");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Server clock error",
            )
        })?
        .as_millis();
    let opened = body.form_opened_at.unwrap_or(0) as u128;
    let open_ms = now_ms.saturating_sub(opened);
    if open_ms < FORM_MIN_OPEN_MS || open_ms > FORM_MAX_OPEN_MS {
        return Err(ApiError::bad_request("Invalid form submission"));
    }

    // JS challenge proof-of-work.
    let proof = body
        .challenge_proof
        .as_deref()
        .ok_or_else(|| ApiError::bad_request("Invalid form submission"))?;
    let nonce = body
        .challenge_nonce
        .as_deref()
        .ok_or_else(|| ApiError::bad_request("Invalid form submission"))?;
    if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ApiError::bad_request("Invalid form submission"));
    }
    if !challenge_proof_valid(proof, nonce, &username, &body.password) {
        return Err(ApiError::bad_request("Invalid form submission"));
    }

    Ok(username)
}

fn valid_username(u: &str) -> bool {
    !u.is_empty()
        && u.chars().count() >= 2
        && u.chars().count() <= 30
        && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---- Handlers ----

pub async fn me(session: Session) -> ApiResult<Json<SessionUser>> {
    let user: Option<SessionUser> = session
        .get(SESSION_USER_KEY)
        .await
        .map_err(ApiError::from)?;
    match user {
        Some(u) => Ok(Json(u)),
        None => Err(ApiError::unauthorized("Sign in required")),
    }
}

pub async fn logout(session: Session) -> ApiResult<axum::response::Response> {
    session.flush().await.map_err(ApiError::from)?;
    Ok(axum::response::Redirect::to("/").into_response())
}

/// GET /api/auth/nonce — fresh random hex nonce for the JS challenge.
pub async fn nonce() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "nonce": fresh_nonce() }))
}

/// POST /api/auth/register — create a local profile and log in.
pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<AuthForm>,
) -> ApiResult<(axum::http::StatusCode, Json<SessionUser>)> {
    let username = validate_auth_form(&body)?;

    if !valid_username(&username) {
        return Err(ApiError::bad_request(
            "Username must be 2–30 chars, letters/digits/underscore only",
        ));
    }
    if body.password.chars().count() < 8 {
        return Err(ApiError::bad_request(
            "Password must be at least 8 characters",
        ));
    }
    if body.password.chars().count() > 1024 {
        return Err(ApiError::bad_request("Password too long"));
    }

    let password_hash = hash_password(&body.password)?;

    // Explicit conflict check first so we can return a clean 409
    // (the unique constraint would also catch it via sqlx::Error mapping).
    let taken: Option<String> =
        sqlx::query_scalar("select username from profiles where username = $1")
            .bind(&username)
            .fetch_optional(&state.pool)
            .await?;
    if taken.is_some() {
        return Err(ApiError::conflict("That username is already taken"));
    }

    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, String, Option<String>)>(
        r#"
        insert into profiles (username, display_name, avatar_url, provider, provider_id, password_hash)
        values ($1, NULL, NULL, 'local', NULL, $2)
        returning id, username, display_name, avatar_url, provider, social_url
        "#,
    )
    .bind(&username)
    .bind(&password_hash)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, uname, dname, av, provider, social_url)) = row else {
        // Race: someone else took the username between check and insert.
        return Err(ApiError::conflict("That username is already taken"));
    };

    let user = SessionUser {
        id,
        username: uname,
        display_name: dname,
        avatar_url: av,
        provider,
        social_url,
    };
    session.insert(SESSION_USER_KEY, &user).await.map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, Json(user)))
}

/// POST /api/auth/login — verify credentials and start a session.
/// Any failure returns a generic 401 so we don't leak which field was wrong.
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<AuthForm>,
) -> ApiResult<Json<SessionUser>> {
    let username = validate_auth_form(&body)?;

    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, String, Option<String>, Option<String>)>(
        "select id, username, display_name, avatar_url, provider, social_url, password_hash
         from profiles where username = $1",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, uname, dname, av, provider, social_url, phc)) = row else {
        // Unknown user: burn a tiny bit of CPU so the response time doesn't
        // reveal whether the username exists.
        let _ = hash_password(&body.password);
        return Err(ApiError::unauthorized("Invalid username or password"));
    };

    // Legacy X/dev rows have no password hash — they can't log in this way.
    let Some(phc) = phc else {
        return Err(ApiError::unauthorized("Invalid username or password"));
    };

    if !verify_password(&body.password, &phc) {
        return Err(ApiError::unauthorized("Invalid username or password"));
    }

    let user = SessionUser {
        id,
        username: uname,
        display_name: dname,
        avatar_url: av,
        provider,
        social_url,
    };
    session.insert(SESSION_USER_KEY, &user).await.map_err(ApiError::from)?;

    Ok(Json(user))
}

// ---- Profile helpers (registration + session refresh) ----

pub async fn fetch_profile(pool: &PgPool, id: Uuid) -> ApiResult<SessionUser> {
    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, String, Option<String>)>(
        "select id, username, display_name, avatar_url, provider, social_url
         from profiles where id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((id, uname, dname, av, provider, social_url)) = row else {
        return Err(ApiError::not_found("Profile not found"));
    };
    Ok(SessionUser {
        id,
        username: uname,
        display_name: dname,
        avatar_url: av,
        provider,
        social_url,
    })
}
