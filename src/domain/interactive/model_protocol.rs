use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JsonProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedProtocolVersions {
    pub versions: Vec<JsonProtocolVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractiveCapabilities {
    pub supports_cancel: bool,
    pub supports_partial_results: bool,
    pub supports_graph_statistics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolNegotiationRequest {
    pub versions: SupportedProtocolVersions,
    pub capabilities: InteractiveCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolNegotiationResult {
    pub selected_version: JsonProtocolVersion,
    pub agreed_capabilities: InteractiveCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolCompatibility {
    BackwardCompatible,
    ForwardCompatible,
    Breaking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolTransition {
    pub from: JsonProtocolVersion,
    pub to: JsonProtocolVersion,
    pub compatibility: ProtocolCompatibility,
}
