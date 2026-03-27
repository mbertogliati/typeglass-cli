use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStartInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStartOutput {
    pub started: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartDaemonUseCase;

impl UseCase for StartDaemonUseCase {
    type Input = DaemonStartInput;
    type Output = DaemonStartOutput;

    const ACTION: UserAction = UserAction::StartDaemon;
}
