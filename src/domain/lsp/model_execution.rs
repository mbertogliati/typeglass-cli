use std::path::PathBuf;
use std::time::Duration;

use crate::domain::daemon::Pid;

use super::primitives::{LspRequestId, LspRetryPolicy, MaxInFlightLspRequests, QueryTimeout};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExecutionPolicy {
    pub timeout: QueryTimeout,
    pub retry_policy: LspRetryPolicy,
    pub allow_partial_results: bool,
    pub restart_on_unverifiable_identity: bool,
    pub max_in_flight_requests: MaxInFlightLspRequests,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryExecutionMode {
    SingleShot,
    InteractiveSession,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResultMeta {
    pub mode: QueryExecutionMode,
    pub retry_count: u8,
    pub partial: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspFailureKind {
    Timeout,
    ProcessDied,
    InvalidResponse,
    WorkspaceError,
    SymbolNotFound,
    FileNotReadable,
    Cancelled,
    Backpressure,
}

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("LSP request timed out after {after:?}")]
    Timeout { after: Duration },
    #[error("LSP process {pid:?} terminated unexpectedly")]
    ProcessDied { pid: Pid, exit_code: Option<i32> },
    #[error("LSP returned invalid response. Raw payload: {raw}")]
    InvalidResponse { raw: String },
    #[error("LSP workspace error: {message}")]
    WorkspaceError { message: String },
    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },
    #[error("File not readable: {path}")]
    FileNotReadable { path: PathBuf },
    #[error("LSP request was cancelled. Request id: {request_id:?}")]
    Cancelled { request_id: LspRequestId },
    #[error("LSP backpressure: too many in-flight requests. Limit: {limit:?}")]
    Backpressure { limit: MaxInFlightLspRequests },
}
