use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::graph::GraphStatistics;

// ============================================================================
// Graph Size Limit
// ============================================================================

/// Limits on graph size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSizeLimit {
    pub max_nodes: Option<usize>,
    pub max_edges: Option<usize>,
    pub max_depth: u8,
}

impl GraphSizeLimit {
    pub fn new(max_nodes: usize, max_edges: usize, max_depth: u8) -> Result<Self, GraphSizeLimitError> {
        if max_nodes == 0 {
            return Err(GraphSizeLimitError::MaxNodesZero);
        }
        if max_edges == 0 {
            return Err(GraphSizeLimitError::MaxEdgesZero);
        }
        if max_depth == 0 {
            return Err(GraphSizeLimitError::MaxDepthZero);
        }
        Ok(Self {
            max_nodes: Some(max_nodes),
            max_edges: Some(max_edges),
            max_depth,
        })
    }

    pub fn unlimited() -> Self {
        Self {
            max_nodes: None,
            max_edges: None,
            max_depth: 255,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_nodes: Some(1000),
            max_edges: Some(5000),
            max_depth: 10,
        }
    }

    pub fn large() -> Self {
        Self {
            max_nodes: Some(10_000),
            max_edges: Some(50_000),
            max_depth: 20,
        }
    }

    pub fn is_exceeded(&self, stats: &GraphStatistics) -> bool {
        if let Some(max_nodes) = self.max_nodes {
            if stats.node_count > max_nodes {
                return true;
            }
        }
        if let Some(max_edges) = self.max_edges {
            if stats.edge_count > max_edges {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Error)]
pub enum GraphSizeLimitError {
    #[error("Max nodes must be greater than zero")]
    MaxNodesZero,
    #[error("Max edges must be greater than zero")]
    MaxEdgesZero,
    #[error("Max depth must be greater than zero")]
    MaxDepthZero,
}

// ============================================================================
// Traversal Budget
// ============================================================================

/// Resource budget for traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalBudget {
    pub max_time_ms: u64,
    pub max_memory_mb: usize,
    pub max_lsp_requests: usize,
}

impl TraversalBudget {
    pub fn new(max_time_ms: u64, max_memory_mb: usize, max_lsp_requests: usize) -> Self {
        Self {
            max_time_ms,
            max_memory_mb,
            max_lsp_requests,
        }
    }

    pub fn quick() -> Self {
        Self {
            max_time_ms: 5_000,
            max_memory_mb: 100,
            max_lsp_requests: 50,
        }
    }

    pub fn standard() -> Self {
        Self {
            max_time_ms: 30_000,
            max_memory_mb: 500,
            max_lsp_requests: 500,
        }
    }

    pub fn extensive() -> Self {
        Self {
            max_time_ms: 300_000,
            max_memory_mb: 2000,
            max_lsp_requests: 5000,
        }
    }
}

/// Tracks budget usage during traversal
#[derive(Debug)]
pub struct BudgetTracker {
    budget: TraversalBudget,
    start_time: Instant,
    lsp_requests_made: usize,
}

impl BudgetTracker {
    pub fn new(budget: TraversalBudget) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            lsp_requests_made: 0,
        }
    }

    pub fn is_exceeded(&self) -> bool {
        self.time_exceeded() || self.request_limit_exceeded()
    }

    pub fn time_exceeded(&self) -> bool {
        self.elapsed_ms() > self.budget.max_time_ms
    }

    pub fn request_limit_exceeded(&self) -> bool {
        self.lsp_requests_made >= self.budget.max_lsp_requests
    }

    pub fn remaining_time_ms(&self) -> u64 {
        self.budget.max_time_ms.saturating_sub(self.elapsed_ms())
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn record_lsp_request(&mut self) {
        self.lsp_requests_made += 1;
    }

    pub fn check(&self) -> Result<(), BudgetExceeded> {
        if self.time_exceeded() {
            return Err(BudgetExceeded::TimeLimit {
                elapsed_ms: self.elapsed_ms(),
                limit_ms: self.budget.max_time_ms,
            });
        }
        if self.request_limit_exceeded() {
            return Err(BudgetExceeded::RequestLimit {
                made: self.lsp_requests_made,
                limit: self.budget.max_lsp_requests,
            });
        }
        Ok(())
    }
}

/// Budget exceeded reason
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BudgetExceeded {
    #[error("Time limit exceeded: {elapsed_ms}ms (limit: {limit_ms}ms)")]
    TimeLimit { elapsed_ms: u64, limit_ms: u64 },
    #[error("Memory limit exceeded: {used_mb}MB (limit: {limit_mb}MB)")]
    MemoryLimit { used_mb: usize, limit_mb: usize },
    #[error("Request limit exceeded: {made} requests (limit: {limit})")]
    RequestLimit { made: usize, limit: usize },
}

// ============================================================================
// Memory Pressure
// ============================================================================

/// Memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Normal,
    Moderate,  // 60-80% of limit
    High,      // 80-95% of limit
    Critical,  // >95% of limit
}

impl MemoryPressureLevel {
    pub fn is_critical(&self) -> bool {
        matches!(self, MemoryPressureLevel::Critical)
    }

