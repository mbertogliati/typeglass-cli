use crate::domain::language::Language;
use crate::domain::workspace::{WorkspaceIdentity, WorkspacePath};

use super::lifecycle::DaemonProcessFiles;
use super::primitives::{NamedPipeName, Pid, SocketPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonTransportKind {
    UnixSocket,
    WindowsNamedPipe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonEndpoint {
    UnixSocket(SocketPath),
    WindowsNamedPipe(NamedPipeName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonRestartReason {
    IdentityMismatch,
    IdentityUnverifiable,
    PathUnavailable,
    SocketUnresponsive,
    ProcessDead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonState {
    NotRunning,
    Running {
        pid: Pid,
        endpoint: DaemonEndpoint,
        transport: DaemonTransportKind,
        language: Language,
        workspace: WorkspacePath,
        process_files: DaemonProcessFiles,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthCheck {
    PathAccessible,
    IdentityStillValid,
    SocketResponsive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonHealth {
    Healthy,
    StalePid {
        pid: Pid,
    },
    UnresponsiveSocket {
        pid: Pid,
        socket_path: SocketPath,
    },
    IdentityMismatch {
        expected: WorkspaceIdentity,
        found: WorkspaceIdentity,
    },
    PathUnavailable {
        workspace: WorkspacePath,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockAcquisitionOutcome {
    Acquired,
    Contended,
}
