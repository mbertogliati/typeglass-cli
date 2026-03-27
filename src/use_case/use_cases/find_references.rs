use std::path::PathBuf;

use crate::domain::interactive::RequestId;

use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindReferencesInput {
    pub request_id: Option<RequestId>,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindReferencesOutput {
    pub references: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindReferencesUseCase;

impl UseCase for FindReferencesUseCase {
    type Input = FindReferencesInput;
    type Output = FindReferencesOutput;

    const ACTION: UserAction = UserAction::FindReferences;
}
