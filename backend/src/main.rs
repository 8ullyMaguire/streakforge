use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::Router;
use streakforge_api::auth;
use streakforge_api::config::Config;
use streakforge_api::db;
use streakforge_api::AppState;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{cookie::time::Duration as CookieDuration, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

// SPA fallback is handled by fallback_service(ServeDir + index.html fallback)
// in main(). Unknown /api/* paths return 404 from the API nest itself.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "streakforge=debug,tower_http=debug,sqlx=warn".into()),
        )
        .init();

    let cfg = Config::from_env();
    let pool = db::connect(&cfg.database_url).await?;
    db::run_migrations(&pool, &cfg.migrations_dir).await?;
    tracing::info!("migrations applied");

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    // The signing key comes from SESSION_SECRET, not from randomness: see
    // auth::signing_key for why a per-process key signs every user out on every
    // restart. Rotating the secret is a deliberate act (it signs everyone out
    // once) and is the only thing that should ever change this key.
    let key = auth::signing_key(&cfg.session_secret);
    // Remember users by default: sessions persist for SESSION_TTL_DAYS (30) of
    // inactivity instead of ending when the browser closes. Combined with the
    // signed cookie and the now-stable key, a returning whiteboi stays signed
    // in across restarts and deploys.
    //
    // SameSite=Lax rather than the library's Strict default: Strict withholds
    // the cookie on a cross-site top-level navigation, so arriving from a link
    // elsewhere (a Bluesky post, a chat) loads the app shell in a
    // signed-out-looking state until the first same-origin fetch lands. Lax
    // sends the cookie on exactly those navigations and still blocks it on
    // cross-site subrequests, which is the whole point of the attribute.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(auth::SESSION_COOKIE_NAME)
        .with_expiry(Expiry::OnInactivity(CookieDuration::days(cfg.session_ttl_days)))
        .with_same_site(SameSite::Lax)
        .with_signed(key)
        .with_secure(cfg.secure_cookies);

    let state = AppState {
        pool,
        cfg: cfg.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(cfg.public_url.parse::<axum::http::HeaderValue>()?)
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
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/nonce", get(auth::nonce))
        .route("/logs", post(streakforge_api::api::log_habit))
        .route("/stats", get(streakforge_api::api::get_stats))
        .route("/drill", get(streakforge_api::api::get_drill))
        .route("/denial", get(streakforge_api::api::get_denial))
        .route("/lock", get(streakforge_api::api::get_lock))
        .route("/lock", post(streakforge_api::api::lock))
        .route("/unlock", post(streakforge_api::api::unlock))
        .route("/leaderboard/{period}", get(streakforge_api::api::get_leaderboard))
        // Public, like /leaderboard and /user-of-the-day: no auth extractor, so
        // the KPI page is readable by anyone, which is the point of it.
        .route("/kpi", get(streakforge_api::api::get_kpi))
        .route("/user-of-the-day", get(streakforge_api::api::get_user_of_the_day))
        .route("/total", get(streakforge_api::api::get_total))
        .route("/feed", get(streakforge_api::api::get_feed))
        .route("/profile/{username}", get(streakforge_api::api::get_profile))
        .route("/profile", patch(streakforge_api::api::update_profile))
        .route("/doctrine", get(streakforge_api::manifesto::list))
        .route("/doctrine/{id}", get(streakforge_api::manifesto::get));

    // Serve real static files from the build root (manifest, icons, etc.)
    // BEFORE the SPA fallback, so PWA assets are served as themselves.
    // Unknown paths fall back to index.html for client-side routing.
    let web_dir = cfg.web_build_dir.clone();
    let static_fallback = tower::service_fn(move |req: axum::extract::Request| {
        let web_dir = web_dir.clone();
        async move {
            // Unknown /api/* paths must 404, not serve the SPA shell (a missing
            // endpoint should not masquerade as the app; stale probing fails loud).
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
    let app = Router::new()
        .nest("/api", api_router)
        .nest_service(
            "/_app",
            ServeDir::new(format!("{}/_app", cfg.web_build_dir)),
        )
        .fallback_service(
            // Serve real static files from the build root (manifest, icons,
            // index.html) and fall back to index.html for SPA client routes.
            ServeDir::new(&cfg.web_build_dir).fallback(static_fallback),
        )
        .with_state(state)
        .layer(session_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("streakforge listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
