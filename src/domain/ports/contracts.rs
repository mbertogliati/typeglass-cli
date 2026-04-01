use std::future::Future;
use std::time::Duration;

use crate::domain::daemon::Pid;
use crate::domain::lsp::InvalidationRequest;
use crate::domain::workspace::WorkspaceFile;

use super::dto::{
    ClockTick, DaemonRuntimeSnapshot, DaemonStartSpec, FileMetadataSnapshot, LspQueryRequest,
    LspQueryResponse, WorkspaceProbe, WorkspaceSnapshot,
};
use super::errors::{
    ClockPortError, DaemonPortError, FileSystemPortError, LspPortError, WorkspacePortError,
};

pub trait WorkspacePort {
    type ProbeFuture<'a>: Future<Output = Result<WorkspaceSnapshot, WorkspacePortError>> + Send + 'a
    where
        Self: 'a;

    fn probe_workspace<'a>(&'a self, probe: WorkspaceProbe) -> Self::ProbeFuture<'a>;
}

pub trait FileSystemPort {
    type MetadataFuture<'a>: Future<Output = Result<FileMetadataSnapshot, FileSystemPortError>>
        + Send
        + 'a
    where
        Self: 'a;

    fn read_file_metadata<'a>(&'a self, file: WorkspaceFile) -> Self::MetadataFuture<'a>;
}

pub trait DaemonPort {
    type StartFuture<'a>: Future<Output = Result<DaemonRuntimeSnapshot, DaemonPortError>>
        + Send
        + 'a
    where
        Self: 'a;
    type StopFuture<'a>: Future<Output = Result<(), DaemonPortError>> + Send + 'a
    where
        Self: 'a;
    type StatusFuture<'a>: Future<Output = Result<DaemonRuntimeSnapshot, DaemonPortError>>
        + Send
        + 'a
    where
        Self: 'a;

    fn start_daemon<'a>(&'a self, spec: DaemonStartSpec) -> Self::StartFuture<'a>;
    fn stop_daemon<'a>(&'a self, pid: Option<Pid>) -> Self::StopFuture<'a>;
    fn daemon_status<'a>(&'a self) -> Self::StatusFuture<'a>;
}

pub trait LspPort {
    type QueryFuture<'a>: Future<Output = Result<LspQueryResponse, LspPortError>> + Send + 'a
    where
        Self: 'a;
    type InvalidateFuture<'a>: Future<Output = Result<crate::domain::lsp::InvalidationResult, LspPortError>>
        + Send
        + 'a
    where
        Self: 'a;

    fn run_query<'a>(&'a self, request: LspQueryRequest) -> Self::QueryFuture<'a>;
    fn invalidate<'a>(&'a self, request: InvalidationRequest) -> Self::InvalidateFuture<'a>;
}

pub trait ClockPort {
    type NowFuture<'a>: Future<Output = Result<ClockTick, ClockPortError>> + Send + 'a
    where
        Self: 'a;
    type SleepFuture<'a>: Future<Output = Result<(), ClockPortError>> + Send + 'a
    where
        Self: 'a;

    fn now<'a>(&'a self) -> Self::NowFuture<'a>;
    fn sleep<'a>(&'a self, duration: Duration) -> Self::SleepFuture<'a>;
}
