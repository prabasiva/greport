//! Rate limiting middleware

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::auth::AuthInfo;
use crate::state::AppState;

/// Rate limit state for a single client
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Number of requests in current window
    count: u32,
    /// Start of current window
    window_start: Instant,
}

/// Rate limiter state
#[derive(Debug, Default)]
pub struct RateLimiter {
    /// Map of client identifier to rate limit state
    entries: RwLock<HashMap<String, RateLimitEntry>>,
    /// Default requests per minute
    default_limit: u32,
    /// Window duration
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(default_limit: u32) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_limit,
            window: Duration::from_secs(60),
        }
    }

    /// Check if a request should be allowed
    pub async fn check(&self, client_id: &str, limit: Option<u32>) -> RateLimitResult {
        let limit = limit.unwrap_or(self.default_limit);
        let now = Instant::now();

        let mut entries = self.entries.write().await;
        let entry = entries
            .entry(client_id.to_string())
            .or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
            });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= self.window {
            entry.count = 0;
            entry.window_start = now;
        }

        // Calculate remaining
        let remaining = limit.saturating_sub(entry.count);
        let reset_at = entry.window_start + self.window;
        let reset_in = reset_at.saturating_duration_since(now);

        if entry.count >= limit {
            return RateLimitResult {
                allowed: false,
                limit,
                remaining: 0,
                reset_in_secs: reset_in.as_secs() as u32,
            };
        }

        entry.count += 1;

        RateLimitResult {
            allowed: true,
            limit,
            remaining: remaining.saturating_sub(1),
            reset_in_secs: reset_in.as_secs() as u32,
        }
    }

    /// Clean up old entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut entries = self.entries.write().await;
        entries.retain(|_, v| now.duration_since(v.window_start) < self.window * 2);
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_in_secs: u32,
}

/// Rate limit exceeded response
#[derive(Serialize)]
struct RateLimitError {
    error: RateLimitErrorBody,
}

#[derive(Serialize)]
struct RateLimitErrorBody {
    code: String,
    message: String,
    retry_after: u32,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Get client identifier
    let client_id = get_client_id(&request);

    // Get rate limit from auth info if available
    let limit = request
        .extensions()
        .get::<Arc<AuthInfo>>()
        .map(|auth| auth.rate_limit as u32);

    // Check rate limit
    let result = state.rate_limiter.check(&client_id, limit).await;

    if !result.allowed {
        let error = RateLimitError {
            error: RateLimitErrorBody {
                code: "RATE_LIMITED".to_string(),
                message: "Rate limit exceeded. Please try again later.".to_string(),
                retry_after: result.reset_in_secs,
            },
        };

        return (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("X-RateLimit-Limit", result.limit.to_string()),
                ("X-RateLimit-Remaining", "0".to_string()),
                ("X-RateLimit-Reset", result.reset_in_secs.to_string()),
                ("Retry-After", result.reset_in_secs.to_string()),
            ],
            Json(error),
        )
            .into_response();
    }

    // Add rate limit headers to response
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        result.limit.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        result.remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        result.reset_in_secs.to_string().parse().unwrap(),
    );

    response
}

/// Get client identifier from request
fn get_client_id(request: &Request) -> String {
    // First try to get from auth info
    if let Some(auth) = request.extensions().get::<Arc<AuthInfo>>() {
        return format!("user:{}", auth.owner);
    }

    // Then try X-Forwarded-For header
    if let Some(forwarded) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(ip) = forwarded.split(',').next() {
            return format!("ip:{}", ip.trim());
        }
    }

    // Then try X-Real-IP header
    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        return format!("ip:{}", real_ip);
    }

    // Fall back to connection info
    if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return format!("ip:{}", connect_info.0.ip());
    }

    // Last resort - anonymous
    "anonymous".to_string()
}

/// Start background cleanup task for rate limiter
pub fn start_cleanup_task(rate_limiter: Arc<RateLimiter>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            rate_limiter.cleanup().await;
        }
    });
}
