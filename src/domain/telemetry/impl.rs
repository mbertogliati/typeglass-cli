use super::types::{
    CorrelationId, CorrelationIdError, EventTimestampError, EventTimestampMs, OperationId,
    OperationIdError, SpanId, SpanIdError, TelemetryRecord, TelemetryRecordError,
};

impl CorrelationId {
    pub fn new(value: String) -> Result<Self, CorrelationIdError> {
        if value.trim().is_empty() {
            return Err(CorrelationIdError::Empty);
        }
        Ok(Self(value))
    }
}

impl OperationId {
    pub fn new(value: String) -> Result<Self, OperationIdError> {
        if value.trim().is_empty() {
            return Err(OperationIdError::Empty);
        }
        Ok(Self(value))
    }
}

impl SpanId {
    pub fn new(value: String) -> Result<Self, SpanIdError> {
        if value.trim().is_empty() {
            return Err(SpanIdError::Empty);
        }
        Ok(Self(value))
    }
}

impl EventTimestampMs {
    pub fn new(value: u128) -> Result<Self, EventTimestampError> {
        if value == 0 {
            return Err(EventTimestampError::Zero);
        }
        Ok(Self { value })
    }
}

impl TelemetryRecord {
    pub fn validate(&self) -> Result<(), TelemetryRecordError> {
        if self.message.trim().is_empty() {
            return Err(TelemetryRecordError::EmptyMessage);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::telemetry::{TelemetryContext, TelemetryEvent, TelemetrySeverity};

    #[test]
    fn correlation_id_rejects_empty() {
        let result = CorrelationId::new(String::new());
        assert!(matches!(result, Err(CorrelationIdError::Empty)));
    }

    #[test]
    fn event_timestamp_rejects_zero() {
        let result = EventTimestampMs::new(0);
        assert!(matches!(result, Err(EventTimestampError::Zero)));
    }

    #[test]
    fn telemetry_record_requires_message() {
        let record = TelemetryRecord {
            timestamp: EventTimestampMs { value: 1 },
            severity: TelemetrySeverity::Info,
            context: TelemetryContext {
                correlation_id: CorrelationId("corr".to_string()),
                operation_id: OperationId("op".to_string()),
                span_id: SpanId("span".to_string()),
                request_id: None,
                workspace_identity: None,
            },
            event: TelemetryEvent::DaemonStarted,
            message: "   ".to_string(),
        };
        let result = record.validate();
        assert!(matches!(result, Err(TelemetryRecordError::EmptyMessage)));
    }
}
