use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

use crate::domain::daemon::Pid;
use crate::domain::lsp::LspError;

#[derive(Debug, Error)]
pub enum WorkspacePortError {
    #[error("Workspace probe failed at path {path}. Reason: {reason}")]
    ProbeFailed { path: PathBuf, reason: String },
    #[error("Workspace identity is unavailable at path {path}")]
    IdentityUnavailable { path: PathBuf },
    #[error("Workspace language detection failed at path {path}. Reason: {reason}")]
    LanguageDetectionFailed { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum FileSystemPortError {
    #[error("Failed to read file metadata for {path}. Reason: {reason}")]
    MetadataReadFailed { path: PathBuf, reason: String },
    #[error("Failed to list directory {path}. Reason: {reason}")]
    ReadDirFailed { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum DaemonPortError {
    #[error("Daemon start failed for workspace {workspace}. Reason: {reason}")]
    StartFailed { workspace: PathBuf, reason: String },
    #[error("Daemon stop failed. PID: {pid:?}. Reason: {reason}")]
    StopFailed { pid: Option<Pid>, reason: String },
    #[error("Daemon status check failed. Reason: {reason}")]
    StatusFailed { reason: String },
}

#[derive(Debug, Error)]
pub enum LspPortError {
    #[error("LSP query failed. Reason: {source}")]
    QueryFailed { source: LspError },
    #[error("LSP invalidation failed. Reason: {reason}")]
    InvalidateFailed { reason: String },
}

#[derive(Debug, Error)]
pub enum ClockPortError {
    #[error("Clock read failed. Reason: {reason}")]
    ReadFailed { reason: String },
    #[error("Sleep failed for duration {duration:?}. Reason: {reason}")]
    SleepFailed { duration: Duration, reason: String },
}
