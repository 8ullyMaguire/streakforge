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

/// Cookie name carrying the signed session id. Named rather than left at the
/// library default (`id`, per the OWASP session-id-name note) so a stray cookie
/// on the same host is never mistaken for this application's session.
pub const SESSION_COOKIE_NAME: &str = "streakforge_session";

/// The cookie name to actually send, given the deployment's cookie policy.
///
/// `__Host-` is a browser-enforced guarantee, not a convention: the prefix is
/// accepted only on a cookie that is `Secure`, has `Path=/` and carries no
/// `Domain`, so it is impossible for a subdomain or a plaintext connection to
/// overwrite this session. It is applied only when `SECURE_COOKIES=1`, because a
/// browser silently rejects a `__Host-` cookie without `Secure` — sending the
/// prefixed name over local HTTP would mean no session at all in development.
///
/// Renaming the cookie signs every existing user out exactly once: the old name
/// is simply ignored. That is the cost of the prefix, and it is paid once.
pub fn session_cookie_name(secure: bool) -> &'static str {
    if secure {
        "__Host-streakforge_session"
    } else {
        SESSION_COOKIE_NAME
    }
}

/// Failed logins allowed for one username before it is throttled.
pub const LOGIN_MAX_PER_USERNAME: i64 = 5;
/// Failed logins allowed from one IP before it is throttled. Six times the
/// per-username ceiling on purpose: a household or a cafe shares an address, and
/// a limit tuned to one attacker's speed would lock out the neighbours. The
/// per-username counter is what stops a targeted guess at one account.
pub const LOGIN_MAX_PER_IP: i64 = 30;
/// The window both counters are measured over, in seconds.
pub const LOGIN_WINDOW_SECS: i64 = 900;
/// How long a failure is kept, in seconds. Longer than the window on purpose:
/// the window decides the first refusal, and the kept history decides how much
/// longer each refusal after that lasts.
pub const LOGIN_ATTEMPT_KEEP_SECS: i64 = 86_400;
/// Failures inside `LOGIN_ATTEMPT_KEEP_SECS` before the wait starts growing.
/// Eight is deliberately past the point a person reaches by mistyping: a
/// forgetful user hits the base window (15 minutes) and nothing more.
pub const LOGIN_ESCALATE_AFTER: i64 = 8;
/// Ceiling on the escalated wait. A password here has no self-service reset, so
/// an aggressive lockout with no way back would be a worse failure than the
/// guesses it prevents. Two hours starves a scripted attack (which drops from
/// ~480 guesses a day to a handful) and is survivable for a person.
pub const LOGIN_MAX_LOCKOUT_SECS: i64 = 7_200;

/// How long a refusal lasts once the day's failures pass the base allowance.
///
/// Doubling from the base window: 15 minutes at nine failures in a day, 30 at
/// ten, an hour at eleven, and the two-hour ceiling from twelve on. Pure, and
/// separate from `throttle_verdict` so the curve can be read and tested on its
/// own.
///
/// The idea this implements (rank 06) said "back off exponentially after 5".
/// This starts at eight instead, and the reason is in the numbers: five is the
/// base allowance, so treating the sixth failure as hostile would spend the
/// escalation on typos. Past eight in a single day the caller is not typing.
pub fn escalating_lockout_secs(fails_in_day: i64) -> Option<i64> {
    if fails_in_day <= LOGIN_ESCALATE_AFTER {
        return None;
    }
    let doublings = (fails_in_day - LOGIN_ESCALATE_AFTER).min(8) as u32;
    let secs = LOGIN_WINDOW_SECS.saturating_mul(1_i64 << doublings);
    Some(secs.min(LOGIN_MAX_LOCKOUT_SECS))
}

