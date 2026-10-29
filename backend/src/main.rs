use streakforge_api::config::Config;
use streakforge_api::db;

// The HTTP surface itself lives in `streakforge_api::router::build_router` so
// integration tests can mount the real thing. This file is only startup:
// logging, config, the pool, migrations, bind, serve.
//
// SPA fallback and the unknown-/api 404 both live in the router module.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // `streakforge-api purge` — the retention job, run from cron. It is a mode of
    // the same binary rather than a second program so it ships with the code that
    // defines the windows, and so the deploy cannot install a purge job that knows
    // older tables than the binary beside it.
    if std::env::args().nth(1).as_deref() == Some("purge") {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "streakforge=info,sqlx=warn".into()),
            )
            .init();
        let cfg = Config::from_env();
        let pool = db::connect(&cfg.database_url).await?;
        db::run_migrations(&pool, &cfg.migrations_dir).await?;
        let days: i32 = std::env::var("TOMBSTONE_RETENTION_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(365);
        let removed = streakforge_api::account::purge_expired(&pool, days).await?;
        // One JSON line: cron mail and `journalctl` both read it without parsing.
        println!("{removed}");
        return Ok(());
    }

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

    let app = streakforge_api::router::build_router(&cfg, pool).await?;

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("streakforge listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
