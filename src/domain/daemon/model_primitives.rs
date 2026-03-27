use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub(crate) u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketPath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedPipeName(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PidFilePath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LockFilePath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheDir(pub(crate) PathBuf);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartBudget {
    pub max_restarts: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartAttempt {
    pub attempt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryBackoffMs {
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryJitterPercent {
    pub value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryDelayRange {
    pub min: RetryBackoffMs,
    pub max: RetryBackoffMs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaemonRetryPolicy {
    pub budget: RestartBudget,
    pub delay_range: RetryDelayRange,
    pub jitter: RetryJitterPercent,
}
