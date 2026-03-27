use thiserror::Error;

use crate::domain::interactive::RequestId;
use crate::domain::workspace::WorkspaceIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanId(pub(crate) String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventTimestampMs {
    pub value: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetrySeverity {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryContext {
    pub correlation_id: CorrelationId,
    pub operation_id: OperationId,
    pub span_id: SpanId,
    pub request_id: Option<RequestId>,
    pub workspace_identity: Option<WorkspaceIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryEvent {
    DaemonStarted,
    DaemonRestarted,
    LspRequestSent,
    LspResponseReceived,
    InteractiveRequestAccepted,
    InteractiveRequestRejected,
    GraphBuiltPartial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryRecord {
    pub timestamp: EventTimestampMs,
    pub severity: TelemetrySeverity,
    pub context: TelemetryContext,
    pub event: TelemetryEvent,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum CorrelationIdError {
    #[error("Correlation id cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum OperationIdError {
    #[error("Operation id cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum SpanIdError {
    #[error("Span id cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum EventTimestampError {
    #[error("Event timestamp must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum TelemetryRecordError {
    #[error("Telemetry message cannot be empty")]
    EmptyMessage,
}
