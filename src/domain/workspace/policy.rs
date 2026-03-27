#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsCaseSensitivity {
    Sensitive,
    Insensitive,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkPolicy {
    FollowWithCycleDetection,
    DoNotFollow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathNormalizationPolicy {
    Canonicalize,
    PreserveInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFileSystemPolicy {
    pub symlink_policy: SymlinkPolicy,
    pub normalization_policy: PathNormalizationPolicy,
    pub case_sensitivity: FsCaseSensitivity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRootSelection {
    Auto,
    Explicit(WorkspacePath),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityConfidence {
    High,
    Low,
}
use super::model::WorkspacePath;
