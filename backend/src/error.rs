use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    /// Seconds the caller should wait, sent as `Retry-After` when set.
    ///
    /// The site's messages already name the clock ("you can waste again at
    /// 00:00Z") because a 429 with no time in it reads as a broken button. A
    /// header is the machine-readable half of that same promise, and it is what
    /// a client or a proxy is entitled to act on.
    pub retry_after: Option<i64>,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            retry_after: None,
        }
    }
    /// Attach a `Retry-After` (in seconds) to this error.
    pub fn with_retry_after(mut self, secs: i64) -> Self {
        self.retry_after = Some(secs.max(0));
        self
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }
    pub fn too_many_requests(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, msg)
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = (self.status, Json(json!({ "error": self.message }))).into_response();
        if let Some(secs) = self.retry_after {
            if let Ok(v) = HeaderValue::from_str(&secs.to_string()) {
                res.headers_mut().insert(header::RETRY_AFTER, v);
            }
        }
        res
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("sqlx error: {e}");
        match e {
            sqlx::Error::RowNotFound => Self::not_found("Not found"),
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                Self::conflict("That username is already taken")
            }
            _ => Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!("redis error: {e}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!("json error: {e}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

impl From<tower_sessions::session::Error> for ApiError {
    fn from(e: tower_sessions::session::Error) -> Self {
        tracing::error!("session error: {e}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
