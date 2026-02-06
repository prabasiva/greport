//! Retry logic for transient network errors

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Default retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// Check if an error is retryable (transient network error)
#[allow(dead_code)]
pub fn is_retryable_error(err: &octocrab::Error) -> bool {
    match err {
        // Network/connection errors are retryable
        octocrab::Error::Http { source, .. } => {
            let err_str = source.to_string().to_lowercase();
            err_str.contains("connection")
                || err_str.contains("timeout")
                || err_str.contains("reset")
                || err_str.contains("broken pipe")
                || err_str.contains("temporarily unavailable")
                || err_str.contains("eof")
        }
        // Service errors (5xx) are retryable
        octocrab::Error::GitHub { source, .. } => {
            let status = source.status_code.as_u16();
            (500..600).contains(&status) // 5xx errors
                || status == 429 // Too Many Requests
                || status == 408 // Request Timeout
        }
        // Other errors are generally not retryable
        _ => false,
    }
}

/// Execute an async operation with retry logic
#[allow(dead_code)]
pub async fn with_retry<F, Fut, T>(
    operation_name: &str,
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, octocrab::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, octocrab::Error>>,
{
    let mut attempt = 0;
    let mut backoff = config.initial_backoff;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt > config.max_retries || !is_retryable_error(&err) {
                    if attempt > 1 {
                        warn!(
                            operation = operation_name,
                            attempts = attempt,
                            error = %err,
                            "Operation failed after retries"
                        );
                    }
                    return Err(err);
                }

                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    error = %err,
                    retry_after = ?backoff,
                    "Transient error, will retry"
                );

                sleep(backoff).await;

                // Exponential backoff with cap
                backoff = Duration::from_secs_f64(
                    (backoff.as_secs_f64() * config.multiplier)
                        .min(config.max_backoff.as_secs_f64()),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff, Duration::from_millis(500));
        assert_eq!(config.max_backoff, Duration::from_secs(30));
        assert!((config.multiplier - 2.0).abs() < f64::EPSILON);
    }
}
