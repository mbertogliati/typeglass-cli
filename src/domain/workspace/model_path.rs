use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspacePath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceRelativePath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceFile(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceScope {
    SingleRoot { root: WorkspacePath },
    MultiRoot { roots: Vec<WorkspacePath> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceRootRef(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalPath(pub(crate) PathBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootSelectionResult {
    Selected(WorkspaceRootRef),
    Ambiguous { candidates: Vec<WorkspaceRootRef> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolRootResolution {
    Unique {
        symbol: String,
        root: WorkspaceRootRef,
    },
    Ambiguous {
        symbol: String,
        roots: Vec<WorkspaceRootRef>,
    },
    NotFound {
        symbol: String,
    },
}
