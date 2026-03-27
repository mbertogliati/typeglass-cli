use std::path::PathBuf;

use crate::domain::daemon::{CacheDir, DaemonEndpoint, DaemonState};
use crate::domain::graph::TypeGraph;
use crate::domain::language::Language;
use crate::domain::lsp::{InvalidationResult, LspCapabilities, LspQuery, QueryTimeout};
use crate::domain::workspace::{WorkspaceFile, WorkspaceIdentity, WorkspacePath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProbe {
    pub requested_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub path: WorkspacePath,
    pub identity: WorkspaceIdentity,
    pub language: Language,
    pub readable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadataSnapshot {
    pub file: WorkspaceFile,
    pub size_bytes: u64,
    pub modified_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStartSpec {
    pub workspace: WorkspacePath,
    pub language: Language,
    pub cache_dir: CacheDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonRuntimeSnapshot {
    pub state: DaemonState,
    pub endpoint: Option<DaemonEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspQueryRequest {
    pub query: LspQuery,
    pub timeout: QueryTimeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspQueryResponse {
    pub graph: Option<TypeGraph>,
    pub capabilities: Option<LspCapabilities>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockTick {
    pub unix_time_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationRequestDto {
    pub files: Vec<WorkspaceFile>,
}

impl From<InvalidationRequestDto> for crate::domain::lsp::InvalidationRequest {
    fn from(value: InvalidationRequestDto) -> Self {
        Self { files: value.files }
    }
}

impl From<InvalidationResult> for usize {
    fn from(value: InvalidationResult) -> Self {
        value.invalidated_files
    }
}
