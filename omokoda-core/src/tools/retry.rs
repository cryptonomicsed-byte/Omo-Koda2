use crate::error::{ApiError, BackoffConfig};
use std::future::Future;

/// Outcome of a single retry attempt.
#[derive(Debug)]
pub enum RetryOutcome<T> {
    Ok(T),
    /// The operation failed but is retryable — will retry after delay
    Retry(ApiError),
    /// The operation failed with a non-retryable error — abort immediately
    Fatal(ApiError),
}

/// Run an async operation with exponential backoff, guided by `ApiError::is_retryable()`.
///
/// `operation` is called up to `config.max_attempts` times.
/// On a retryable failure, sleeps for `config.delay_for_attempt(n)` before retrying.
/// On a fatal (non-retryable) error or exhaustion, returns the final `ApiError`.
///
/// Called FROM WITHIN `think` (provider calls) and `act` (tool network calls).
/// Never retries on success.
pub async fn with_retry<T, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) if err.is_retryable() => {
                let delay_ms = config.backoff.delay_for_attempt(attempt);
                last_error = Some(err);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            Err(err) => return Err(err),
        }
    }

    Err(last_error.unwrap_or_else(|| ApiError::NetworkError { message: "retry exhausted".to_string() }))
}

/// Configuration for retry behaviour.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum total attempts (including the first). Defaults to 4.
    pub max_attempts: u32,
    /// Backoff configuration for per-attempt sleep durations.
    pub backoff: BackoffConfig,
}

impl RetryConfig {
    /// Standard provider retry: 4 attempts, 1s base, 30s cap.
    #[must_use]
    pub fn provider() -> Self {
        Self {
            max_attempts: 4,
            backoff: BackoffConfig {
                base_ms: 1000,
                max_ms: 30_000,
                jitter: 0.1,
                max_attempts: 4,
            },
        }
    }

    /// Fast tool retry: 3 attempts, 200ms base (for transient IO errors).
    #[must_use]
    pub fn tool() -> Self {
        Self {
            max_attempts: 3,
            backoff: BackoffConfig {
                base_ms: 200,
                max_ms: 5_000,
                jitter: 0.05,
                max_attempts: 3,
            },
        }
    }

    /// Single attempt — effectively disables retry.
    #[must_use]
    pub fn once() -> Self {
        Self {
            max_attempts: 1,
            backoff: BackoffConfig {
                base_ms: 0,
                max_ms: 0,
                jitter: 0.0,
                max_attempts: 1,
            },
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::provider()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn retryable_err() -> ApiError {
        ApiError::NetworkError { message: "transient".to_string() }
    }

    fn fatal_err() -> ApiError {
        ApiError::ClientError { status: 400, message: "bad request".to_string() }
    }

    #[tokio::test]
    async fn succeeds_on_first_attempt() {
        let config = RetryConfig::once();
        let result: Result<u32, ApiError> =
            with_retry(&config, || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn retries_then_succeeds() {
        let call_count = Arc::new(Mutex::new(0u32));
        let config = RetryConfig {
            max_attempts: 4,
            backoff: BackoffConfig { base_ms: 1, max_ms: 10, jitter: 0.0, max_attempts: 4 },
        };
        let cc = call_count.clone();
        let result: Result<u32, ApiError> = with_retry(&config, || {
            let cc = cc.clone();
            async move {
                let mut n = cc.lock().unwrap();
                *n += 1;
                if *n < 3 {
                    Err(retryable_err())
                } else {
                    Ok(*n)
                }
            }
        })
        .await;
        assert!(result.is_ok());
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn fatal_error_does_not_retry() {
        let call_count = Arc::new(Mutex::new(0u32));
        let config = RetryConfig {
            max_attempts: 4,
            backoff: BackoffConfig { base_ms: 1, max_ms: 10, jitter: 0.0, max_attempts: 4 },
        };
        let cc = call_count.clone();
        let result: Result<u32, ApiError> = with_retry(&config, || {
            let cc = cc.clone();
            async move {
                *cc.lock().unwrap() += 1;
                Err(fatal_err())
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(*call_count.lock().unwrap(), 1, "fatal errors must not retry");
    }

    #[tokio::test]
    async fn exhausts_attempts_on_persistent_retryable() {
        let call_count = Arc::new(Mutex::new(0u32));
        let config = RetryConfig {
            max_attempts: 3,
            backoff: BackoffConfig { base_ms: 1, max_ms: 5, jitter: 0.0, max_attempts: 3 },
        };
        let cc = call_count.clone();
        let result: Result<u32, ApiError> = with_retry(&config, || {
            let cc = cc.clone();
            async move {
                *cc.lock().unwrap() += 1;
                Err(retryable_err())
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[test]
    fn retry_config_provider_defaults() {
        let c = RetryConfig::provider();
        assert_eq!(c.max_attempts, 4);
        assert_eq!(c.backoff.base_ms, 1000);
    }

    #[test]
    fn retry_config_tool_is_faster() {
        let tool = RetryConfig::tool();
        let provider = RetryConfig::provider();
        assert!(tool.backoff.base_ms < provider.backoff.base_ms);
    }
}
