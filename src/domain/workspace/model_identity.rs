use uuid::Uuid;

use crate::domain::workspace::IdentityConfidence;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceIdentity(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructuralFingerprint(pub(crate) String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityReliability {
    Strong,
    Weak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySource {
    Git { head_content: String },
    ExplicitFile { id: Uuid },
    StructuralHeuristic { confidence: IdentityConfidence },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIdentity {
    pub value: WorkspaceIdentity,
    pub source: IdentitySource,
    pub reliability: IdentityReliability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityVerificationStatus {
    Verified(ProjectIdentity),
    Unverifiable { reason: IdentityUnverifiableReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityUnverifiableReason {
    NoIdentitySource,
    FingerprintUnavailable,
    FileSystemUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityStrategy {
    GitHead,
    ExplicitIdFile,
    StructuralFingerprint,
    AlwaysRestart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityStrategyChain {
    pub ordered: Vec<IdentityStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityFallbackBehavior {
    RestartEveryInvocation,
    AllowDaemonReuseWithWarning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityPolicy {
    pub chain: IdentityStrategyChain,
    pub fallback_behavior: IdentityFallbackBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIdentityMismatch {
    pub previous: WorkspaceIdentity,
    pub current: WorkspaceIdentity,
}
