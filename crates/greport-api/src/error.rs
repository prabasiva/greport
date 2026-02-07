//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error type
#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    /// Resource not found
    NotFound(String),
    /// Bad request
    BadRequest(String),
    /// Unauthorized
    Unauthorized,
    /// Rate limited
    RateLimited,
    /// Internal server error
    Internal(String),
    /// GitHub API error
    GitHub(greport_core::Error),
}

/// Error response body
#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "{msg}"),
            ApiError::BadRequest(msg) => write!(f, "{msg}"),
            ApiError::Unauthorized => write!(f, "Invalid or missing authentication"),
            ApiError::RateLimited => write!(f, "Rate limit exceeded"),
            ApiError::Internal(msg) => write!(f, "{msg}"),
            ApiError::GitHub(e) => write!(f, "{}", friendly_github_message(e)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid or missing authentication".into(),
            ),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Rate limit exceeded".into(),
            ),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg),
            ApiError::GitHub(e) => {
                tracing::error!("GitHub API error: {:?}", e);
                let message = friendly_github_message(&e);
                (StatusCode::BAD_GATEWAY, "GITHUB_ERROR", message)
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
            },
        });

        (status, body).into_response()
    }
}

/// Convert a raw GitHub/octocrab error into a user-friendly message.
fn friendly_github_message(e: &greport_core::Error) -> String {
    let raw = format!("{e}");
    if raw.contains("404") || raw.contains("Not Found") {
        "Repository not found. Check that the name is correct and your token has access.".into()
    } else if raw.contains("403") || raw.contains("not accessible") {
        "Access denied. Your GitHub token does not have permission to access this resource.".into()
    } else if raw.contains("401") || raw.contains("Unauthorized") || raw.contains("Bad credentials")
    {
        "Authentication failed. Check that your GitHub token is valid and not expired.".into()
    } else if raw.contains("rate limit") || raw.contains("429") {
        "GitHub API rate limit exceeded. Wait a few minutes and try again.".into()
    } else if raw.contains("timeout") || raw.contains("timed out") {
        "Request to GitHub timed out. Check your network connection and try again.".into()
    } else if raw.contains("DNS") || raw.contains("resolve") {
        "Could not reach GitHub. Check your network connection and DNS settings.".into()
    } else {
        format!("GitHub API error: {raw}")
    }
}

impl From<greport_core::Error> for ApiError {
    fn from(err: greport_core::Error) -> Self {
        ApiError::GitHub(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ApiError::Internal(format!("Database error: {}", err))
    }
}
