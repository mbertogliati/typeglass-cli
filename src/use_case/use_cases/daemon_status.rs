use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStatusInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStatusOutput {
    pub running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetDaemonStatusUseCase;

impl UseCase for GetDaemonStatusUseCase {
    type Input = DaemonStatusInput;
    type Output = DaemonStatusOutput;

    const ACTION: UserAction = UserAction::GetDaemonStatus;
}
