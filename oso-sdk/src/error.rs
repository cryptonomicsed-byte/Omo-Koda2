use thiserror::Error;

pub type SdkResult<T> = Result<T, SdkError>;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("job not found: {0}")]
    JobNotFound(String),
    #[error("job failed: {0}")]
    JobFailed(String),
    #[error("contract not found: {0}")]
    ContractNotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("compile error: {0}")]
    Compile(#[from] oso_parser::OsoError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("timeout waiting for proof")]
    ProofTimeout,
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
}
