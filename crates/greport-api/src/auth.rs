//! API authentication and authorization

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::error::ApiError;
use crate::state::AppState;

/// Hash an API key for storage/lookup
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Authenticated user info extracted from request
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthInfo {
    pub owner: String,
    pub scopes: Vec<String>,
    pub rate_limit: i32,
}

/// Authentication middleware
/// Extracts and validates API key from Authorization header
#[allow(dead_code)]
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Get Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err(ApiError::Unauthorized);
        }
    };

    // Check if it's a GitHub token (passthrough mode)
    if token.starts_with("ghp_") || token.starts_with("gho_") || token.starts_with("github_pat_") {
        // GitHub token - allow through without DB lookup
        let auth_info = AuthInfo {
            owner: "github_user".to_string(),
            scopes: vec!["read".to_string()],
            rate_limit: 60,
        };
        request.extensions_mut().insert(Arc::new(auth_info));
        return Ok(next.run(request).await);
    }

    // Hash the API key and look it up
    let key_hash = hash_api_key(token);

    // Only validate if we have a database connection
    if let Some(ref pool) = state.db {
        match greport_db::queries::get_api_key_by_hash(pool, &key_hash).await {
            Ok(Some(api_key)) => {
                // Update last used timestamp
                let _ = greport_db::queries::update_api_key_last_used(pool, api_key.id).await;

                let auth_info = AuthInfo {
                    owner: api_key.owner,
                    scopes: api_key.scopes,
                    rate_limit: api_key.rate_limit,
                };
                request.extensions_mut().insert(Arc::new(auth_info));
            }
            Ok(None) => {
                return Err(ApiError::Unauthorized);
            }
            Err(e) => {
                tracing::error!("Database error during auth: {:?}", e);
                return Err(ApiError::Internal(
                    "Authentication service unavailable".to_string(),
                ));
            }
        }
    } else {
        // No database - reject API keys (only GitHub tokens allowed)
        return Err(ApiError::Unauthorized);
    }

    Ok(next.run(request).await)
}

/// Optional auth middleware - allows unauthenticated requests but extracts auth info if present
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Get Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(h) = auth_header {
        if let Some(token) = h.strip_prefix("Bearer ") {
            // Check if it's a GitHub token
            if token.starts_with("ghp_")
                || token.starts_with("gho_")
                || token.starts_with("github_pat_")
            {
                let auth_info = AuthInfo {
                    owner: "github_user".to_string(),
                    scopes: vec!["read".to_string()],
                    rate_limit: 60,
                };
                request.extensions_mut().insert(Arc::new(auth_info));
            } else if let Some(ref pool) = state.db {
                let key_hash = hash_api_key(token);
                if let Ok(Some(api_key)) =
                    greport_db::queries::get_api_key_by_hash(pool, &key_hash).await
                {
                    let _ = greport_db::queries::update_api_key_last_used(pool, api_key.id).await;
                    let auth_info = AuthInfo {
                        owner: api_key.owner,
                        scopes: api_key.scopes,
                        rate_limit: api_key.rate_limit,
                    };
                    request.extensions_mut().insert(Arc::new(auth_info));
                }
            }
        }
    }

    next.run(request).await
}

/// Extract auth info from request extensions
#[allow(dead_code)]
pub fn get_auth_info(request: &Request) -> Option<Arc<AuthInfo>> {
    request.extensions().get::<Arc<AuthInfo>>().cloned()
}

/// Check if user has required scope
#[allow(dead_code)]
pub fn has_scope(auth_info: &AuthInfo, required_scope: &str) -> bool {
    auth_info
        .scopes
        .iter()
        .any(|s| s == required_scope || s == "*")
}
