//! The HTTP surface, built from a pool and a config.
//!
//! This lives in the library rather than in `main.rs` so integration tests can
//! mount the REAL router and drive it with `tower::ServiceExt::oneshot`. That
//! matters more than it sounds: every SQL function this app reads is decoded by
//! position, and the two outages this project has had were both "the query
//! compiled, the tests passed, the public page returned 500". A test that goes
//! through the router and reads the JSON body is the only kind that notices —
//! a function-level test asserts against the same assumption the handler makes.

use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::Router;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{cookie::time::Duration as CookieDuration, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

use crate::config::Config;
use crate::AppState;

/// Build the whole application: the JSON API under `/api`, the static assets,
/// the SPA fallback, and the session layer.
///
/// Unknown `/api/*` paths 404 rather than serving the SPA shell, so a missing
/// endpoint cannot masquerade as the app (stale probing fails loud).
pub async fn build_router(cfg: &Config, pool: PgPool) -> Result<Router, sqlx::Error> {
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    // The signing key comes from SESSION_SECRET, not from randomness: see
    // auth::signing_key for why a per-process key signs every user out on every
    // restart. Rotating the secret is a deliberate act (it signs everyone out
    // once) and is the only thing that should ever change this key.
    let key = crate::auth::signing_key(&cfg.session_secret);

    // Remember users by default: sessions persist for SESSION_TTL_DAYS (30) of
    // inactivity instead of ending when the browser closes.
    //
    // SameSite=Lax rather than the library's Strict default: Strict withholds
    // the cookie on a cross-site top-level navigation, so arriving from a link
    // elsewhere loads the app shell in a signed-out-looking state until the
    // first same-origin fetch lands.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(crate::auth::session_cookie_name(cfg.secure_cookies))
        .with_expiry(Expiry::OnInactivity(CookieDuration::days(
            cfg.session_ttl_days,
        )))
        .with_same_site(SameSite::Lax)
        .with_signed(key)
        .with_secure(cfg.secure_cookies);

    let state = AppState {
        pool,
        cfg: cfg.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(
            cfg.public_url
                .parse::<axum::http::HeaderValue>()
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("http://localhost:8787")),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let api_router = Router::new()
        .route("/auth/me", get(crate::auth::me))
        .route("/auth/logout", post(crate::auth::logout))
        .route("/auth/register", post(crate::auth::register))
        .route("/auth/login", post(crate::auth::login))
        .route("/auth/nonce", get(crate::auth::nonce))
        // GS-3. `/consent` is reachable without consent by definition, and the
        // layer below refuses every other mutating request until it is given.
        .route(
            "/consent",
            get(crate::consent::get).post(crate::consent::post),
        )
        // GS-1. Minting is a mutation, so it goes through the consent gate above;
        // `GET` is here so the client can ask whether the door is open without
        // learning it from a failed click.
        .route("/guest", get(crate::guest::status).post(crate::guest::mint))
        .route("/logs", post(crate::api::log_habit))
        // The undo/edit window: the landing page logs on ONE click, so the same
        // fifteen minutes have to carry both "that number was wrong" and "that
        // click was wrong".
        .route(
            "/logs/{id}",
            patch(crate::api::update_log).delete(crate::api::delete_log),
        )
        .route("/cage", get(crate::cage::get_cage))
        .route("/cage-check", post(crate::cage::create_check))
        .route("/cage-check/queue", get(crate::cage::queue))
        .route("/cage-check/{id}/attest", post(crate::cage::attest))
        .route("/metrics/time-to-cum", get(crate::metrics::time_to_cum))
        .route("/metrics/boards", get(crate::metrics::boards))
        .route("/trust", get(crate::moderation::get_trust))
        .route("/flags", post(crate::moderation::flag))
        .route("/moderation/queue", get(crate::moderation::queue))
        .route("/moderation/action", post(crate::moderation::action))
        .route("/stats", get(crate::api::get_stats))
        .route("/drill", get(crate::api::get_drill))
        .route("/denial", get(crate::api::get_denial))
        .route("/lock", get(crate::api::get_lock))
        .route("/lock", post(crate::api::lock))
        .route("/unlock", post(crate::api::unlock))
        .route("/leaderboard/{period}", get(crate::api::get_leaderboard))
        // Public, like /leaderboard and /user-of-the-day: no auth extractor, so
        // the KPI page is readable by anyone, which is the point of it.
        .route("/kpi", get(crate::api::get_kpi))
        .route("/user-of-the-day", get(crate::api::get_user_of_the_day))
        .route("/total", get(crate::api::get_total))
        .route("/today", get(crate::api::get_today))
        .route("/feed", get(crate::api::get_feed))
        .route("/profile/{username}", get(crate::api::get_profile))
        .route("/profile", patch(crate::api::update_profile))
        .route("/doctrine", get(crate::manifesto::list))
        .route("/doctrine/{id}", get(crate::manifesto::get))
        .route("/health", get(crate::api::health))
        // Outermost layer on the API: it runs before every handler, so a route
        // added later is gated by existing rather than by remembering to call
        // the check. That is the failure this codebase has had before — a
        // server-side rule enforced in each handler is a rule with holes.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::consent::require_consent,
        ));

    // Serve real static files from the build root (manifest, icons, etc.)
    // BEFORE the SPA fallback, so PWA assets are served as themselves.
    let web_dir = cfg.web_build_dir.clone();
    let static_fallback = tower::service_fn(move |req: axum::extract::Request| {
        let web_dir = web_dir.clone();
        async move {
            if req.uri().path().starts_with("/api/") {
                return Ok::<_, std::convert::Infallible>(
                    axum::http::StatusCode::NOT_FOUND.into_response(),
                );
            }
            let path = std::path::Path::new(&web_dir).join("index.html");
            let res = match tokio::fs::read(&path).await {
                Ok(body) => axum::response::Response::new(axum::body::Body::from(body)),
                Err(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
            };
            Ok::<_, std::convert::Infallible>(res)
        }
    });

    Ok(Router::new()
        // The same handler at the root path as well, because `/health` is what a
        // person and most monitors try first -- and until this existed, that path
        // returned 200 with the SPA shell, i.e. it reported a healthy site whatever
        // the database was doing. `/api/health` remains the canonical one.
        .route("/health", get(crate::api::health))
        .nest("/api", api_router)
        .nest_service(
            "/_app",
            ServeDir::new(format!("{}/_app", cfg.web_build_dir)),
        )
        .fallback_service(ServeDir::new(&cfg.web_build_dir).fallback(static_fallback))
        .with_state(state)
        .layer(session_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}
