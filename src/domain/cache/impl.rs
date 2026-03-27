use super::types::{
    CacheFallbackDecision, CacheFallbackError, CacheFallbackPolicy, CacheNamespace,
    CacheNamespaceError, CacheReadOutcome, LspQueryOutcome, MaxCacheAgeError, MaxCacheAgeMs,
    PolicyHash, PolicyHashError,
};

impl PolicyHash {
    pub fn new(value: String) -> Result<Self, PolicyHashError> {
        if value.trim().is_empty() {
            return Err(PolicyHashError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MaxCacheAgeMs {
    pub fn new(value: u64) -> Result<Self, MaxCacheAgeError> {
        if value == 0 {
            return Err(MaxCacheAgeError::Zero);
        }
        Ok(Self { value })
    }
}

impl CacheNamespace {
    pub fn validate(&self) -> Result<(), CacheNamespaceError> {
        if self.lsp_version.trim().is_empty() {
            return Err(CacheNamespaceError::EmptyLspVersion);
        }
        Ok(())
    }
}

pub fn decide_cache_fallback(
    policy: CacheFallbackPolicy,
    cache_outcome: CacheReadOutcome,
    lsp_outcome: LspQueryOutcome,
) -> Result<CacheFallbackDecision, CacheFallbackError> {
    match (cache_outcome, lsp_outcome) {
        (CacheReadOutcome::HitFresh, _) => Ok(CacheFallbackDecision::UseFreshCache),
        (CacheReadOutcome::HitStale, LspQueryOutcome::Failed) if policy.allow_stale_cache => {
            Ok(CacheFallbackDecision::UseStaleCache)
        }
        (CacheReadOutcome::Miss, LspQueryOutcome::Success)
        | (CacheReadOutcome::ReadError { .. }, LspQueryOutcome::Success)
        | (CacheReadOutcome::HitStale, LspQueryOutcome::Success) => {
            Ok(CacheFallbackDecision::QueryLsp)
        }
        (_, LspQueryOutcome::Failed) if policy.allow_fail_open_on_lsp_error => {
            Ok(CacheFallbackDecision::Fail)
        }
        (cache_outcome, lsp_outcome) => Err(CacheFallbackError::NoFallbackAvailable {
            cache_outcome,
            lsp_outcome,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::Language;
    use crate::domain::workspace::WorkspaceIdentity;

    #[test]
    fn policy_hash_rejects_empty() {
        let result = PolicyHash::new(String::new());
        assert!(matches!(result, Err(PolicyHashError::Empty)));
    }

    #[test]
    fn max_cache_age_rejects_zero() {
        let result = MaxCacheAgeMs::new(0);
        assert!(matches!(result, Err(MaxCacheAgeError::Zero)));
    }

    #[test]
    fn cache_namespace_requires_lsp_version() {
        let namespace = CacheNamespace {
            workspace_identity: WorkspaceIdentity("wk-1".to_string()),
            language: Language::Rust,
            lsp_version: String::new(),
            policy_hash: PolicyHash("abc".to_string()),
        };
        let result = namespace.validate();
        assert!(matches!(result, Err(CacheNamespaceError::EmptyLspVersion)));
    }

    #[test]
    fn fallback_uses_stale_cache_when_allowed_and_lsp_fails() {
        let decision = decide_cache_fallback(
            CacheFallbackPolicy {
                allow_stale_cache: true,
                allow_fail_open_on_lsp_error: false,
            },
            CacheReadOutcome::HitStale,
            LspQueryOutcome::Failed,
        )
        .expect("fallback must resolve");

        assert!(matches!(decision, CacheFallbackDecision::UseStaleCache));
    }
}
