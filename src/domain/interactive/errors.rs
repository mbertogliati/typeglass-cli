use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::language::Language;

use super::model::{JsonProtocolVersion, ProtocolCompatibility, RequestId, SessionPhase};

#[derive(Debug, Error)]
pub enum RequestIdError {
    #[error("Request id cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum ResponseSizeLimitError {
    #[error("Response size limit must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum SessionConcurrencyPolicyError {
    #[error("Max in-flight requests must be greater than zero")]
    ZeroInFlight,
}

#[derive(Debug, Error)]
pub enum InputSizeLimitError {
    #[error("Input size limit must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum IdleTimeoutError {
    #[error("Idle timeout must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum SessionPhaseError {
    #[error("Init request is only allowed in pre-init phase. Current phase: {phase:?}. Request: {request_kind}")]
    InitOutsidePreInit {
        phase: SessionPhase,
        request_kind: &'static str,
    },
    #[error("Session query request requires initialized workspace. Current phase: {phase:?}. Request: {request_kind}")]
    QueryBeforeInit {
        phase: SessionPhase,
        request_kind: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum JsonProtocolVersionError {
    #[error("Protocol major version must be greater than zero")]
    InvalidMajor,
}

#[derive(Debug, Error)]
pub enum SupportedProtocolVersionsError {
    #[error("Supported protocol versions list cannot be empty")]
    Empty,
    #[error("Supported protocol versions contain duplicates")]
    Duplicated,
}

#[derive(Debug, Error)]
pub enum ProtocolNegotiationError {
    #[error("No compatible protocol version between client and server")]
    NoCompatibleVersion,
    #[error("Protocol transition is not compatible. From: {from:?}, To: {to:?}, Compatibility: {compatibility:?}")]
    IncompatibleTransition {
        from: JsonProtocolVersion,
        to: JsonProtocolVersion,
        compatibility: ProtocolCompatibility,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractiveError {
    #[error("Detected unsupported languages: {detected:?}")]
    UnsupportedLanguage { detected: Vec<Language> },
    #[error("Multiple languages detected: {detected:?}")]
    MultipleLanguages { detected: Vec<Language> },
    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },
    #[error("LSP unavailable. Reason: {reason}")]
    LspUnavailable { reason: String },
    #[error("Invalid request payload. Reason: {reason}. Raw: {raw}")]
    InvalidRequest { raw: String, reason: String },
    #[error("Workspace not initialized")]
    WorkspaceNotInitialized,
    #[error("Response exceeds configured size limit. Size: {size_bytes} bytes, Limit: {limit_bytes} bytes")]
    ResponseTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },
    #[error("Path is outside workspace. Path: {path}, Workspace: {workspace}")]
    PathOutsideWorkspace { path: PathBuf, workspace: PathBuf },
    #[error("Request id is duplicated and still in-flight: {id:?}")]
    DuplicateRequestId { id: RequestId },
    #[error(
        "Input exceeds configured size limit. Size: {size_bytes} bytes, Limit: {limit_bytes} bytes"
    )]
    InputTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },
    #[error("Too many in-flight requests in interactive session. In-flight: {in_flight}, Limit: {limit}")]
    SessionBackpressure { in_flight: usize, limit: usize },
    #[error("Protocol negotiation failed")]
    ProtocolNegotiationFailed,
}
