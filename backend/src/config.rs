use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub session_secret: String,
    pub x_client_id: Option<String>,
    pub x_client_secret: Option<String>,
    pub public_url: String,
    pub allow_dev_login: bool,
    pub dev_login_secret: Option<String>,
    pub secure_cookies: bool,
    pub web_build_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge".into()
            }),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            session_secret: env::var("SESSION_SECRET").unwrap_or_else(|_| {
                "dev-only-insecure-secret-change-me-0123456789abcdef".into()
            }),
            x_client_id: env::var("X_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            x_client_secret: env::var("X_CLIENT_SECRET").ok().filter(|s| !s.is_empty()),
            public_url: env::var("PUBLIC_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into()),
            allow_dev_login: env::var("ALLOW_DEV_LOGIN").map(|v| v == "1").unwrap_or(true),
            dev_login_secret: env::var("DEV_LOGIN_SECRET").ok().filter(|s| !s.is_empty()),
            secure_cookies: env::var("SECURE_COOKIES").map(|v| v == "1").unwrap_or(false),
            web_build_dir: env::var("WEB_BUILD_DIR").unwrap_or_else(|_| "./web/build".into()),
        }
    }
}
