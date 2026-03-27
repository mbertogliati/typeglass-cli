use std::time::SystemTime;

use crate::domain::interactive::{
    IdleTimeoutError, IdleTimeoutMs, InputSizeLimit, InputSizeLimitError, JsonProtocolVersion,
    JsonProtocolVersionError, RequestEnvelope, RequestId, RequestIdError, ResponseSizeLimit,
    ResponseSizeLimitError, SessionConcurrencyPolicy, SessionConcurrencyPolicyError,
    SupportedProtocolVersions, SupportedProtocolVersionsError,
};

impl RequestId {
    pub fn new(value: String) -> Result<Self, RequestIdError> {
        if value.trim().is_empty() {
            return Err(RequestIdError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResponseSizeLimit {
    pub fn new(bytes: usize) -> Result<Self, ResponseSizeLimitError> {
        if bytes == 0 {
            return Err(ResponseSizeLimitError::Zero);
        }
        Ok(Self { bytes })
    }
}

impl InputSizeLimit {
    pub fn new(bytes: usize) -> Result<Self, InputSizeLimitError> {
        if bytes == 0 {
            return Err(InputSizeLimitError::Zero);
        }
        Ok(Self { bytes })
    }
}

impl IdleTimeoutMs {
    pub fn new(value: u64) -> Result<Self, IdleTimeoutError> {
        if value == 0 {
            return Err(IdleTimeoutError::Zero);
        }
        Ok(Self { value })
    }
}

impl SessionConcurrencyPolicy {
    pub fn validate(&self) -> Result<(), SessionConcurrencyPolicyError> {
        if self.max_in_flight_requests == 0 {
            return Err(SessionConcurrencyPolicyError::ZeroInFlight);
        }
        Ok(())
    }
}

impl<T> RequestEnvelope<T> {
    pub fn now(id: RequestId, payload: T) -> Self {
        let issued_at_unix_ms = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        Self {
            id,
            issued_at_unix_ms,
            payload,
        }
    }
}

impl JsonProtocolVersion {
    pub fn new(major: u16, minor: u16) -> Result<Self, JsonProtocolVersionError> {
        if major == 0 {
            return Err(JsonProtocolVersionError::InvalidMajor);
        }
        Ok(Self { major, minor })
    }
}

impl SupportedProtocolVersions {
    pub fn new(versions: Vec<JsonProtocolVersion>) -> Result<Self, SupportedProtocolVersionsError> {
        if versions.is_empty() {
            return Err(SupportedProtocolVersionsError::Empty);
        }

        for (index, version) in versions.iter().enumerate() {
            if versions
                .iter()
                .skip(index + 1)
                .any(|other| other == version)
            {
                return Err(SupportedProtocolVersionsError::Duplicated);
            }
        }

        Ok(Self { versions })
    }
}
