use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::graph::{GraphCompleteness, TypeGraph};
use crate::domain::language::Language;

use crate::domain::interactive::{InteractiveError, ProtocolNegotiationRequest, SessionAction};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestEnvelope<T> {
    pub id: RequestId,
    pub issued_at_unix_ms: u128,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseEnvelope<T> {
    pub id: RequestId,
    pub payload: T,
    pub completeness: GraphCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseErrorDetail {
    pub raw: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestHandlingOutcome {
    Accepted {
        id: Option<RequestId>,
    },
    Rejected {
        error: InteractiveError,
        action: SessionAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PreInitRequest {
    Init { path: PathBuf },
    NegotiateProtocol { request: ProtocolNegotiationRequest },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReadyRequest {
    FromSymbol {
        id: RequestId,
        symbol: String,
        depth: Option<u8>,
    },
    FromFile {
        id: RequestId,
        path: PathBuf,
        depth: Option<u8>,
    },
    References {
        id: RequestId,
        symbol: String,
    },
    Invalidate {
        id: RequestId,
        files: Vec<PathBuf>,
    },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhasedRequest {
    PreInit(PreInitRequest),
    Ready(ReadyRequest),
}

pub type InteractiveRequest = PhasedRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InteractiveResponse {
    Ready {
        language: Language,
        workspace: String,
    },
    Graph {
        id: RequestId,
        graph: TypeGraph,
    },
    Error {
        id: Option<RequestId>,
        error: InteractiveError,
    },
    Bye,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Truncated<T> {
    Full(T),
    Truncated {
        value: T,
        original_size_bytes: usize,
        limit_size_bytes: usize,
    },
}
