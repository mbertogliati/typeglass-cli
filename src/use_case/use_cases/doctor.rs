use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOutput {
    pub healthy: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunDoctorUseCase;

impl UseCase for RunDoctorUseCase {
    type Input = DoctorInput;
    type Output = DoctorOutput;

    const ACTION: UserAction = UserAction::RunDoctor;
}
