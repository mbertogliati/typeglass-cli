use crate::domain::contracts::{MachineCode, UserMessage};

use crate::domain::interactive::InteractiveError;

impl UserMessage for InteractiveError {
    fn user_message(&self) -> String {
        self.to_string()
    }
}

impl MachineCode for InteractiveError {
    fn code(&self) -> &'static str {
        match self {
            InteractiveError::UnsupportedLanguage { .. } => "INTERACTIVE_UNSUPPORTED_LANGUAGE",
            InteractiveError::MultipleLanguages { .. } => "INTERACTIVE_MULTIPLE_LANGUAGES",
            InteractiveError::SymbolNotFound { .. } => "INTERACTIVE_SYMBOL_NOT_FOUND",
            InteractiveError::LspUnavailable { .. } => "INTERACTIVE_LSP_UNAVAILABLE",
            InteractiveError::InvalidRequest { .. } => "INTERACTIVE_INVALID_REQUEST",
            InteractiveError::WorkspaceNotInitialized => "INTERACTIVE_WORKSPACE_NOT_INITIALIZED",
            InteractiveError::ResponseTooLarge { .. } => "INTERACTIVE_RESPONSE_TOO_LARGE",
            InteractiveError::PathOutsideWorkspace { .. } => "INTERACTIVE_PATH_OUTSIDE_WORKSPACE",
            InteractiveError::DuplicateRequestId { .. } => "INTERACTIVE_DUPLICATE_REQUEST_ID",
            InteractiveError::InputTooLarge { .. } => "INTERACTIVE_INPUT_TOO_LARGE",
            InteractiveError::SessionBackpressure { .. } => "INTERACTIVE_BACKPRESSURE",
            InteractiveError::ProtocolNegotiationFailed => {
                "INTERACTIVE_PROTOCOL_NEGOTIATION_FAILED"
            }
        }
    }
}
