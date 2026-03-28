use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::domain::graph::{TypeGraph, TraversalDirection};

/// Simple file-based cache for TypeGraph results
pub struct GraphCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    graph: TypeGraph,
    cached_at: SystemTime,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Failed to create cache directory: {0}")]
    DirectoryCreationFailed(std::io::Error),
    
    #[error("Failed to read cache file: {0}")]
    ReadFailed(std::io::Error),
    
    #[error("Failed to write cache file: {0}")]
    WriteFailed(std::io::Error),
    
    #[error("Failed to parse cache file: {0}")]
    ParseFailed(serde_json::Error),
    
    #[error("Cache entry expired")]
    Expired,
}

impl GraphCache {
    /// Create a new cache with 5 minute TTL
    pub fn new() -> Result<Self, CacheError> {
        // Use $HOME/.typeglass/cache
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                CacheError::DirectoryCreationFailed(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found (neither $HOME nor $USERPROFILE set)",
                ))
            })?;
        
        let cache_dir = PathBuf::from(home).join(".typeglass").join("cache");
        
        std::fs::create_dir_all(&cache_dir)
            .map_err(CacheError::DirectoryCreationFailed)?;
        
        Ok(Self {
            cache_dir,
            ttl: Duration::from_secs(300), // 5 minutes
        })
    }
    
    /// Get cached graph if available and not expired
    pub fn get(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
    ) -> Result<TypeGraph, CacheError> {
        let path = self.cache_path(symbol, depth, direction);
        
        let content = std::fs::read_to_string(&path)
            .map_err(CacheError::ReadFailed)?;
        
        let entry: CacheEntry = serde_json::from_str(&content)
            .map_err(CacheError::ParseFailed)?;
        
        // Check TTL
        let elapsed = SystemTime::now()
            .duration_since(entry.cached_at)
            .unwrap_or(Duration::from_secs(u64::MAX));
        
        if elapsed > self.ttl {
            // Clean up expired entry
            let _ = std::fs::remove_file(&path);
            return Err(CacheError::Expired);
        }
        
        Ok(entry.graph)
    }
    
    /// Store graph in cache
    pub fn set(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
        graph: &TypeGraph,
    ) -> Result<(), CacheError> {
        let path = self.cache_path(symbol, depth, direction);
        
        let entry = CacheEntry {
            graph: graph.clone(),
            cached_at: SystemTime::now(),
        };
        
        let content = serde_json::to_string_pretty(&entry)
            .map_err(CacheError::ParseFailed)?;
        
        std::fs::write(&path, content)
            .map_err(CacheError::WriteFailed)?;
        
        Ok(())
    }
    
    /// Generate cache file path
    fn cache_path(&self, symbol: &str, depth: usize, direction: TraversalDirection) -> PathBuf {
        // Safe filename from symbol (replace non-alphanumeric with _)
        let safe_symbol: String = symbol
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        
        let direction_str = match direction {
            TraversalDirection::Upstream => "up",
            TraversalDirection::Downstream => "down",
            TraversalDirection::Both => "both",
        };
        
        let filename = format!("{}_d{}_dir{}.json", safe_symbol, depth, direction_str);
        self.cache_dir.join(filename)
    }
    
    /// Invalidate all cache entries
    pub fn clear(&self) -> Result<(), CacheError> {
        std::fs::remove_dir_all(&self.cache_dir)
            .map_err(CacheError::WriteFailed)?;
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(CacheError::DirectoryCreationFailed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::{TypeNode, SymbolKind};
    use std::collections::HashMap;

    #[test]
    fn test_cache_path_generation() {
        let cache = GraphCache::new().unwrap();
        let path = cache.cache_path("TypeA", 2, TraversalDirection::Downstream);
        assert!(path.to_string_lossy().contains("TypeA_d2_dirdown.json"));
    }

    #[test]
    fn test_cache_miss() {
        let cache = GraphCache::new().unwrap();
        let result = cache.get("NonExistent", 1, TraversalDirection::Both);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_roundtrip() {
        let cache = GraphCache::new().unwrap();
        
        let graph = TypeGraph {
            nodes: HashMap::from([
                ("test".to_string(), TypeNode {
                    id: "test".to_string(),
                    symbol_kind: SymbolKind::Struct,
                    qualified_name: "Test".to_string(),
                    source_file: None,
                    metadata: HashMap::new(),
                }),
            ]),
            edges: HashMap::new(),
        };
        
        cache.set("Test", 1, TraversalDirection::Both, &graph).unwrap();
        let cached = cache.get("Test", 1, TraversalDirection::Both).unwrap();
        
        assert_eq!(cached.nodes.len(), 1);
        assert!(cached.nodes.contains_key("test"));
    }
}
