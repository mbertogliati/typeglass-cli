use std::path::{Path, PathBuf};

use crate::domain::graph::SourceLocation;

/// Filesystem Gateway - Output port for filesystem operations
/// 
/// Implemented by infrastructure/filesystem adapters.
/// Provides filesystem scanning and symbol search operations.
pub trait FilesystemGateway: Send + Sync {
    /// Find first occurrence of a symbol in workspace
    fn find_symbol(
        &self,
        symbol: &str,
        workspace_root: &Path,
    ) -> Result<Option<SourceLocation>, FilesystemError>;

    /// Scan directory recursively for symbol occurrences
    fn scan_directory(
        &self,
        symbol: &str,
        dir: &Path,
    ) -> Result<Vec<SourceLocation>, FilesystemError>;

    /// Check if a path is a valid source file
    fn is_source_file(&self, path: &Path) -> bool;

    /// List all source files in a directory
    fn list_source_files(&self, dir: &Path) -> Result<Vec<PathBuf>, FilesystemError>;

    /// Read file contents
    fn read_file(&self, path: &Path) -> Result<String, FilesystemError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FilesystemError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid UTF-8 in file: {0}")]
    InvalidUtf8(PathBuf),
}
