// Auth — session-based with X OAuth (PKCE) + dev login.
// tower-sessions Session extractor auto-creates sessions, so OAuth
// callback/dev-login just insert the user into the session.

use crate::config::Config;
use crate::error::{ApiError, ApiResult};
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

pub const SESSION_USER_KEY: &str = "user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub provider: String,
}

// ---- X OAuth ----

#[derive(Debug, Clone)]
pub struct XOAuth {
    client: Option<BasicClient>,
}

impl XOAuth {
    pub fn new(cfg: &Config) -> Self {
        let client = match (&cfg.x_client_id, &cfg.x_client_secret) {
            (Some(id), Some(secret)) => {
                let auth_url = AuthUrl::new("https://twitter.com/i/oauth2/authorize".into())
                    .expect("valid auth url");
                let token_url = TokenUrl::new("https://api.twitter.com/2/oauth2/token".into())
                    .expect("valid token url");
                let _ = (&auth_url, &token_url); // URLs built inline in BasicClient::new
                Some(
                    BasicClient::new(
                        ClientId::new(id.clone()),
                        Some(ClientSecret::new(secret.clone())),
                        AuthUrl::new("https://twitter.com/i/oauth2/authorize".into())
                            .expect("valid auth url"),
                        Some(
                            TokenUrl::new("https://api.twitter.com/2/oauth2/token".into())
                                .expect("valid token url"),
                        ),
                    )
                    .set_redirect_uri(
                        RedirectUrl::new(format!("{}/api/auth/x/callback", cfg.public_url))
                            .expect("valid redirect url"),
                    ),
                )
            }
            _ => None,
        };
        Self { client }
    }

    pub fn enabled(&self) -> bool {
        self.client.is_some()
    }

    pub fn start(&self) -> ApiResult<(String, String, String)> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ApiError::bad_request("X login not configured"))?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("users.read".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        Ok((
            auth_url.to_string(),
            csrf_token.secret().clone(),
            pkce_verifier.secret().clone(),
        ))
    }

    pub async fn exchange(&self, code: &str, verifier: &str) -> ApiResult<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ApiError::bad_request("X login not configured"))?;
        let token = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                tracing::error!("oauth exchange failed: {e}");
                ApiError::bad_request("X OAuth exchange failed")
            })?;
        Ok(token.access_token().secret().clone())
    }
}

pub fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("streakforge/0.1")
        .build()
        .expect("reqwest client")
}

#[derive(Debug, Deserialize)]
struct XUserResponse {
    data: XUserData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XUserData {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub name: String,
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

pub async fn logout(session: Session) -> ApiResult<Response> {
    session.flush().await.map_err(ApiError::from)?;
    Ok(Redirect::to("/").into_response())
}

pub async fn login_start(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<Response> {
    let (url, csrf, verifier) = state.x_oauth.start()?;
    session.insert("oauth_csrf", &csrf).await.map_err(ApiError::from)?;
    session
        .insert("oauth_verifier", &verifier)
        .await
        .map_err(ApiError::from)?;
    Ok(Redirect::to(&url).into_response())
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn login_callback(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<OAuthCallbackParams>,
) -> ApiResult<Response> {
    if let Some(err) = &params.error {
        tracing::warn!("oauth error: {err}");
        return Err(ApiError::bad_request(format!("X login failed: {err}")));
    }
    let expected_csrf: Option<String> = session.get("oauth_csrf").await.map_err(ApiError::from)?;
    let verifier: Option<String> = session
        .get("oauth_verifier")
        .await
        .map_err(ApiError::from)?;
    let state_param = params.state.ok_or_else(|| ApiError::bad_request("Missing state"))?;
    if expected_csrf.as_deref() != Some(state_param.as_str()) {
        return Err(ApiError::bad_request("OAuth state mismatch"));
    }
    let code = params.code.ok_or_else(|| ApiError::bad_request("Missing code"))?;
    let verifier = verifier.ok_or_else(|| ApiError::bad_request("Missing verifier"))?;
    let access_token = state.x_oauth.exchange(&code, &verifier).await?;
    let x_user = fetch_x_user(&access_token).await?;
    let user = upsert_profile(&state.pool, &x_user, "x").await?;
    session.insert(SESSION_USER_KEY, &user).await.map_err(ApiError::from)?;
    session.remove::<String>("oauth_csrf").await.map_err(ApiError::from)?;
    session.remove::<String>("oauth_verifier").await.map_err(ApiError::from)?;
    Ok(Redirect::to("/dashboard").into_response())
}

pub async fn fetch_x_user(access_token: &str) -> ApiResult<XUserData> {
    let resp = reqwest_client()
        .get("https://api.twitter.com/2/users/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("x user fetch failed: {e}");
            ApiError::bad_request("Failed to fetch X profile")
        })?;
    if !resp.status().is_success() {
        tracing::error!("x user fetch status: {}", resp.status());
        return Err(ApiError::bad_request("Failed to fetch X profile"));
    }
    let body: XUserResponse = resp.json().await.map_err(|e| {
        tracing::error!("x user parse failed: {e}");
        ApiError::bad_request("Failed to parse X profile")
    })?;
    Ok(body.data)
}

pub async fn upsert_profile(pool: &PgPool, x: &XUserData, provider: &str) -> ApiResult<SessionUser> {
    let base = sanitize_username(&x.username);
    for i in 0..100u32 {
        let candidate = if i == 0 {
            base.clone()
        } else {
            format!("{base}_{i}")
        };
        let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, String)>(
            r#"
            insert into profiles (id, username, display_name, avatar_url, provider, provider_id)
            values (gen_random_uuid(), $1, $2, NULL, $3, $4)
            on conflict (provider, provider_id) do update set provider_id = excluded.provider_id
            returning id, username, display_name, avatar_url, provider
            "#,
        )
        .bind(&candidate)
        .bind(if x.name.is_empty() { None } else { Some(&x.name) })
        .bind(provider)
        .bind(&x.id)
        .fetch_optional(pool)
        .await?;
        if let Some((id, uname, dname, av, prov)) = row {
            return Ok(SessionUser {
                id,
                username: uname,
                display_name: dname,
                avatar_url: av,
                provider: prov,
            });
        }
    }
    Err(ApiError::conflict("Could not create profile"))
}

pub fn sanitize_username(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim_matches('_').to_string();
    let base = if trimmed.len() < 2 {
        format!("user{trimmed}")
    } else {
        trimmed
    };
    base.chars().take(30).collect()
}

// ---- Dev login (local testing only) ----

#[derive(Debug, Deserialize)]
pub struct DevLoginParams {
    pub username: Option<String>,
}

pub async fn dev_login(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<DevLoginParams>,
) -> ApiResult<Response> {
    if !state.cfg.allow_dev_login {
        return Err(ApiError::unauthorized("Dev login disabled"));
    }
    let username = params
        .username
        .clone()
        .unwrap_or_else(|| "dev_forger".into());
    let username = sanitize_username(&username);
    let x = XUserData {
        id: format!("dev-{username}"),
        username: username.clone(),
        name: "Dev Forger".into(),
    };
    let user = upsert_profile(&state.pool, &x, "dev").await?;
    session.insert(SESSION_USER_KEY, &user).await.map_err(ApiError::from)?;
    Ok(Redirect::to("/dashboard").into_response())
}