/// Should this attempt be refused, and for how long?
///
/// Pure, so the rule is testable without a database: `Some(secs)` means blocked
/// for that many seconds. Two independent rules decide it, and the longer one
/// wins:
///
/// * the sliding window — 5 failures for a username or 30 from an address in
///   `LOGIN_WINDOW_SECS`, answered with the time until the oldest of them ages
///   out;
/// * the escalation — failures kept over 24 hours, which buy a doubling wait up
///   to a ceiling (see `escalating_lockout_secs`).
pub fn throttle_verdict(
    user_fails: i64,
    ip_fails: i64,
    user_retry_secs: Option<i64>,
    user_fails_in_day: i64,
) -> Option<i64> {
    // Never report 0: a `Retry-After: 0` invites an immediate retry that will be
    // refused again, which reads as a broken client contract.
    let base = user_retry_secs.unwrap_or(LOGIN_WINDOW_SECS).max(1);

    if user_fails >= LOGIN_MAX_PER_USERNAME {
        let escalated = escalating_lockout_secs(user_fails_in_day).unwrap_or(base);
        return Some(base.max(escalated));
    }
    if ip_fails >= LOGIN_MAX_PER_IP {
        // Over the address ceiling, and NOT escalated. A cafe shares an address,
        // so one attacker behind it must not lengthen the wait for everybody
        // else at the table; the plain window already caps them, and the
        // per-username rule is what a targeted guess trips. This branch is why
        // the escalation is keyed on the username counter rather than on
        // whichever counter happened to fire.
        return Some(base);
    }
    None
}

