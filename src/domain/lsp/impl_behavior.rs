use crate::domain::lsp::{LspError, LspFailureKind, Retryability};

impl LspError {
    pub const fn kind(&self) -> LspFailureKind {
        match self {
            Self::Timeout { .. } => LspFailureKind::Timeout,
            Self::ProcessDied { .. } => LspFailureKind::ProcessDied,
            Self::InvalidResponse { .. } => LspFailureKind::InvalidResponse,
            Self::WorkspaceError { .. } => LspFailureKind::WorkspaceError,
            Self::SymbolNotFound { .. } => LspFailureKind::SymbolNotFound,
            Self::FileNotReadable { .. } => LspFailureKind::FileNotReadable,
            Self::Cancelled { .. } => LspFailureKind::Cancelled,
            Self::Backpressure { .. } => LspFailureKind::Backpressure,
        }
    }

    pub const fn retryability(&self) -> Retryability {
        match self.kind() {
            LspFailureKind::Timeout
            | LspFailureKind::ProcessDied
            | LspFailureKind::InvalidResponse
            | LspFailureKind::WorkspaceError
            | LspFailureKind::Backpressure => Retryability::Retryable,
            LspFailureKind::SymbolNotFound
            | LspFailureKind::FileNotReadable
            | LspFailureKind::Cancelled => Retryability::NonRetryable,
        }
    }
}
