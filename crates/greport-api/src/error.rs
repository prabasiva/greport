//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error type
#[derive(Debug)]
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
                (StatusCode::BAD_GATEWAY, "GITHUB_ERROR", e.to_string())
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
