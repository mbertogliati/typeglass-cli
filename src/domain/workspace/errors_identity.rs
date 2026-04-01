use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::domain::workspace::WorkspaceIdentity;

#[derive(Debug, Error)]
pub enum WorkspaceIdentityError {
    #[error("Workspace path is not accessible: {path}. Reason: {reason}")]
    PathNotAccessible { path: PathBuf, reason: io::Error },
    #[error("No workspace identity source is available for workspace: {workspace}")]
    IdentitySourceUnavailable { workspace: PathBuf },
}

#[derive(Debug, Error)]
pub enum WorkspaceIdentityValueError {
    #[error("Workspace identity cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum StructuralFingerprintError {
    #[error("Structural fingerprint cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum IdentityStrategyChainError {
    #[error("Identity strategy chain cannot be empty. Provide at least one strategy")]
    Empty,
    #[error("Identity strategy chain contains duplicate strategy entries")]
    DuplicatedStrategy,
}

#[derive(Debug, Error)]
pub enum WorkspaceRuntimeError {
    #[error("Workspace path became unavailable: {path}. Reason: {reason}")]
    PathBecameUnavailable { path: PathBuf, reason: io::Error },
    #[error("Workspace project identity changed while running. Previous: {previous:?}, Current: {current:?}")]
    IdentityChanged {
        previous: WorkspaceIdentity,
        current: WorkspaceIdentity,
    },
    #[error("Workspace disk became unavailable")]
    DiskUnavailable,
}
