use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::domain::language::Language;

use super::model::{CacheDir, DaemonLifecycleCommand, NamedPipeName, Pid, SocketPath};

#[derive(Debug, Error)]
pub enum RestartBudgetError {
    #[error("Restart budget cannot be zero")]
    Zero,
    #[error("Restart attempt {attempt} exceeded max restarts {max}")]
    Exhausted { attempt: u8, max: u8 },
}

#[derive(Debug, Error)]
pub enum RetryBackoffError {
    #[error("Retry backoff must be greater than zero milliseconds")]
    Zero,
}

#[derive(Debug, Error)]
pub enum RetryJitterError {
    #[error("Retry jitter percent must be in range 0..=100")]
    OutOfRange,
}

#[derive(Debug, Error)]
pub enum RetryDelayRangeError {
    #[error("Retry delay max must be greater than or equal to min")]
    InvalidRange,
}

#[derive(Debug, Error)]
pub enum NamedPipeNameError {
    #[error("Named pipe cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum DaemonProcessFilesError {
    #[error("PID file path cannot be empty")]
    EmptyPidFile,
    #[error("Lock file path cannot be empty")]
    EmptyLockFile,
}

#[derive(Debug, Error)]
pub enum DaemonTransitionError {
    #[error("Invalid daemon transition. Command: {command:?}. Daemon is already running")]
    AlreadyRunning { command: DaemonLifecycleCommand },
    #[error("Invalid daemon transition. Command: {command:?}. Daemon is not running")]
    NotRunning { command: DaemonLifecycleCommand },
    #[error("Invalid daemon transition. Command: {command:?}. GC is not valid in one-shot mode")]
    GcUnsupportedForOneShot { command: DaemonLifecycleCommand },
}

#[derive(Debug, Error)]
pub enum PidError {
    #[error("PID cannot be zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum SocketPathError {
    #[error("Socket path cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum CacheDirError {
    #[error("Cache directory path cannot be empty")]
    EmptyPath,
    #[error("Cache directory is not writable: {path}. Reason: {reason}")]
    NotWritable { path: PathBuf, reason: io::Error },
}

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Daemon is already running with PID {pid:?}")]
    AlreadyRunning { pid: Pid },
    #[error("Failed to create daemon socket at {path:?}. Reason: {reason}")]
    SocketCreationFailed { path: SocketPath, reason: io::Error },
    #[error("Failed to create daemon named pipe '{name:?}'. Reason: {reason}")]
    NamedPipeCreationFailed {
        name: NamedPipeName,
        reason: io::Error,
    },
    #[error("No LSP binary found for language {language:?}")]
    LspNotFound { language: Language },
    #[error("Wrong LSP version for {language:?}. Found '{found}', expected '{expected}'")]
    LspWrongVersion {
        language: Language,
        found: String,
        expected: String,
    },
    #[error("Failed to start LSP process for language {language:?}. Reason: {reason}")]
    LspStartFailed {
        language: Language,
        reason: io::Error,
    },
    #[error("Failed to acquire daemon lock. Reason: {reason}")]
    LockAcquisitionFailed { reason: io::Error },
    #[error("Cache directory is not writable: {path:?}. Reason: {reason}")]
    CacheDirNotWritable { path: CacheDir, reason: io::Error },
    #[error("Detected stale PID file for PID {pid:?} at {path}")]
    StalePidFile { pid: Pid, path: PathBuf },
    #[error("Daemon socket is unresponsive for PID {pid:?} at {socket_path:?}")]
    UnresponsiveSocket { pid: Pid, socket_path: SocketPath },
    #[error("Daemon named pipe is unresponsive for PID {pid:?} at {pipe_name:?}")]
    UnresponsiveNamedPipe { pid: Pid, pipe_name: NamedPipeName },
}