    pub fn is_high_or_critical(&self) -> bool {
        matches!(
            self,
            MemoryPressureLevel::High | MemoryPressureLevel::Critical
        )
    }
}

/// Action to take under memory pressure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureAction {
    Continue,
    ShedLoad { drop_caches: bool },
    RejectNewWork,
    EmergencyShutdown,
}

/// Monitors memory usage
#[derive(Debug, Clone)]
pub struct MemoryMonitor {
    limit_mb: usize,
    current_mb: Arc<Mutex<usize>>,
}

impl MemoryMonitor {
    pub fn new(limit_mb: usize) -> Self {
        Self {
            limit_mb,
            current_mb: Arc::new(Mutex::new(0)),
        }
    }

    pub fn update_usage(&self, mb: usize) {
        if let Ok(mut current) = self.current_mb.lock() {
            *current = mb;
        }
    }

    pub fn current_pressure(&self) -> MemoryPressureLevel {
        let current = self.current_mb.lock().map(|c| *c).unwrap_or(0);
        let percentage = (current as f64 / self.limit_mb as f64) * 100.0;

        match percentage as u8 {
            0..=59 => MemoryPressureLevel::Normal,
            60..=79 => MemoryPressureLevel::Moderate,
            80..=94 => MemoryPressureLevel::High,
            _ => MemoryPressureLevel::Critical,
        }
    }

    pub fn should_shed_load(&self) -> bool {
        matches!(
            self.current_pressure(),
            MemoryPressureLevel::High | MemoryPressureLevel::Critical
        )
    }

    pub fn should_trigger_gc(&self) -> bool {
        matches!(
            self.current_pressure(),
            MemoryPressureLevel::Moderate | MemoryPressureLevel::High
        )
    }

    pub fn recommended_action(&self) -> PressureAction {
        match self.current_pressure() {
            MemoryPressureLevel::Normal => PressureAction::Continue,
            MemoryPressureLevel::Moderate => PressureAction::Continue,
            MemoryPressureLevel::High => PressureAction::ShedLoad { drop_caches: true },
            MemoryPressureLevel::Critical => PressureAction::RejectNewWork,
        }
    }
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limit policy (token bucket)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RateLimitPolicy {
    pub requests_per_second: f64,
    pub burst_size: usize,
}

impl RateLimitPolicy {
    pub fn new(requests_per_second: f64, burst_size: usize) -> Self {
        Self {
            requests_per_second,
            burst_size,
        }
    }

    pub fn conservative() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 20,
        }
    }

    pub fn standard() -> Self {
        Self {
            requests_per_second: 100.0,
            burst_size: 200,
        }
    }

    pub fn unlimited() -> Self {
        Self {
            requests_per_second: f64::MAX,
            burst_size: usize::MAX,
        }
    }
}

/// Token bucket rate limiter
#[derive(Debug)]
pub struct TokenBucketRateLimiter {
    policy: RateLimitPolicy,
    tokens: Arc<Mutex<f64>>,
    last_update: Arc<Mutex<Instant>>,
}

impl TokenBucketRateLimiter {
    pub fn new(policy: RateLimitPolicy) -> Self {
        Self {
            policy,
            tokens: Arc::new(Mutex::new(policy.burst_size as f64)),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn try_acquire(&self) -> Result<(), RateLimitExceeded> {
        let mut tokens = self.tokens.lock().unwrap();
        let mut last_update = self.last_update.lock().unwrap();

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        let tokens_to_add = elapsed * self.policy.requests_per_second;
        *tokens = (*tokens + tokens_to_add).min(self.policy.burst_size as f64);
        *last_update = now;

        // Try to consume a token
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            let wait_time = ((1.0 - *tokens) / self.policy.requests_per_second) * 1000.0;
            Err(RateLimitExceeded {
                retry_after_ms: wait_time as u64,
            })
        }
    }
}

/// Rate limit exceeded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Rate limit exceeded, retry after {retry_after_ms}ms")]
pub struct RateLimitExceeded {
    pub retry_after_ms: u64,
}

// ============================================================================
// Batching Policy
// ============================================================================

/// Policy for batching operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchingPolicy {
    pub max_batch_size: usize,
    pub max_wait_ms: u64,
    pub min_batch_size: usize,
}

impl BatchingPolicy {
    pub fn new(max_batch_size: usize, max_wait_ms: u64, min_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            max_wait_ms,
            min_batch_size,
        }
    }

    pub fn eager() -> Self {
        Self {
            max_batch_size: 10,
            max_wait_ms: 10,
            min_batch_size: 1,
        }
    }

    pub fn balanced() -> Self {
        Self {
            max_batch_size: 100,
            max_wait_ms: 100,
            min_batch_size: 10,
        }
    }

    pub fn lazy() -> Self {
        Self {
            max_batch_size: 1000,
            max_wait_ms: 1000,
            min_batch_size: 50,
        }
    }

    pub fn should_flush(&self, current_size: usize, elapsed_ms: u64) -> bool {
        current_size >= self.max_batch_size || elapsed_ms >= self.max_wait_ms
    }

    pub fn should_wait(&self, current_size: usize) -> bool {
        current_size < self.min_batch_size
    }
}
