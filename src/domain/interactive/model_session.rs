use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSizeLimit {
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdleTimeoutMs {
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseSizeLimit {
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConcurrencyPolicy {
    pub allow_parallel_requests: bool,
    pub max_in_flight_requests: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractiveSessionPolicy {
    pub response_size_limit: ResponseSizeLimit,
    pub input_size_limit: InputSizeLimit,
    pub idle_timeout: IdleTimeoutMs,
    pub concurrency: SessionConcurrencyPolicy,
    pub keep_alive_on_parse_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPhase {
    PreInit,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionAction {
    KeepAlive,
    Terminate(SessionTermination),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractiveShutdownReason {
    ExitRequest,
    StdinClosed,
    IdleTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionTermination {
    ExitCommand,
    StdinClosed,
    IdleTimeout,
}
