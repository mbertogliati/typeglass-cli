use std::time::Duration;

use crate::domain::lsp::{
    CompatibilityMatrixEntry, Depth, DepthError, LspCompatibilityMatrix,
    LspCompatibilityMatrixError, LspRequestId, LspRequestIdError, LspRetryBackoffError,
    LspRetryBackoffMs, LspRetryBudget, LspRetryBudgetError, LspRetryDelayRange,
    LspRetryDelayRangeError, LspRetryJitterError, LspRetryJitterPercent, MaxInFlightLspRequests,
    MaxInFlightLspRequestsError, QueryExecutionMode, QueryExecutionPolicy,
    QueryExecutionPolicyError, QueryTimeout, QueryTimeoutError,
};

const MAX_DEPTH: u8 = 32;

impl Depth {
    pub fn new(value: u8) -> Result<Self, DepthError> {
        if value == 0 {
            return Err(DepthError::Zero);
        }
        if value > MAX_DEPTH {
            return Err(DepthError::TooDeep {
                requested: value,
                maximum: MAX_DEPTH,
            });
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn max() -> u8 {
        MAX_DEPTH
    }
}

impl QueryTimeout {
    pub fn new(value: Duration) -> Result<Self, QueryTimeoutError> {
        if value.is_zero() {
            return Err(QueryTimeoutError::Zero);
        }
        Ok(Self(value))
    }
}

impl QueryExecutionPolicy {
    pub fn validate_for_mode(
        &self,
        mode: QueryExecutionMode,
    ) -> Result<(), QueryExecutionPolicyError> {
        if matches!(mode, QueryExecutionMode::InteractiveSession) && !self.allow_partial_results {
            return Err(QueryExecutionPolicyError::InteractiveModeRequiresPartialResults { mode });
        }
        Ok(())
    }
}

impl LspRequestId {
    pub fn new(value: String) -> Result<Self, LspRequestIdError> {
        if value.trim().is_empty() {
            return Err(LspRequestIdError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MaxInFlightLspRequests {
    pub fn new(value: usize) -> Result<Self, MaxInFlightLspRequestsError> {
        if value == 0 {
            return Err(MaxInFlightLspRequestsError::Zero);
        }
        Ok(Self { value })
    }
}

impl LspRetryBudget {
    pub fn new(max_attempts: u8) -> Result<Self, LspRetryBudgetError> {
        if max_attempts == 0 {
            return Err(LspRetryBudgetError::Zero);
        }
        Ok(Self { max_attempts })
    }
}

impl LspRetryBackoffMs {
    pub fn new(value: u64) -> Result<Self, LspRetryBackoffError> {
        if value == 0 {
            return Err(LspRetryBackoffError::Zero);
        }
        Ok(Self { value })
    }
}

impl LspRetryJitterPercent {
    pub fn new(value: u8) -> Result<Self, LspRetryJitterError> {
        if value > 100 {
            return Err(LspRetryJitterError::OutOfRange);
        }
        Ok(Self { value })
    }
}

impl LspRetryDelayRange {
    pub fn new(
        min: LspRetryBackoffMs,
        max: LspRetryBackoffMs,
    ) -> Result<Self, LspRetryDelayRangeError> {
        if max.value < min.value {
            return Err(LspRetryDelayRangeError::InvalidRange);
        }
        Ok(Self { min, max })
    }
}

impl LspCompatibilityMatrix {
    pub fn validate(&self) -> Result<(), LspCompatibilityMatrixError> {
        if self.entries.is_empty() {
            return Err(LspCompatibilityMatrixError::Empty);
        }

        for (index, current) in self.entries.iter().enumerate() {
            if self
                .entries
                .iter()
                .skip(index + 1)
                .any(|other| is_duplicate_matrix_entry(current, other))
            {
                return Err(LspCompatibilityMatrixError::DuplicatedEntry);
            }
        }

        Ok(())
    }
}

fn is_duplicate_matrix_entry(a: &CompatibilityMatrixEntry, b: &CompatibilityMatrixEntry) -> bool {
    a.language == b.language && a.feature == b.feature
}
