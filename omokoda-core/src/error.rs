use serde::{Deserialize, Serialize};
use std::fmt;

/// Provider error taxonomy with retryability signal and structured backoff.
/// Rich error variants so callers never need to inspect error strings to decide whether to retry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiError {
    /// HTTP 429 / provider rate limit — always retryable
    RateLimit {
        message: String,
        retry_after_secs: Option<u64>,
    },
    /// HTTP 5xx — retryable (transient server fault)
    ServerError { status: u16, message: String },
    /// HTTP 408 / network timeout — retryable
    Timeout { elapsed_ms: u64 },
    /// HTTP 4xx (not 408/429) — not retryable (caller error)
    ClientError { status: u16, message: String },
    /// Authentication failure — not retryable without credential change
    Unauthorized { message: String },
    /// Context window exceeded — not retryable with same input
    ContextLengthExceeded { max_tokens: u32, sent_tokens: u32 },
    /// Provider returned malformed JSON/response
    ParseError { message: String },
    /// Tool execution error (non-provider)
    ToolError { tool: String, message: String },
    /// Network-level connectivity failure — retryable
    NetworkError { message: String },
    /// Overload / service unavailable — retryable
    Overloaded { message: String },
}

impl ApiError {
    /// True if the request should be retried with exponential backoff.
    /// Callers MUST respect this signal — retrying non-retryable errors wastes quota.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimit { .. }
                | Self::ServerError { .. }
                | Self::Timeout { .. }
                | Self::NetworkError { .. }
                | Self::Overloaded { .. }
        )
    }

    /// Suggested delay before the next retry, in milliseconds.
    /// Returns `None` for non-retryable errors.
    #[must_use]
    pub fn retry_delay_ms(&self, attempt: u32, config: &BackoffConfig) -> Option<u64> {
        if !self.is_retryable() {
            return None;
        }
        // Honour provider-supplied retry-after header when present
        if let Self::RateLimit {
            retry_after_secs: Some(secs),
            ..
        } = self
        {
            return Some(secs.saturating_mul(1000));
        }
        Some(config.delay_for_attempt(attempt))
    }

    /// Human-readable error message independent of variant.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::RateLimit { message, .. } => message,
            Self::ServerError { message, .. } => message,
            Self::Timeout { .. } => "request timed out",
            Self::ClientError { message, .. } => message,
            Self::Unauthorized { message } => message,
            Self::ContextLengthExceeded { .. } => "context length exceeded",
            Self::ParseError { message } => message,
            Self::ToolError { message, .. } => message,
            Self::NetworkError { message } => message,
            Self::Overloaded { message } => message,
        }
    }

    /// Parse an HTTP status + body into the most specific `ApiError` variant.
    #[must_use]
    pub fn from_http(status: u16, body: &str) -> Self {
        match status {
            401 | 403 => Self::Unauthorized {
                message: body.to_string(),
            },
            408 => Self::Timeout { elapsed_ms: 0 },
            429 => {
                // Try to extract retry-after from body JSON
                let retry_after = serde_json::from_str::<serde_json::Value>(body)
                    .ok()
                    .and_then(|v| v["retry_after"].as_u64());
                Self::RateLimit {
                    message: body.to_string(),
                    retry_after_secs: retry_after,
                }
            }
            529 => Self::Overloaded {
                message: body.to_string(),
            },
            400..=499 => Self::ClientError {
                status,
                message: body.to_string(),
            },
            500..=599 => Self::ServerError {
                status,
                message: body.to_string(),
            },
            _ => Self::NetworkError {
                message: format!("unexpected status {}: {}", status, body),
            },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ApiError {}

/// Exponential backoff configuration for retrying `ApiError`s.
#[derive(Debug, Clone, PartialEq)]
pub struct BackoffConfig {
    /// Base delay in milliseconds
    pub base_ms: u64,
    /// Maximum delay cap in milliseconds
    pub max_ms: u64,
    /// Jitter factor (0.0 = no jitter, 1.0 = ±100%)
    pub jitter: f64,
    /// Maximum number of attempts before giving up
    pub max_attempts: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_ms: 1000,
            max_ms: 32_000,
            jitter: 0.1,
            max_attempts: 5,
        }
    }
}

impl BackoffConfig {
    /// Compute the delay for attempt number `n` (1-based) using truncated exponential backoff.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        // Guard against overflow: cap the shift at 20 bits
        let shift = attempt.saturating_sub(1).min(20);
        let raw = self.base_ms.saturating_mul(1u64 << shift);
        raw.min(self.max_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_is_retryable() {
        let e = ApiError::RateLimit {
            message: "too many requests".to_string(),
            retry_after_secs: None,
        };
        assert!(e.is_retryable());
    }

    #[test]
    fn client_error_is_not_retryable() {
        let e = ApiError::ClientError {
            status: 400,
            message: "bad request".to_string(),
        };
        assert!(!e.is_retryable());
        assert_eq!(e.retry_delay_ms(1, &BackoffConfig::default()), None);
    }

    #[test]
    fn from_http_429_maps_to_rate_limit() {
        let e = ApiError::from_http(429, r#"{"retry_after":5}"#);
        assert!(matches!(
            e,
            ApiError::RateLimit {
                retry_after_secs: Some(5),
                ..
            }
        ));
        assert!(e.is_retryable());
    }

    #[test]
    fn from_http_500_maps_to_server_error() {
        let e = ApiError::from_http(500, "internal server error");
        assert!(matches!(e, ApiError::ServerError { status: 500, .. }));
        assert!(e.is_retryable());
    }

    #[test]
    fn backoff_doubles_each_attempt() {
        let cfg = BackoffConfig {
            base_ms: 1000,
            max_ms: 32_000,
            jitter: 0.0,
            max_attempts: 5,
        };
        assert_eq!(cfg.delay_for_attempt(1), 1000);
        assert_eq!(cfg.delay_for_attempt(2), 2000);
        assert_eq!(cfg.delay_for_attempt(3), 4000);
        assert_eq!(cfg.delay_for_attempt(4), 8000);
        assert_eq!(cfg.delay_for_attempt(5), 16_000);
    }

    #[test]
    fn backoff_caps_at_max() {
        let cfg = BackoffConfig {
            base_ms: 1000,
            max_ms: 5_000,
            jitter: 0.0,
            max_attempts: 10,
        };
        assert_eq!(cfg.delay_for_attempt(10), 5_000);
    }

    #[test]
    fn context_exceeded_not_retryable() {
        let e = ApiError::ContextLengthExceeded {
            max_tokens: 200_000,
            sent_tokens: 210_000,
        };
        assert!(!e.is_retryable());
    }

    #[test]
    fn rate_limit_with_retry_after_uses_it() {
        let e = ApiError::RateLimit {
            message: "slow down".to_string(),
            retry_after_secs: Some(10),
        };
        let delay = e.retry_delay_ms(1, &BackoffConfig::default()).unwrap();
        assert_eq!(delay, 10_000);
    }
}
