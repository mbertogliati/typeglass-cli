use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStopInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStopOutput {
    pub stopped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopDaemonUseCase;

impl UseCase for StopDaemonUseCase {
    type Input = DaemonStopInput;
    type Output = DaemonStopOutput;

    const ACTION: UserAction = UserAction::StopDaemon;
}
