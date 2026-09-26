use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::migrate::Migrator;
use std::path::Path;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool, migrations_dir: &str) -> Result<(), sqlx::migrate::MigrateError> {
    let migrator = Migrator::new(Path::new(migrations_dir)).await?;
    migrator.run(pool).await
}
