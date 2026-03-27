use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::domain::workspace::WorkspaceRootRef;

#[derive(Debug, Error)]
pub enum WorkspacePathError {
    #[error("Workspace path cannot be empty")]
    Empty,
    #[error("Workspace path does not exist: {path}")]
    DoesNotExist { path: PathBuf },
    #[error("Workspace path is not a directory: {path}")]
    NotADirectory { path: PathBuf },
    #[error("Workspace path is not readable: {path}. Reason: {reason}")]
    NotReadable { path: PathBuf, reason: io::Error },
}

#[derive(Debug, Error)]
pub enum WorkspaceRelativePathError {
    #[error("Relative path cannot be absolute: {path}")]
    AbsolutePathNotAllowed { path: PathBuf },
    #[error("Relative path cannot escape workspace root: {path}")]
    PathTraversalNotAllowed { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum WorkspaceFileError {
    #[error("Workspace file path cannot be empty")]
    Empty,
    #[error("Workspace file does not exist: {path}")]
    DoesNotExist { path: PathBuf },
    #[error("Workspace file is outside of workspace root. Workspace: {workspace}, File: {path}")]
    OutsideWorkspace { workspace: PathBuf, path: PathBuf },
    #[error("Workspace file path is not a regular file: {path}")]
    NotAFile { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum WorkspaceRootSelectionError {
    #[error("No workspace root matched current path: {current_path}")]
    NoMatchingRoot { current_path: PathBuf },
    #[error("Multiple workspace roots matched current path: {current_path}. Candidate count: {candidate_count}")]
    AmbiguousSelection {
        current_path: PathBuf,
        candidate_count: usize,
    },
}

#[derive(Debug, Error)]
pub enum WorkspaceRootRefError {
    #[error("Workspace root reference cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum CanonicalPathError {
    #[error("Canonical path cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum MultiRootQueryError {
    #[error("A root must be selected in multi-root mode")]
    RootNotSelected,
    #[error("Symbol is ambiguous across multiple roots: {symbol}")]
    SymbolAmbiguous {
        symbol: String,
        roots: Vec<WorkspaceRootRef>,
    },
}

#[derive(Debug, Error)]
pub enum WorkspaceTraversalError {
    #[error("Detected a symlink cycle at: {path}")]
    SymlinkCycleDetected { path: PathBuf },
    #[error("Symlink escapes workspace boundary: {path}")]
    SymlinkEscapesWorkspace { path: PathBuf },
    #[error("Path casing mismatch may break imports on case-sensitive filesystems. Expected: {expected}, Actual: {actual}")]
    CaseMismatch { expected: PathBuf, actual: PathBuf },
}
