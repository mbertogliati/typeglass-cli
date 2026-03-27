use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct LspConnection {
    _private: (),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspBinary {
    pub executable: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LspRequestId(pub(crate) String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxInFlightLspRequests {
    pub value: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRetryBudget {
    pub max_attempts: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRetryBackoffMs {
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRetryJitterPercent {
    pub value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRetryDelayRange {
    pub min: LspRetryBackoffMs,
    pub max: LspRetryBackoffMs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRetryPolicy {
    pub budget: LspRetryBudget,
    pub delay_range: LspRetryDelayRange,
    pub jitter: LspRetryJitterPercent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Depth(pub(crate) u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryTimeout(pub Duration);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retryability {
    Retryable,
    NonRetryable,
}
