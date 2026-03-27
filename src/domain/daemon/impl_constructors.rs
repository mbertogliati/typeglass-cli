use std::path::{Path, PathBuf};

use crate::domain::daemon::{
    CacheDir, CacheDirError, DaemonMode, DaemonProcessFiles, DaemonProcessFilesError, LockFilePath,
    NamedPipeName, NamedPipeNameError, Pid, PidError, PidFilePath, RestartAttempt, RestartBudget,
    RestartBudgetError, RetryBackoffError, RetryBackoffMs, RetryDelayRange, RetryDelayRangeError,
    RetryJitterError, RetryJitterPercent, SocketPath, SocketPathError,
};

impl Pid {
    pub fn new(value: u32) -> Result<Self, PidError> {
        if value == 0 {
            return Err(PidError::Zero);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl SocketPath {
    pub fn new(path: PathBuf) -> Result<Self, SocketPathError> {
        if path.as_os_str().is_empty() {
            return Err(SocketPathError::Empty);
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl NamedPipeName {
    pub fn new(value: String) -> Result<Self, NamedPipeNameError> {
        if value.trim().is_empty() {
            return Err(NamedPipeNameError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl DaemonProcessFiles {
    pub fn new(pid_file: PathBuf, lock_file: PathBuf) -> Result<Self, DaemonProcessFilesError> {
        if pid_file.as_os_str().is_empty() {
            return Err(DaemonProcessFilesError::EmptyPidFile);
        }
        if lock_file.as_os_str().is_empty() {
            return Err(DaemonProcessFilesError::EmptyLockFile);
        }
        Ok(Self {
            pid_file: PidFilePath(pid_file),
            lock_file: LockFilePath(lock_file),
        })
    }
}

impl CacheDir {
    pub fn new(path: PathBuf) -> Result<Self, CacheDirError> {
        if path.as_os_str().is_empty() {
            return Err(CacheDirError::EmptyPath);
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl DaemonMode {
    pub const fn should_persist(self) -> bool {
        matches!(self, Self::Persistent)
    }
}

impl From<crate::domain::workspace::RuntimeEnvironment> for DaemonMode {
    fn from(value: crate::domain::workspace::RuntimeEnvironment) -> Self {
        match value {
            crate::domain::workspace::RuntimeEnvironment::Local => Self::Persistent,
            crate::domain::workspace::RuntimeEnvironment::Ci
            | crate::domain::workspace::RuntimeEnvironment::Container
            | crate::domain::workspace::RuntimeEnvironment::NoTty => Self::OneShot,
        }
    }
}

impl RestartBudget {
    pub fn new(max_restarts: u8) -> Result<Self, RestartBudgetError> {
        if max_restarts == 0 {
            return Err(RestartBudgetError::Zero);
        }
        Ok(Self { max_restarts })
    }

    pub fn validate_attempt(self, attempt: RestartAttempt) -> Result<(), RestartBudgetError> {
        if attempt.attempt > self.max_restarts {
            return Err(RestartBudgetError::Exhausted {
                attempt: attempt.attempt,
                max: self.max_restarts,
            });
        }
        Ok(())
    }
}

impl RestartAttempt {
    pub const fn new(attempt: u8) -> Self {
        Self { attempt }
    }
}

impl RetryBackoffMs {
    pub fn new(value: u64) -> Result<Self, RetryBackoffError> {
        if value == 0 {
            return Err(RetryBackoffError::Zero);
        }
        Ok(Self { value })
    }
}

impl RetryJitterPercent {
    pub fn new(value: u8) -> Result<Self, RetryJitterError> {
        if value > 100 {
            return Err(RetryJitterError::OutOfRange);
        }
        Ok(Self { value })
    }
}

impl RetryDelayRange {
    pub fn new(min: RetryBackoffMs, max: RetryBackoffMs) -> Result<Self, RetryDelayRangeError> {
        if max.value < min.value {
            return Err(RetryDelayRangeError::InvalidRange);
        }
        Ok(Self { min, max })
    }
}
