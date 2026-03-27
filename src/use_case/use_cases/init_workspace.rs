use std::path::PathBuf;

use crate::domain::language::Language;

use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitWorkspaceInput {
    pub workspace_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitWorkspaceOutput {
    pub language: Language,
    pub workspace_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitializeWorkspaceUseCase;

impl UseCase for InitializeWorkspaceUseCase {
    type Input = InitWorkspaceInput;
    type Output = InitWorkspaceOutput;

    const ACTION: UserAction = UserAction::InitializeWorkspace;
}
