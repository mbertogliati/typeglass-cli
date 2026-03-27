use crate::domain::interactive::RequestId;
use crate::domain::workspace::WorkspaceFile;

use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidateFilesInput {
    pub request_id: Option<RequestId>,
    pub files: Vec<WorkspaceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidateFilesOutput {
    pub invalidated_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidateFilesUseCase;

impl UseCase for InvalidateFilesUseCase {
    type Input = InvalidateFilesInput;
    type Output = InvalidateFilesOutput;

    const ACTION: UserAction = UserAction::InvalidateFiles;
}
