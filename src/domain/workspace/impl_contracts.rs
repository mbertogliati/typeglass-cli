use crate::domain::contracts::{MachineCode, UserMessage};

use crate::domain::workspace::WorkspacePathError;

impl UserMessage for WorkspacePathError {
    fn user_message(&self) -> String {
        self.to_string()
    }
}

impl MachineCode for WorkspacePathError {
    fn code(&self) -> &'static str {
        match self {
            WorkspacePathError::Empty => "WORKSPACE_PATH_EMPTY",
            WorkspacePathError::DoesNotExist { .. } => "WORKSPACE_PATH_MISSING",
            WorkspacePathError::NotADirectory { .. } => "WORKSPACE_NOT_A_DIRECTORY",
            WorkspacePathError::NotReadable { .. } => "WORKSPACE_NOT_READABLE",
        }
    }
}
