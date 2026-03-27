use crate::domain::interactive::{
    InteractiveCapabilities, JsonProtocolVersion, PhasedRequest, ProtocolCompatibility,
    ProtocolNegotiationError, ProtocolNegotiationResult, ProtocolTransition, SessionPhase,
    SessionPhaseError, SupportedProtocolVersions,
};

pub fn negotiate_protocol_version(
    server: &SupportedProtocolVersions,
    client: &SupportedProtocolVersions,
) -> Result<JsonProtocolVersion, ProtocolNegotiationError> {
    let mut candidates: Vec<JsonProtocolVersion> = server
        .versions
        .iter()
        .copied()
        .filter(|server_version| {
            client
                .versions
                .iter()
                .any(|client_version| client_version == server_version)
        })
        .collect();

    candidates.sort_by(|a, b| (a.major, a.minor).cmp(&(b.major, b.minor)));
    candidates
        .pop()
        .ok_or(ProtocolNegotiationError::NoCompatibleVersion)
}

pub fn finalize_protocol_negotiation(
    server: &SupportedProtocolVersions,
    client: &SupportedProtocolVersions,
    agreed_capabilities: InteractiveCapabilities,
) -> Result<ProtocolNegotiationResult, ProtocolNegotiationError> {
    let selected_version = negotiate_protocol_version(server, client)?;
    Ok(ProtocolNegotiationResult {
        selected_version,
        agreed_capabilities,
    })
}

pub fn validate_protocol_transition(
    transition: &ProtocolTransition,
) -> Result<(), ProtocolNegotiationError> {
    if matches!(transition.compatibility, ProtocolCompatibility::Breaking) {
        return Err(ProtocolNegotiationError::IncompatibleTransition {
            from: transition.from,
            to: transition.to,
            compatibility: transition.compatibility,
        });
    }
    Ok(())
}

pub fn validate_request_phase(
    phase: SessionPhase,
    request: &PhasedRequest,
) -> Result<(), SessionPhaseError> {
    match (phase, request) {
        (SessionPhase::PreInit, PhasedRequest::Ready(_)) => {
            Err(SessionPhaseError::QueryBeforeInit {
                phase,
                request_kind: "ready_request",
            })
        }
        (SessionPhase::Ready, PhasedRequest::PreInit(_)) => {
            Err(SessionPhaseError::InitOutsidePreInit {
                phase,
                request_kind: "pre_init_request",
            })
        }
        _ => Ok(()),
    }
}
