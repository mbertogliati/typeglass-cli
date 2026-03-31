#[cfg(test)]
mod recovery_tests {
    use crate::domain::recovery::*;
    

    #[test]
    fn degradation_level_identifies_states() {
        let full = DegradationLevel::Full;
        assert!(full.is_full());
        assert!(!full.is_degraded());

        let partial = DegradationLevel::Partial;
        assert!(!partial.is_full());
        assert!(partial.is_degraded());
    }

    #[test]
    fn service_degradation_status_presets() {
        let full = ServiceDegradationStatus::full();
        assert_eq!(full.level, DegradationLevel::Full);
        assert!(full.unavailable_features.is_empty());

        let minimal = ServiceDegradationStatus::minimal(DegradationReason::LspNotResponding);
        assert_eq!(minimal.level, DegradationLevel::MinimalFallback);
        assert!(!minimal.unavailable_features.is_empty());
    }

    #[test]
    fn fallback_strategy_constructors() {
        let cache_strat = FallbackStrategy::cache_with_age(5000);
        assert!(matches!(cache_strat, FallbackStrategy::UseCache { max_age_ms: 5000 }));

        let partial_strat = FallbackStrategy::partial_with_threshold(80);
        assert!(matches!(
            partial_strat,
            FallbackStrategy::UsePartial { min_completeness: 80 }
        ));
    }

    #[test]
    fn fallback_policy_chain_presets() {
        let conservative = FallbackPolicyChain::conservative();
        assert!(!conservative.strategies.is_empty());

        let aggressive = FallbackPolicyChain::aggressive();
        assert!(!aggressive.strategies.is_empty());
    }

    #[test]
    fn recovery_suggestion_types() {
        let auto = RecoverySuggestion::retry(1000, "Retry after 1s".to_string());
        assert!(auto.automatic);

        let manual = RecoverySuggestion::manual(
            RecoveryAction::UseCached,
            "Use cached data".to_string(),
        );
        assert!(!manual.automatic);
    }

    #[test]
    fn recoverable_error_with_partial() {
        let error = RecoverableError::with_partial("Failed".to_string(), 42);
        assert!(error.has_partial_result());
        assert_eq!(error.partial_result, Some(42));
    }

    #[test]
    fn recoverable_error_suggestions() {
        let error = RecoverableError::<()>::new(Box::new(std::io::Error::other(
            "test",
        )))
        .suggest(RecoverySuggestion::retry(100, "Retry".to_string()));

        assert!(error.has_automatic_recovery());
        assert_eq!(error.recovery_suggestions.len(), 1);
    }

    #[test]
    fn retryable_failure_tracking() {
        let mut failure = RetryableFailure::new("Error".to_string(), 100, 3);
        assert_eq!(failure.attempts_so_far, 0);
        assert!(failure.should_retry());

        failure.record_attempt();
        assert_eq!(failure.attempts_so_far, 1);

        failure.record_attempt();
        failure.record_attempt();
        assert!(!failure.should_retry()); // max retries reached
    }

    #[test]
    fn permanent_failure_with_remediation() {
        let failure = PermanentFailure::new("Fatal".to_string())
            .with_remediation("Run doctor command".to_string());

        assert_eq!(failure.remediation, Some("Run doctor command".to_string()));
    }

    #[test]
    fn graceful_shutdown_configuration() {
        let shutdown = GracefulShutdown::new(ShutdownReason::UserRequested)
            .with_timeout(10000)
            .with_steps(GracefulShutdown::default_steps());

        assert_eq!(shutdown.timeout_ms, 10000);
        assert!(!shutdown.cleanup_steps.is_empty());
    }

    #[test]
    fn shutdown_result_states() {
        let clean = ShutdownResult::Clean;
        assert!(clean.is_clean());
        assert!(!clean.has_failures());

        let timeout = ShutdownResult::TimedOut {
            completed: vec![],
            pending: vec![CleanupStep::FlushCache],
        };
        assert!(!timeout.is_clean());
        assert!(timeout.has_failures());
    }
}
