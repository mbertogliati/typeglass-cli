use crate::domain::ports::*;

pub trait ApplicationAdapters: Send + Sync {
    type Workspace: WorkspacePort + Send + Sync;
    type FileSystem: FileSystemPort + Send + Sync;
    type Daemon: DaemonPort + Send + Sync;
    type Lsp: LspPort + Send + Sync + Clone + 'static;  // Add 'static bound
    type Clock: ClockPort + Send + Sync;

    fn workspace(&self) -> &Self::Workspace;
    fn file_system(&self) -> &Self::FileSystem;
    fn daemon(&self) -> &Self::Daemon;
    fn lsp(&self) -> &Self::Lsp;
    fn clock(&self) -> &Self::Clock;
}

pub struct UnwiredAdapters;

impl ApplicationAdapters for UnwiredAdapters {
    type Workspace = UnwiredWorkspace;
    type FileSystem = UnwiredFileSystem;
    type Daemon = UnwiredDaemon;
    type Lsp = UnwiredLsp;
    type Clock = UnwiredClock;

    fn workspace(&self) -> &Self::Workspace { &UnwiredWorkspace }
    fn file_system(&self) -> &Self::FileSystem { &UnwiredFileSystem }
    fn daemon(&self) -> &Self::Daemon { &UnwiredDaemon }
    fn lsp(&self) -> &Self::Lsp { &UnwiredLsp }
    fn clock(&self) -> &Self::Clock { &UnwiredClock }
}

pub struct UnwiredWorkspace;
impl WorkspacePort for UnwiredWorkspace {
    type ProbeFuture<'a> = std::future::Ready<Result<crate::domain::ports::WorkspaceSnapshot, crate::domain::ports::WorkspacePortError>>;
    fn probe_workspace<'a>(&'a self, probe: crate::domain::ports::WorkspaceProbe) -> Self::ProbeFuture<'a> {
        std::future::ready(Err(crate::domain::ports::WorkspacePortError::ProbeFailed {
            path: probe.requested_path,
            reason: "WorkspacePort not wired in test".to_string(),
        }))
    }
}

pub struct UnwiredFileSystem;
impl FileSystemPort for UnwiredFileSystem {
    type MetadataFuture<'a> = std::future::Ready<Result<crate::domain::ports::FileMetadataSnapshot, crate::domain::ports::FileSystemPortError>>;
    fn read_file_metadata<'a>(&'a self, file: crate::domain::workspace::WorkspaceFile) -> Self::MetadataFuture<'a> {
        std::future::ready(Err(crate::domain::ports::FileSystemPortError::MetadataReadFailed {
            path: file.as_path().to_path_buf(),
            reason: "FileSystemPort not wired in test".to_string(),
        }))
    }
}

pub struct UnwiredDaemon;
impl DaemonPort for UnwiredDaemon {
    type StartFuture<'a> = std::future::Ready<Result<crate::domain::ports::DaemonRuntimeSnapshot, crate::domain::ports::DaemonPortError>>;
    type StopFuture<'a> = std::future::Ready<Result<(), crate::domain::ports::DaemonPortError>>;
    type StatusFuture<'a> = std::future::Ready<Result<crate::domain::ports::DaemonRuntimeSnapshot, crate::domain::ports::DaemonPortError>>;

    fn start_daemon<'a>(&'a self, spec: crate::domain::ports::DaemonStartSpec) -> Self::StartFuture<'a> {
        std::future::ready(Err(crate::domain::ports::DaemonPortError::StartFailed {
            workspace: spec.workspace.as_path().to_path_buf(),
            reason: "DaemonPort not wired in test".to_string(),
        }))
    }
    
    fn stop_daemon<'a>(&'a self, pid: Option<crate::domain::daemon::Pid>) -> Self::StopFuture<'a> {
        std::future::ready(Err(crate::domain::ports::DaemonPortError::StopFailed {
            pid,
            reason: "DaemonPort not wired in test".to_string(),
        }))
    }
    
    fn daemon_status<'a>(&'a self) -> Self::StatusFuture<'a> {
        std::future::ready(Err(crate::domain::ports::DaemonPortError::StatusFailed {
            reason: "DaemonPort not wired in test".to_string(),
        }))
    }
}

#[derive(Clone)]
pub struct UnwiredLsp;
impl LspPort for UnwiredLsp {
    type QueryFuture<'a> = std::future::Ready<Result<crate::domain::ports::LspQueryResponse, crate::domain::ports::LspPortError>>;
    type InvalidateFuture<'a> = std::future::Ready<Result<crate::domain::lsp::InvalidationResult, crate::domain::ports::LspPortError>>;

    fn run_query<'a>(&'a self, _request: crate::domain::ports::LspQueryRequest) -> Self::QueryFuture<'a> {
        std::future::ready(Err(crate::domain::ports::LspPortError::QueryFailed {
            source: crate::domain::lsp::LspError::SymbolNotFound {
                symbol: "test-unwired".to_string(),
            },
        }))
    }
    
    fn invalidate<'a>(&'a self, _request: crate::domain::lsp::InvalidationRequest) -> Self::InvalidateFuture<'a> {
        std::future::ready(Ok(crate::domain::lsp::InvalidationResult {
            reindexed: false,
            invalidated_files: 0,
        }))
    }
}

pub struct UnwiredClock;
impl ClockPort for UnwiredClock {
    type NowFuture<'a> = std::future::Ready<Result<crate::domain::ports::ClockTick, crate::domain::ports::ClockPortError>>;
    type SleepFuture<'a> = std::future::Ready<Result<(), crate::domain::ports::ClockPortError>>;

    fn now<'a>(&'a self) -> Self::NowFuture<'a> {
        std::future::ready(Err(crate::domain::ports::ClockPortError::ReadFailed {
            reason: "ClockPort not wired in test".to_string(),
        }))
    }
    
    fn sleep<'a>(&'a self, duration: std::time::Duration) -> Self::SleepFuture<'a> {
        std::future::ready(Err(crate::domain::ports::ClockPortError::SleepFailed {
            duration,
            reason: "ClockPort not wired in test".to_string(),
        }))
    }
}
