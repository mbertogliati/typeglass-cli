use crate::domain::lsp::*;

#[test]
fn lsp_request_id_cannot_be_empty() {
    let result = LspRequestId::new(String::new());
    assert!(matches!(result, Err(LspRequestIdError::Empty)));
}

#[test]
fn max_in_flight_requests_cannot_be_zero() {
    let result = MaxInFlightLspRequests::new(0);
    assert!(matches!(result, Err(MaxInFlightLspRequestsError::Zero)));
}

#[test]
fn cancelled_error_is_non_retryable() {
    let error = LspError::Cancelled {
        request_id: LspRequestId("abc".to_string()),
    };
    assert!(matches!(error.retryability(), Retryability::NonRetryable));
}

#[test]
fn lsp_retry_budget_cannot_be_zero() {
    let result = LspRetryBudget::new(0);
    assert!(matches!(result, Err(LspRetryBudgetError::Zero)));
}

#[test]
fn lsp_retry_delay_range_rejects_inverted_values() {
    let min = LspRetryBackoffMs::new(500).expect("min must be valid");
    let max = LspRetryBackoffMs::new(100).expect("max must be valid");
    let result = LspRetryDelayRange::new(min, max);
    assert!(matches!(result, Err(LspRetryDelayRangeError::InvalidRange)));
}

#[test]
fn compatibility_matrix_rejects_duplicates() {
    let entry = CompatibilityMatrixEntry {
        language: crate::domain::language::Language::Rust,
        feature: crate::domain::lsp::LanguageFeature::TypeGraphTraversal,
        availability: crate::domain::lsp::FeatureAvailability::Available,
    };

    let matrix = LspCompatibilityMatrix {
        entries: vec![entry.clone(), entry],
    };

    let result = matrix.validate();
    assert!(matches!(
        result,
        Err(LspCompatibilityMatrixError::DuplicatedEntry)
    ));
}
