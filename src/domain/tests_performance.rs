#[cfg(test)]
mod performance_tests {
    use crate::domain::performance::*;
    use crate::domain::graph::GraphStatistics;

    #[test]
    fn graph_size_limit_validates_non_zero() {
        let result = GraphSizeLimit::new(0, 100, 10);
        assert!(result.is_err());

        let result = GraphSizeLimit::new(100, 0, 10);
        assert!(result.is_err());

        let result = GraphSizeLimit::new(100, 200, 0);
        assert!(result.is_err());
    }

    #[test]
    fn graph_size_limit_presets() {
        let unlimited = GraphSizeLimit::unlimited();
        assert!(unlimited.max_nodes.is_none());

        let conservative = GraphSizeLimit::conservative();
        assert_eq!(conservative.max_nodes, Some(1000));
        assert_eq!(conservative.max_edges, Some(5000));
    }

    #[test]
    fn graph_size_limit_checks_exceeded() {
        let limit = GraphSizeLimit::conservative();
        
        let small_stats = GraphStatistics {
            node_count: 500,
            edge_count: 2000,
            warning_count: 0,
            unresolved_references: crate::domain::graph::UnresolvedReferenceCount { value: 0 },
        };
        assert!(!limit.is_exceeded(&small_stats));

        let large_stats = GraphStatistics {
            node_count: 2000, // Exceeds conservative limit
            edge_count: 10000,
            warning_count: 0,
            unresolved_references: crate::domain::graph::UnresolvedReferenceCount { value: 0 },
        };
        assert!(limit.is_exceeded(&large_stats));
    }

    #[test]
    fn traversal_budget_presets() {
        let quick = TraversalBudget::quick();
        assert_eq!(quick.max_time_ms, 5_000);

        let standard = TraversalBudget::standard();
        assert_eq!(standard.max_time_ms, 30_000);

        let extensive = TraversalBudget::extensive();
        assert_eq!(extensive.max_time_ms, 300_000);
    }

    #[test]
    fn budget_tracker_monitors_usage() {
        let budget = TraversalBudget::quick();
        let mut tracker = BudgetTracker::new(budget);

        assert!(!tracker.is_exceeded());

        tracker.record_lsp_request();
        // Can't check private field, but can check behavior

        // Simulate exceeding request limit
        for _ in 0..60 {
            tracker.record_lsp_request();
        }
        assert!(tracker.request_limit_exceeded());
        assert!(tracker.is_exceeded());
    }

    #[test]
    fn memory_pressure_levels() {
        use MemoryPressureLevel::*;
        
        assert!(Critical.is_critical());
        assert!(!Normal.is_critical());

        assert!(High.is_high_or_critical());
        assert!(Critical.is_high_or_critical());
        assert!(!Moderate.is_high_or_critical());
    }

    #[test]
    fn memory_monitor_calculates_pressure() {
        let monitor = MemoryMonitor::new(1000);
        
        monitor.update_usage(500);
        assert_eq!(monitor.current_pressure(), MemoryPressureLevel::Normal);

        monitor.update_usage(700);
        assert_eq!(monitor.current_pressure(), MemoryPressureLevel::Moderate);

        monitor.update_usage(900);
        assert_eq!(monitor.current_pressure(), MemoryPressureLevel::High);

        monitor.update_usage(980);
        assert_eq!(monitor.current_pressure(), MemoryPressureLevel::Critical);
    }

    #[test]
    fn memory_monitor_recommends_actions() {
        let monitor = MemoryMonitor::new(100);

        monitor.update_usage(50);
        assert!(matches!(
            monitor.recommended_action(),
            PressureAction::Continue
        ));

        monitor.update_usage(90);
        assert!(matches!(
            monitor.recommended_action(),
            PressureAction::ShedLoad { .. }
        ));

        monitor.update_usage(98);
        assert!(matches!(
            monitor.recommended_action(),
            PressureAction::RejectNewWork
        ));
    }

    #[test]
    fn rate_limit_policy_presets() {
        let conservative = RateLimitPolicy::conservative();
        assert_eq!(conservative.requests_per_second, 10.0);

        let standard = RateLimitPolicy::standard();
        assert_eq!(standard.requests_per_second, 100.0);

        let unlimited = RateLimitPolicy::unlimited();
        assert!(unlimited.requests_per_second > 1000.0);
    }

    #[test]
    fn token_bucket_rate_limiter_enforces_limits() {
        let policy = RateLimitPolicy::new(2.0, 2); // 2 req/sec, burst of 2
        let limiter = TokenBucketRateLimiter::new(policy);

        // First 2 should succeed (burst)
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());

        // Third should fail (rate limited)
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn batching_policy_presets() {
        let eager = BatchingPolicy::eager();
        assert_eq!(eager.max_batch_size, 10);

        let balanced = BatchingPolicy::balanced();
        assert_eq!(balanced.max_batch_size, 100);

        let lazy = BatchingPolicy::lazy();
        assert_eq!(lazy.max_batch_size, 1000);
    }

    #[test]
    fn batching_policy_flush_logic() {
        let policy = BatchingPolicy::balanced();

        // Should not flush with small batch and short time
        assert!(!policy.should_flush(10, 10));

        // Should flush when batch size reached
        assert!(policy.should_flush(100, 10));

        // Should flush when time elapsed
        assert!(policy.should_flush(10, 100));
    }

    #[test]
    fn batching_policy_wait_logic() {
        let policy = BatchingPolicy::new(100, 1000, 10);

        assert!(policy.should_wait(5)); // Below minimum
        assert!(!policy.should_wait(10)); // At minimum
        assert!(!policy.should_wait(50)); // Above minimum
    }
}
