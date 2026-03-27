use crate::domain::interactive::RequestId;

use super::action::UserAction;
use super::use_cases::{
    DaemonStartInput, DaemonStatusInput, DaemonStopInput, DoctorInput, FindReferencesInput,
    InitWorkspaceInput, InspectFromFileInput, InspectFromSymbolInput, InvalidateFilesInput,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserCommand {
    InitializeWorkspace(InitWorkspaceInput),
    InspectTypeFromSymbol(InspectFromSymbolInput),
    InspectTypeFromFile(InspectFromFileInput),
    FindReferences(FindReferencesInput),
    InvalidateFiles(InvalidateFilesInput),
    StartDaemon(DaemonStartInput),
    StopDaemon(DaemonStopInput),
    GetDaemonStatus(DaemonStatusInput),
    RunDoctor(DoctorInput),
}

impl UserCommand {
    pub const fn action(&self) -> UserAction {
        match self {
            Self::InitializeWorkspace(_) => UserAction::InitializeWorkspace,
            Self::InspectTypeFromSymbol(_) => UserAction::InspectTypeFromSymbol,
            Self::InspectTypeFromFile(_) => UserAction::InspectTypeFromFile,
            Self::FindReferences(_) => UserAction::FindReferences,
            Self::InvalidateFiles(_) => UserAction::InvalidateFiles,
            Self::StartDaemon(_) => UserAction::StartDaemon,
            Self::StopDaemon(_) => UserAction::StopDaemon,
            Self::GetDaemonStatus(_) => UserAction::GetDaemonStatus,
            Self::RunDoctor(_) => UserAction::RunDoctor,
        }
    }

    pub fn request_id(&self) -> Option<&RequestId> {
        match self {
            Self::InspectTypeFromSymbol(input) => input.request_id.as_ref(),
            Self::InspectTypeFromFile(input) => input.request_id.as_ref(),
            Self::FindReferences(input) => input.request_id.as_ref(),
            Self::InvalidateFiles(input) => input.request_id.as_ref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_maps_to_expected_action() {
        let command = UserCommand::StartDaemon(DaemonStartInput);
        assert!(matches!(command.action(), UserAction::StartDaemon));
    }
}
