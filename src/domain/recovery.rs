use std::error::Error;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::domain::lsp::LanguageFeature;

// ============================================================================
// Degradation Level
// ============================================================================

/// Level of service degradation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationLevel {
    /// All features working
    Full,
    /// Some features unavailable
    Partial,
    /// Only cached/basic features
    MinimalFallback,
}

impl DegradationLevel {
    pub fn is_full(&self) -> bool {
        matches!(self, DegradationLevel::Full)
    }

    pub fn is_degraded(&self) -> bool {
        !self.is_full()
    }
}

/// Reason for degradation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationReason {
    LspNotResponding,
    LspCrashed { exit_code: i32 },
    WorkspaceTooLarge { node_count: usize },
    RateLimitExceeded,
}

/// Service degradation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDegradationStatus {
    pub level: DegradationLevel,
    pub unavailable_features: Vec<LanguageFeature>,
    pub reason: DegradationReason,
}

impl ServiceDegradationStatus {
    pub fn full() -> Self {
        Self {
            level: DegradationLevel::Full,
            unavailable_features: Vec::new(),
            reason: DegradationReason::LspNotResponding, // placeholder
        }
    }

    pub fn partial(reason: DegradationReason, unavailable: Vec<LanguageFeature>) -> Self {
        Self {
            level: DegradationLevel::Partial,
            unavailable_features: unavailable,
            reason,
        }
    }

    pub fn minimal(reason: DegradationReason) -> Self {
        Self {
            level: DegradationLevel::MinimalFallback,
            unavailable_features: vec![
                LanguageFeature::TypeGraphTraversal,
                LanguageFeature::SymbolReferences,
                LanguageFeature::FileEntryPoint,
            ],
            reason,
        }
    }
}

// ============================================================================
// Fallback Strategy
// ============================================================================

/// Strategy for fallback when primary operation fails
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Use cached data (max age)
    UseCache { max_age_ms: u64 },
    /// Use partial result (min completeness)
    UsePartial { min_completeness: u8 },
    /// Return empty result
    ReturnEmpty,
    /// Fail and propagate error
    Fail,
}

impl FallbackStrategy {
    pub fn cache_with_age(max_age_ms: u64) -> Self {
        Self::UseCache { max_age_ms }
    }

    pub fn partial_with_threshold(min_completeness: u8) -> Self {
        Self::UsePartial {
            min_completeness: min_completeness.min(100),
        }
    }
}

/// Chain of fallback strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackPolicyChain {
    pub strategies: Vec<FallbackStrategy>,
}

impl FallbackPolicyChain {
    pub fn new(strategies: Vec<FallbackStrategy>) -> Self {
        Self { strategies }
    }

    pub fn conservative() -> Self {
        Self {
            strategies: vec![
                FallbackStrategy::UseCache { max_age_ms: 60_000 },
                FallbackStrategy::UsePartial { min_completeness: 80 },
                FallbackStrategy::Fail,
            ],
        }
    }

    pub fn aggressive() -> Self {
        Self {
            strategies: vec![
                FallbackStrategy::UseCache { max_age_ms: 300_000 },
                FallbackStrategy::UsePartial { min_completeness: 50 },
                FallbackStrategy::ReturnEmpty,
            ],
        }
    }
}

/// Result of fallback decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackDecision {
    UseCached { age_ms: u64 },
    UsePartial { completeness: u8 },
    Empty,
    PropagateFailure,
}

// ============================================================================
// Recoverable Error
// ============================================================================

/// Recovery action suggestion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    Retry { after_ms: u64 },
    UseCached,
    ReduceScope,
    SkipItem,
}

/// Recovery suggestion with context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    pub action: RecoveryAction,
    pub description: String,
    pub automatic: bool,
}

impl RecoverySuggestion {
    pub fn retry(after_ms: u64, description: String) -> Self {
        Self {
            action: RecoveryAction::Retry { after_ms },
            description,
            automatic: true,
        }
    }

    pub fn manual(action: RecoveryAction, description: String) -> Self {
        Self {
            action,
            description,
            automatic: false,
        }
    }
}

