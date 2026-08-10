use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub session_secret: String,
    pub public_url: String,
    pub secure_cookies: bool,
    pub web_build_dir: String,
    pub manifestos_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://streakforge:***@127.0.0.1:5432/streakforge".into()
            }),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            session_secret: env::var("SESSION_SECRET").unwrap_or_else(|_| {
                "dev-only-insecure-secret-change-me-0123456789abcdef".into()
            }),
            public_url: env::var("PUBLIC_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into()),
            secure_cookies: env::var("SECURE_COOKIES").map(|v| v == "1").unwrap_or(false),
            web_build_dir: env::var("WEB_BUILD_DIR").unwrap_or_else(|_| "./web/build".into()),
            manifestos_dir: env::var("MANIFESTOS_DIR").unwrap_or_else(|_| "./manifestos".into()),
        }
    }
}
