use thiserror::Error;

use crate::domain::language::Language;
use crate::domain::workspace::{WorkspaceFile, WorkspaceIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolicyHash(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheNamespace {
    pub workspace_identity: WorkspaceIdentity,
    pub language: Language,
    pub lsp_version: String,
    pub policy_hash: PolicyHash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEpoch {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheFreshness {
    Fresh,
    Stale { reason: CacheStaleReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStaleReason {
    IdentityChanged,
    LspVersionChanged,
    PolicyChanged,
    MissingMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationScope {
    FileSet(Vec<WorkspaceFile>),
    WholeWorkspace,
    LanguageOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxCacheAgeMs {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheReadPolicy {
    pub max_age: MaxCacheAgeMs,
    pub allow_stale_on_lsp_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheReadOutcome {
    HitFresh,
    HitStale,
    Miss,
    ReadError { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspQueryOutcome {
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheFallbackDecision {
    UseFreshCache,
    UseStaleCache,
    QueryLsp,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheFallbackPolicy {
    pub allow_stale_cache: bool,
    pub allow_fail_open_on_lsp_error: bool,
}

#[derive(Debug, Error)]
pub enum CacheFallbackError {
    #[error("No valid fallback decision for cache/LSP combined failure. Cache outcome: {cache_outcome:?}, LSP outcome: {lsp_outcome:?}")]
    NoFallbackAvailable {
        cache_outcome: CacheReadOutcome,
        lsp_outcome: LspQueryOutcome,
    },
}

#[derive(Debug, Error)]
pub enum PolicyHashError {
    #[error("Policy hash cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum CacheNamespaceError {
    #[error("LSP version cannot be empty")]
    EmptyLspVersion,
}

#[derive(Debug, Error)]
pub enum MaxCacheAgeError {
    #[error("Max cache age must be greater than zero")]
    Zero,
}