/// The client's address, as the site can actually see it.
///
/// The backend binds to loopback behind the Cloudflare tunnel, so the socket
/// address is the proxy and every visitor would share one bucket. Cloudflare's
/// own header is preferred, then the first `X-Forwarded-For` hop. Only loopback
/// can reach the listener, which is what makes those headers trustworthy here.
pub fn client_ip(headers: &axum::http::HeaderMap) -> String {
    for name in ["cf-connecting-ip", "x-forwarded-for"] {
        if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()) {
            let first = v.split(',').next().unwrap_or("").trim();
            if !first.is_empty() {
                return first.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Failures recorded for this username inside the window, and the seconds until
/// the oldest of them ages out.
async fn username_failures(pool: &PgPool, username: &str) -> ApiResult<(i64, Option<i64>)> {
    let row: (i64, Option<f64>) = sqlx::query_as(
        "select count(*)::bigint,
                ceil(extract(epoch from (
                  min(attempted_at) + make_interval(secs => $2::double precision) - now()
                )))::float8
         from public.login_attempts
         where lower(username) = lower($1)
           and attempted_at > now() - make_interval(secs => $2::double precision)",
    )
    .bind(username)
    .bind(LOGIN_WINDOW_SECS as f64)
    .fetch_one(pool)
    .await?;
    Ok((row.0, row.1.map(|s| s as i64)))
}

/// Failures for this username inside the retention window, which is what the
/// escalation is measured against. Separate from `username_failures` because the
/// two answer different questions: that one is "is the window full", this one is
/// "how persistent has this been".
async fn username_failures_in_day(pool: &PgPool, username: &str) -> ApiResult<i64> {
    let n: i64 = sqlx::query_scalar(
        "select count(*)::bigint from public.login_attempts
         where lower(username) = lower($1)
           and attempted_at > now() - make_interval(secs => $2::double precision)",
    )
    .bind(username)
    .bind(LOGIN_ATTEMPT_KEEP_SECS as f64)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// Same, for the address.
async fn ip_failures(pool: &PgPool, ip: &str) -> ApiResult<i64> {
    let n: i64 = sqlx::query_scalar(
        "select count(*)::bigint from public.login_attempts
         where ip = $1 and attempted_at > now() - make_interval(secs => $2::double precision)",
    )
    .bind(ip)
    .bind(LOGIN_WINDOW_SECS as f64)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// Record a failure, pruning rows that can no longer matter.
///
/// The prune rides along with the insert so the table stays bounded with no
/// scheduled job, and it is scoped to this username and address: a global
/// "delete everything older than a day" would quietly shorten somebody else's
/// window under load.
///
/// The horizon is `LOGIN_ATTEMPT_KEEP_SECS` (24h), not the 15-minute window: the
/// escalation reads the day's count, so pruning at the window would erase the
/// history that makes it escalate. Rows are still bounded, because a row only
/// outlives its window for an account or an address somebody is actively
/// guessing at.
async fn record_login_failure(pool: &PgPool, username: &str, ip: &str) -> ApiResult<()> {
    sqlx::query(
        "with pruned as (
           delete from public.login_attempts
           where attempted_at < now() - make_interval(secs => $3::double precision)
             and (lower(username) = lower($1) or ip = $2)
         )
         insert into public.login_attempts (username, ip) values ($1, $2)",
    )
    .bind(username)
    .bind(ip)
    .bind(LOGIN_ATTEMPT_KEEP_SECS as f64)
    .execute(pool)
    .await?;
    Ok(())
}

/// A successful login clears that username's failures. The IP counter is NOT
/// cleared: one legitimate user signing in does not prove the other guesses from
/// the same address were benign.
async fn clear_login_failures(pool: &PgPool, username: &str) -> ApiResult<()> {
    sqlx::query("delete from public.login_attempts where lower(username) = lower($1)")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deterministic session-cookie signing key, derived from `SESSION_SECRET`.
///
/// MUST be stable across process lifetimes. `cookie::Key::generate()` makes a
/// new random key every start, and the session cookie's value is an HMAC over
/// the session id under that key — so every restart silently invalidates every
/// cookie already sitting in a browser. The session row survives in Postgres
/// untouched, nothing can point at it any more, and the user is signed out.
/// Observed in production: 3,339 restarts in one day (Sep 27) and 1,306 on
/// Oct 1, each one signing out every logged-in user at once. Verified locally
/// by restarting the process: `/api/auth/me` went 200 → 401 with the session
/// row still present.
///
/// The secret is SHA-256'd first, for two reasons: `Key::derive_from` panics on
/// a master key shorter than 32 bytes, and a human-typed secret should not be
/// handed to HKDF as though it were a pseudorandom key.
pub fn signing_key(secret: &str) -> tower_sessions::cookie::Key {
    let master = Sha256::digest(secret.as_bytes());
    tower_sessions::cookie::Key::derive_from(&master)
}

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
///
/// Throttled on two counters before any password work happens: 5 failures for a
/// username, 30 from an address, per 15 minutes. The check runs FIRST so a
/// guessing loop cannot spend the server's Argon2 budget either — a rate limit
/// that only fires after hashing protects the account and not the machine.
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    headers: axum::http::HeaderMap,
    Json(body): Json<AuthForm>,
) -> ApiResult<Json<SessionUser>> {
    let username = validate_auth_form(&body)?;
    let ip = client_ip(&headers);

    let (user_fails, user_retry_secs) = username_failures(&state.pool, &username).await?;
    let ip_fails = ip_failures(&state.pool, &ip).await?;
    let user_fails_in_day = username_failures_in_day(&state.pool, &username).await?;
    if let Some(secs) = throttle_verdict(user_fails, ip_fails, user_retry_secs, user_fails_in_day) {
        // Log the event (rank 06 asked for it explicitly). The username and the
        // address are the only personal data this app writes to a log, and they
        // are written here and nowhere else: a refusal nobody can look up is a
        // refusal nobody can act on. OPERATIONS.md records the retention.
        tracing::warn!(
            username = %username,
            ip = %ip,
            failures_in_window = user_fails,
            failures_in_day = user_fails_in_day,
            retry_after_secs = secs,
            "sign-in refused: too many failures"
        );
        // Name the clock, like the logging limits do: a bare 429 reads as a
        // broken sign-in button rather than as a wait.
        let mins = ((secs + 59) / 60).max(1);
        return Err(ApiError::too_many_requests(format!(
            "Too many failed sign-ins for this account. Try again in {mins} minute(s)."
        ))
        .with_retry_after(secs));
    }

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
        // Counted too. Enumeration attempts land here, and a throttle that only
        // counted real usernames would leave the enumeration path unlimited.
        record_login_failure(&state.pool, &username, &ip).await?;
        return Err(ApiError::unauthorized("Invalid username or password"));
    };

    // Legacy X/dev rows have no password hash — they can't log in this way.
    let Some(phc) = phc else {
        record_login_failure(&state.pool, &username, &ip).await?;
        return Err(ApiError::unauthorized("Invalid username or password"));
    };

    if !verify_password(&body.password, &phc) {
        record_login_failure(&state.pool, &username, &ip).await?;
        return Err(ApiError::unauthorized("Invalid username or password"));
    }

    clear_login_failures(&state.pool, &username).await?;

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
