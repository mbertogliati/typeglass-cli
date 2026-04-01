use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::domain::graph::{
    TraversalDirection, TypeGraph,
};

/// Simple file-based cache for TypeGraph results
pub struct GraphCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    graph: TypeGraph,
    cached_at: SystemTime,
    /// Files that were analyzed to build this graph (with their hashes)
    source_files: Vec<SourceFileInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SourceFileInfo {
    path: PathBuf,
    /// Simple hash of file modification time
    mtime_hash: u64,
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
    
    /// Store graph in cache with source file tracking
    pub fn set(
        &self,
        symbol: &str,
        depth: usize,
        direction: TraversalDirection,
        graph: &TypeGraph,
    ) -> Result<(), CacheError> {
        let path = self.cache_path(symbol, depth, direction);
        
        // Extract source files from graph nodes
        let source_files = extract_source_files(graph);
        
        let entry = CacheEntry {
            graph: graph.clone(),
            cached_at: SystemTime::now(),
            source_files,
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
    
    /// Invalidate cache entries for specific files (selective invalidation)
    /// Removes only cache entries that depend on the modified files
    pub fn clear_for_files(&self, files: &[PathBuf]) -> Result<usize, CacheError> {
        if files.is_empty() {
            return Ok(0);
        }
        
        let mut invalidated_count = 0;
        let changed_files: HashSet<PathBuf> = files.iter().cloned().collect();
        
        // Scan all cache files
        let entries = match std::fs::read_dir(&self.cache_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(0), // No cache dir, nothing to invalidate
        };
        
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            
            // Read cache entry to check source files
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                    // Check if any source file has changed
                    let should_invalidate = cache_entry.source_files.iter().any(|sf| {
                        // Check if file is in changed set
                        if !changed_files.contains(&sf.path) {
                            return false;
                        }
                        
                        // Check if file modification time changed
                        match get_file_mtime_hash(&sf.path) {
                            Ok(current_hash) => current_hash != sf.mtime_hash,
                            Err(_) => true, // File doesn't exist or can't be read, invalidate
                        }
                    });
                    
                    if should_invalidate {
                        if std::fs::remove_file(&path).is_ok() {
                            invalidated_count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(invalidated_count)
    }
    
    /// Count cache entries (for metrics)
    fn count_entries(&self) -> usize {
        std::fs::read_dir(&self.cache_dir)
            .map(|entries| entries.filter_map(Result::ok).count())
            .unwrap_or(0)
    }
}

/// Extract unique source files from graph nodes
fn extract_source_files(graph: &TypeGraph) -> Vec<SourceFileInfo> {
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    
    for node in graph.nodes().values() {
        let file_path = &node.location.file;
        
        if seen.insert(file_path.clone()) {
            if let Ok(hash) = get_file_mtime_hash(file_path) {
                files.push(SourceFileInfo {
                    path: file_path.clone(),
                    mtime_hash: hash,
                });
            }
        }
    }
    
    files
}

/// Get file modification time hash (simple u64 from SystemTime)
fn get_file_mtime_hash(path: &PathBuf) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let mtime = metadata.modified()?;
    
    // Convert SystemTime to u64 (seconds since UNIX_EPOCH)
    match mtime.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs()),
        Err(_) => Ok(0), // File modified before UNIX_EPOCH (unlikely)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::{
        TypeNode, SymbolKind, GraphCompleteness, SymbolOrigin,
        SourceLocation, QualifiedSymbolName, SymbolName
    };
    use crate::domain::language::Language;
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
        
        let test_symbol = SymbolName("test".to_string());
        let graph = TypeGraph {
            nodes: HashMap::from([
                (test_symbol.clone(), TypeNode {
                    id: QualifiedSymbolName {
                        module_path: PathBuf::from("test"),
                        symbol: test_symbol.clone(),
                    },
                    name: test_symbol.clone(),
                    kind: SymbolKind::Struct,
                    origin: SymbolOrigin::Canonical,
                    location: SourceLocation {
                        file: PathBuf::from("test.rs"),
                        line: 1,
                        column: 0,
                    },
                    language: Language::Rust,
                    generic_parameters: vec![],
                }),
            ]),
            edges: vec![],
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };
        
        cache.set("Test", 1, TraversalDirection::Both, &graph).unwrap();
        let cached = cache.get("Test", 1, TraversalDirection::Both).unwrap();
        
        assert_eq!(cached.nodes.len(), 1);
        assert!(cached.nodes.contains_key(&test_symbol));
    }
}
