use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageFeature {
    TypeGraphTraversal,
    SymbolReferences,
    FileEntryPoint,
    IncrementalInvalidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureAvailability {
    Available,
    Unavailable { reason: FeatureUnavailableReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureUnavailableReason {
    MissingLspCapability,
    UnsupportedLanguage,
    ExperimentalDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityMatrixEntry {
    pub language: crate::domain::language::Language,
    pub feature: LanguageFeature,
    pub availability: FeatureAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCompatibilityMatrix {
    pub entries: Vec<CompatibilityMatrixEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConstraint {
    pub minimum: String,
    pub maximum_exclusive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCapabilities {
    pub supports_document_symbols: bool,
    pub supports_references: bool,
    pub supports_definition: bool,
    pub supports_hover: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspIndexingState {
    NotStarted,
    InProgress { scanned_files: usize },
    Ready,
    Partial { unresolved_count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspServerStatus {
    Starting,
    Indexing(LspIndexingState),
    Ready { capabilities: LspCapabilities },
    Degraded { reason: LspDegradedReason },
    Unavailable { reason: LspUnavailableReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspUnavailableReason {
    BinaryMissing,
    StartupFailed,
    HandshakeFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspDegradedReason {
    VersionMismatch,
    WorkspaceErrors,
    PartialIndex,
}
