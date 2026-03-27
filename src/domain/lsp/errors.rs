use thiserror::Error;

use super::model::{QueryExecutionMode, VersionConstraint};

#[derive(Debug, Error)]
pub enum QueryTimeoutError {
    #[error("Query timeout must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum DepthError {
    #[error("Depth cannot be zero")]
    Zero,
    #[error("Requested depth {requested} exceeds max {maximum}")]
    TooDeep { requested: u8, maximum: u8 },
}

#[derive(Debug, Error)]
pub enum QueryExecutionPolicyError {
    #[error("Invalid query execution policy for mode {mode:?}. Interactive mode must allow partial results")]
    InteractiveModeRequiresPartialResults { mode: QueryExecutionMode },
}

#[derive(Debug, Error)]
pub enum LspRequestIdError {
    #[error("LSP request id cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum MaxInFlightLspRequestsError {
    #[error("Max in-flight LSP requests must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum LspRetryBudgetError {
    #[error("LSP retry budget must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum LspRetryBackoffError {
    #[error("LSP retry backoff must be greater than zero milliseconds")]
    Zero,
}

#[derive(Debug, Error)]
pub enum LspRetryJitterError {
    #[error("LSP retry jitter percent must be in range 0..=100")]
    OutOfRange,
}

#[derive(Debug, Error)]
pub enum LspRetryDelayRangeError {
    #[error("LSP retry delay max must be greater than or equal to min")]
    InvalidRange,
}

#[derive(Debug, Error)]
pub enum LspCompatibilityMatrixError {
    #[error("LSP compatibility matrix cannot be empty")]
    Empty,
    #[error("LSP compatibility matrix contains duplicate language-feature entries")]
    DuplicatedEntry,
}

#[derive(Debug, Error)]
pub enum RootedTraversalRequestError {
    #[error("Traversal root is required in multi-root mode. Workspace roots available: {available_roots}")]
    RootRequired { available_roots: usize },
}

#[derive(Debug, Error)]
pub enum LspCapabilityError {
    #[error("LSP server does not support required capability: {capability}")]
    MissingCapability { capability: &'static str },
}

#[derive(Debug, Error)]
pub enum LspVersionError {
    #[error("LSP version '{found}' is outside supported range")]
    UnsupportedVersion {
        found: String,
        required: VersionConstraint,
    },
}
