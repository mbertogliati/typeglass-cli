use std::path::PathBuf;

use crate::domain::graph::{QualifiedSymbolName, SourceLocation};
use crate::domain::language::Language;

/// LSP Gateway - Output port for LSP communication
/// 
/// Implemented by infrastructure/lsp adapters.
/// Provides LSP operations needed by the domain.
/// 
/// Note: Methods return Future types manually to avoid async_trait dependency in domain.
pub trait LspGateway: Send + Sync {
    /// Initialize LSP connection for a workspace
    fn initialize(&mut self, workspace_root: PathBuf) -> Result<(), LspError>;

    /// Query definition location of a symbol
    fn query_definition(
        &mut self,
        file_path: PathBuf,
        line: u32,
        column: u32,
    ) -> Result<Vec<SourceLocation>, LspError>;

    /// Find all references to a symbol
    fn find_references(
        &mut self,
        file_path: PathBuf,
        line: u32,
        column: u32,
    ) -> Result<Vec<SourceLocation>, LspError>;

    /// Search for symbols by name in workspace
    fn workspace_symbols(
        &mut self,
        query: &str,
    ) -> Result<Vec<SymbolInfo>, LspError>;

    /// Shutdown LSP connection
    fn shutdown(&mut self) -> Result<(), LspError>;

    /// Check if LSP is initialized and ready
    fn is_ready(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Class,
    Interface,
    Struct,
    Enum,
    Function,
    Method,
    Variable,
    Constant,
    Module,
    Namespace,
}

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("LSP not initialized")]
    NotInitialized,

    #[error("LSP initialization failed: {0}")]
    InitializationFailed(String),

    #[error("LSP query failed: {0}")]
    QueryFailed(String),

    #[error("LSP timeout")]
    Timeout,

    #[error("LSP process error: {0}")]
    ProcessError(String),

    #[error("Invalid response from LSP: {0}")]
    InvalidResponse(String),
}
