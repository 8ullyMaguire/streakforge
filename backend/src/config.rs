use std::env;

/// Session lifetime for an inactive user, in days. A signed-in whiteboi stays
/// signed in for this long without touching the site.
pub const DEFAULT_SESSION_TTL_DAYS: i64 = 30;
/// How many loads wasted (`kind = 'habit'`) one user may log per UTC day.
pub const DEFAULT_WLW_DAILY_LIMIT: i64 = 5;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub session_secret: String,
    pub public_url: String,
    pub secure_cookies: bool,
    pub web_build_dir: String,
    pub manifestos_dir: String,
    pub migrations_dir: String,
    /// Days of inactivity before a session expires. `SESSION_TTL_DAYS`.
    pub session_ttl_days: i64,
    /// Loads wasted allowed per UTC day per user. `WLW_DAILY_LIMIT`.
    pub wlw_daily_limit: i64,
}

/// Parse a bounded positive integer out of an env value.
///
/// Split out from the env read so the clamping rule is testable without
/// touching the process environment.
///
/// A value outside `lo..=hi` (or not a number at all) falls back to the default
/// AND is logged, because the failure mode is silent and total: `WLW_DAILY_LIMIT=0`
/// or a typo like `WLW_DAILY_LIMIT=five` would otherwise become a limit that
/// locks every user out of logging, with no error anywhere to say why.
pub fn bounded_i64(name: &str, raw: Option<&str>, default: i64, lo: i64, hi: i64) -> i64 {
    let Some(raw) = raw else { return default };
    match raw.trim().parse::<i64>() {
        Ok(n) if (lo..=hi).contains(&n) => n,
        Ok(n) => {
            tracing::warn!("{name}={n} is outside {lo}..={hi}; using {default}");
            default
        }
        Err(_) => {
            tracing::warn!("{name}={raw:?} is not an integer; using {default}");
            default
        }
    }
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
            migrations_dir: env::var("MIGRATIONS_DIR").unwrap_or_else(|_| "./migrations".into()),
            session_ttl_days: bounded_i64(
                "SESSION_TTL_DAYS",
                env::var("SESSION_TTL_DAYS").ok().as_deref(),
                DEFAULT_SESSION_TTL_DAYS,
                1,
                365,
            ),
            wlw_daily_limit: bounded_i64(
                "WLW_DAILY_LIMIT",
                env::var("WLW_DAILY_LIMIT").ok().as_deref(),
                DEFAULT_WLW_DAILY_LIMIT,
                1,
                100,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::bounded_i64;

    #[test]
    fn bounded_i64_uses_the_default_when_unset() {
        assert_eq!(bounded_i64("X", None, 5, 1, 100), 5);
    }

    #[test]
    fn bounded_i64_accepts_a_value_in_range() {
        assert_eq!(bounded_i64("X", Some("3"), 5, 1, 100), 3);
        assert_eq!(bounded_i64("X", Some("  7  "), 5, 1, 100), 7);
        assert_eq!(bounded_i64("X", Some("1"), 5, 1, 100), 1);
        assert_eq!(bounded_i64("X", Some("100"), 5, 1, 100), 100);
    }

    #[test]
    fn bounded_i64_refuses_out_of_range_and_garbage() {
        // The point of the clamp: neither of these may become a live limit of 0.
        assert_eq!(bounded_i64("X", Some("0"), 5, 1, 100), 5);
        assert_eq!(bounded_i64("X", Some("-3"), 5, 1, 100), 5);
        assert_eq!(bounded_i64("X", Some("101"), 5, 1, 100), 5);
        assert_eq!(bounded_i64("X", Some("five"), 5, 1, 100), 5);
        assert_eq!(bounded_i64("X", Some(""), 5, 1, 100), 5);
    }
}
