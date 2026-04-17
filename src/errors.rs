//! Typed CLI errors with codes agents can branch on.
//!
//! Code taxonomy:
//! - `AUTH_MISSING`   — no API key / project ID configured
//! - `AUTH_INVALID`   — API key rejected by PostHog (401/403)
//! - `NOT_FOUND`      — resource does not exist (404)
//! - `RATE_LIMITED`   — rate limit hit and retries exhausted (429)
//! - `VALIDATION`     — bad input (4xx other than the above, or local CLI validation)
//! - `API_ERROR`      — server-side or network failure (5xx, transport errors, fallback)

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(dead_code)]
pub enum ErrorCode {
    AuthMissing,
    AuthInvalid,
    NotFound,
    RateLimited,
    Validation,
    ApiError,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct PostHogError {
    pub message: String,
    pub code: ErrorCode,
    pub hint: Option<String>,
    pub docs_url: Option<String>,
    /// HTTP status code, when the error originated from an API response.
    pub status: Option<u16>,
}

impl PostHogError {
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            hint: None,
            docs_url: None,
            status: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Classify an HTTP status into an error code. Mirrors TS `classifyHttpStatus`.
/// Exported for use by the API client in M2.
#[allow(dead_code)]
pub fn classify_http_status(status: u16) -> ErrorCode {
    match status {
        401 | 403 => ErrorCode::AuthInvalid,
        404 => ErrorCode::NotFound,
        429 => ErrorCode::RateLimited,
        s if (400..500).contains(&s) => ErrorCode::Validation,
        _ => ErrorCode::ApiError,
    }
}

impl From<std::io::Error> for PostHogError {
    fn from(e: std::io::Error) -> Self {
        PostHogError::new(format!("I/O error: {}", e), ErrorCode::ApiError)
    }
}

impl From<reqwest::Error> for PostHogError {
    fn from(e: reqwest::Error) -> Self {
        PostHogError::new(format!("Network error: {}", e), ErrorCode::ApiError)
    }
}
