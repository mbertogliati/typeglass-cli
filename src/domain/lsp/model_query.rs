use std::path::PathBuf;

use crate::domain::graph::SymbolName;
use crate::domain::workspace::{WorkspaceFile, WorkspaceRootRef};

use super::primitives::{Depth, LspRequestId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPoint {
    Symbol(SymbolName),
    File(WorkspaceFile),
    Module(PathBuf),
    PublicExports,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspQuery {
    FromSymbol { symbol: String, depth: Depth },
    FromFile { path: PathBuf, depth: Depth },
    References { symbol: String },
    Invalidate { files: Vec<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspCommand {
    Execute(LspQuery),
    Cancel { request_id: LspRequestId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationRequest {
    pub files: Vec<WorkspaceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationResult {
    pub reindexed: bool,
    pub invalidated_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalRequest {
    pub entry_point: EntryPoint,
    pub depth: Depth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootedTraversalRequest {
    pub root: WorkspaceRootRef,
    pub traversal: TraversalRequest,
}
