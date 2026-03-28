use crate::domain::graph::{TraversalDirection, TypeGraph};

/// Cache Repository - Output port for graph caching
/// 
/// Implemented by infrastructure/persistence adapters.
/// Provides caching operations for type graphs.
pub trait CacheRepository: Send + Sync {
    /// Store a type graph in cache
    fn set(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
        graph: &TypeGraph,
    ) -> Result<(), CacheError>;

    /// Retrieve a cached type graph
    fn get(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
    ) -> Result<Option<TypeGraph>, CacheError>;

    /// Check if a graph is cached
    fn has(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
    ) -> bool;

    /// Clear expired cache entries
    fn clear_expired(&self) -> Result<usize, CacheError>;

    /// Clear all cache entries
    fn clear_all(&self) -> Result<(), CacheError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Failed to write cache: {0}")]
    WriteFailed(String),

    #[error("Failed to read cache: {0}")]
    ReadFailed(String),

    #[error("Cache entry corrupted: {0}")]
    Corrupted(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