/// Error with recovery options and partial results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverableError<T> {
    pub partial_result: Option<T>,
    pub error_message: String,
    pub recovery_suggestions: Vec<RecoverySuggestion>,
}

impl<T> RecoverableError<T> {
    pub fn new(error: Box<dyn Error>) -> Self {
        Self {
            partial_result: None,
            error_message: error.to_string(),
            recovery_suggestions: Vec::new(),
        }
    }

    pub fn with_partial(error_message: String, partial: T) -> Self {
        Self {
            partial_result: Some(partial),
            error_message,
            recovery_suggestions: Vec::new(),
        }
    }

    pub fn suggest(mut self, suggestion: RecoverySuggestion) -> Self {
        self.recovery_suggestions.push(suggestion);
        self
    }

    pub fn has_partial_result(&self) -> bool {
        self.partial_result.is_some()
    }

    pub fn has_automatic_recovery(&self) -> bool {
        self.recovery_suggestions.iter().any(|s| s.automatic)
    }
}

// ============================================================================
// Retryable vs Permanent
// ============================================================================

/// Trait for classifying errors
pub trait ErrorClassification {
    fn is_retryable(&self) -> bool;
    fn is_permanent(&self) -> bool;
    fn recommended_retry_delay(&self) -> Option<Duration>;
}

/// Category of failure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureCategory {
    Retryable(RetryableFailure),
    Permanent(PermanentFailure),
}

/// Failure that can be retried
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryableFailure {
    pub error_message: String,
    pub retry_after_ms: u64,
    pub max_retries: u8,
    pub attempts_so_far: u8,
}

impl RetryableFailure {
    pub fn new(error_message: String, retry_after_ms: u64, max_retries: u8) -> Self {
        Self {
            error_message,
            retry_after_ms,
            max_retries,
            attempts_so_far: 0,
        }
    }

    pub fn should_retry(&self) -> bool {
        self.attempts_so_far < self.max_retries
    }

    pub fn record_attempt(&mut self) {
        self.attempts_so_far += 1;
    }
}

/// Permanent failure that cannot be retried
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermanentFailure {
    pub error_message: String,
    pub remediation: Option<String>,
}

impl PermanentFailure {
    pub fn new(error_message: String) -> Self {
        Self {
            error_message,
            remediation: None,
        }
    }

    pub fn with_remediation(mut self, remediation: String) -> Self {
        self.remediation = Some(remediation);
        self
    }
}

// ============================================================================
// Graceful Shutdown
// ============================================================================

/// Reason for shutdown
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownReason {
    UserRequested,
    Idle { timeout_exceeded: Duration },
    Error { fatal_error: String },
    Upgrade { new_version: String },
}

/// Cleanup step during shutdown
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupStep {
    FlushCache,
    CloseLspConnection,
    SaveState { path: String },
    RemoveLockFile { path: String },
    KillChildProcesses,
}

/// Orchestrated graceful shutdown
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GracefulShutdown {
    pub reason: ShutdownReason,
    pub timeout_ms: u64,
    pub cleanup_steps: Vec<CleanupStep>,
}

impl GracefulShutdown {
    pub fn new(reason: ShutdownReason) -> Self {
        Self {
            reason,
            timeout_ms: 5000,
            cleanup_steps: Vec::new(),
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_steps(mut self, steps: Vec<CleanupStep>) -> Self {
        self.cleanup_steps = steps;
        self
    }

    pub fn default_steps() -> Vec<CleanupStep> {
        vec![
            CleanupStep::FlushCache,
            CleanupStep::CloseLspConnection,
            CleanupStep::KillChildProcesses,
        ]
    }
}

/// Result of shutdown
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownResult {
    Clean,
    TimedOut {
        completed: Vec<CleanupStep>,
        pending: Vec<CleanupStep>,
    },
    PartialFailure {
        failed: Vec<(CleanupStep, String)>,
    },
}

impl ShutdownResult {
    pub fn is_clean(&self) -> bool {
        matches!(self, ShutdownResult::Clean)
    }

    pub fn has_failures(&self) -> bool {
        matches!(
            self,
            ShutdownResult::TimedOut { .. } | ShutdownResult::PartialFailure { .. }
        )
    }
}
