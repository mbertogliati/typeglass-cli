use std::path::PathBuf;

use crate::domain::graph::TypeGraph;
use crate::domain::interactive::RequestId;

use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectFromFileInput {
    pub request_id: Option<RequestId>,
    pub file: PathBuf,
    pub depth: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectFromFileOutput {
    pub graph: TypeGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectTypeFromFileUseCase;

impl UseCase for InspectTypeFromFileUseCase {
    type Input = InspectFromFileInput;
    type Output = InspectFromFileOutput;

    const ACTION: UserAction = UserAction::InspectTypeFromFile;
}
