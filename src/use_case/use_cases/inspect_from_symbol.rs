use crate::domain::graph::TypeGraph;
use crate::domain::interactive::RequestId;

use crate::use_case::{UseCase, UserAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectFromSymbolInput {
    pub request_id: Option<RequestId>,
    pub symbol: String,
    pub depth: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectFromSymbolOutput {
    pub graph: TypeGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectTypeFromSymbolUseCase;

impl UseCase for InspectTypeFromSymbolUseCase {
    type Input = InspectFromSymbolInput;
    type Output = InspectFromSymbolOutput;

    const ACTION: UserAction = UserAction::InspectTypeFromSymbol;
}
