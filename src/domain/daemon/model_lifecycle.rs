use super::primitives::{LockFilePath, Pid, PidFilePath};
use super::state::{DaemonRestartReason, DaemonState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonMode {
    Persistent,
    OneShot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonProcessFiles {
    pub pid_file: PidFilePath,
    pub lock_file: LockFilePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonReuseDecision {
    Reuse,
    Restart { reason: DaemonRestartReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonLifecycleCommand {
    Start,
    Stop,
    Status,
    Gc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonLifecycleEvent {
    Started { pid: Pid },
    Stopped { pid: Pid },
    Restarted { previous: Pid, current: Pid },
    CleanedStaleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonTransition {
    pub from: DaemonState,
    pub command: DaemonLifecycleCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheDirCandidate {
    UserCache(std::path::PathBuf),
    TempDir(std::path::PathBuf),
    Explicit(std::path::PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheDirResolution {
    pub selected: super::primitives::CacheDir,
    pub attempted: Vec<CacheDirCandidate>,
}
