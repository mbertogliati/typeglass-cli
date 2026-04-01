use std::path::PathBuf;

use crate::domain::interactive::*;

#[test]
fn phase_validation_rejects_query_before_init() {
    let request = PhasedRequest::Ready(ReadyRequest::Exit);
    let result = validate_request_phase(SessionPhase::PreInit, &request);
    assert!(matches!(
        result,
        Err(SessionPhaseError::QueryBeforeInit { .. })
    ));
}

#[test]
fn phase_validation_rejects_init_after_ready() {
    let request = PhasedRequest::PreInit(PreInitRequest::Init {
        path: PathBuf::from("/tmp/project"),
    });
    let result = validate_request_phase(SessionPhase::Ready, &request);
    assert!(matches!(
        result,
        Err(SessionPhaseError::InitOutsidePreInit { .. })
    ));
}

#[test]
fn concurrency_policy_rejects_zero_in_flight() {
    let policy = SessionConcurrencyPolicy {
        allow_parallel_requests: true,
        max_in_flight_requests: 0,
    };
    let result = policy.validate();
    assert!(matches!(
        result,
        Err(SessionConcurrencyPolicyError::ZeroInFlight)
    ));
}

#[test]
fn input_size_limit_rejects_zero() {
    let result = InputSizeLimit::new(0);
    assert!(matches!(result, Err(InputSizeLimitError::Zero)));
}

#[test]
fn idle_timeout_rejects_zero() {
    let result = IdleTimeoutMs::new(0);
    assert!(matches!(result, Err(IdleTimeoutError::Zero)));
}

#[test]
fn protocol_version_rejects_zero_major() {
    let result = JsonProtocolVersion::new(0, 1);
    assert!(matches!(
        result,
        Err(JsonProtocolVersionError::InvalidMajor)
    ));
}

#[test]
fn supported_versions_reject_duplicates() {
    let v = JsonProtocolVersion::new(1, 0).expect("version must be valid");
    let result = SupportedProtocolVersions::new(vec![v, v]);
    assert!(matches!(
        result,
        Err(SupportedProtocolVersionsError::Duplicated)
    ));
}

#[test]
fn negotiation_selects_highest_compatible_version() {
    let server = SupportedProtocolVersions::new(vec![
        JsonProtocolVersion::new(1, 0).expect("version must be valid"),
        JsonProtocolVersion::new(1, 2).expect("version must be valid"),
        JsonProtocolVersion::new(2, 0).expect("version must be valid"),
    ])
    .expect("server versions must be valid");

    let client = SupportedProtocolVersions::new(vec![
        JsonProtocolVersion::new(1, 1).expect("version must be valid"),
        JsonProtocolVersion::new(1, 2).expect("version must be valid"),
    ])
    .expect("client versions must be valid");

    let result = negotiate_protocol_version(&server, &client).expect("must negotiate");
    assert_eq!(
        result,
        JsonProtocolVersion::new(1, 2).expect("version must be valid")
    );
}

#[test]
fn protocol_transition_rejects_breaking_compatibility() {
    let transition = ProtocolTransition {
        from: JsonProtocolVersion::new(1, 2).expect("version must be valid"),
        to: JsonProtocolVersion::new(2, 0).expect("version must be valid"),
        compatibility: ProtocolCompatibility::Breaking,
    };

    let result = validate_protocol_transition(&transition);
    assert!(matches!(
        result,
        Err(ProtocolNegotiationError::IncompatibleTransition { .. })
    ));
}
