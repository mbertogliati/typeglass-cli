use crate::domain::contracts::{MachineCode, UserMessage};

use crate::domain::lsp::LspError;

impl UserMessage for LspError {
    fn user_message(&self) -> String {
        self.to_string()
    }
}

impl MachineCode for LspError {
    fn code(&self) -> &'static str {
        match self {
            LspError::Timeout { .. } => "LSP_TIMEOUT",
            LspError::ProcessDied { .. } => "LSP_PROCESS_DIED",
            LspError::InvalidResponse { .. } => "LSP_INVALID_RESPONSE",
            LspError::WorkspaceError { .. } => "LSP_WORKSPACE_ERROR",
            LspError::SymbolNotFound { .. } => "LSP_SYMBOL_NOT_FOUND",
            LspError::FileNotReadable { .. } => "LSP_FILE_NOT_READABLE",
            LspError::Cancelled { .. } => "LSP_CANCELLED",
            LspError::Backpressure { .. } => "LSP_BACKPRESSURE",
        }
    }
}
