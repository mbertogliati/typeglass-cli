use crate::domain::contracts::{MachineCode, UserMessage};

use crate::domain::daemon::DaemonError;

impl UserMessage for DaemonError {
    fn user_message(&self) -> String {
        self.to_string()
    }
}

impl MachineCode for DaemonError {
    fn code(&self) -> &'static str {
        match self {
            DaemonError::AlreadyRunning { .. } => "DAEMON_ALREADY_RUNNING",
            DaemonError::SocketCreationFailed { .. } => "DAEMON_SOCKET_CREATE_FAILED",
            DaemonError::NamedPipeCreationFailed { .. } => "DAEMON_PIPE_CREATE_FAILED",
            DaemonError::LspNotFound { .. } => "DAEMON_LSP_NOT_FOUND",
            DaemonError::LspWrongVersion { .. } => "DAEMON_LSP_WRONG_VERSION",
            DaemonError::LspStartFailed { .. } => "DAEMON_LSP_START_FAILED",
            DaemonError::LockAcquisitionFailed { .. } => "DAEMON_LOCK_FAILED",
            DaemonError::CacheDirNotWritable { .. } => "DAEMON_CACHE_NOT_WRITABLE",
            DaemonError::StalePidFile { .. } => "DAEMON_STALE_PID",
            DaemonError::UnresponsiveSocket { .. } => "DAEMON_SOCKET_UNRESPONSIVE",
            DaemonError::UnresponsiveNamedPipe { .. } => "DAEMON_PIPE_UNRESPONSIVE",
        }
    }
}
