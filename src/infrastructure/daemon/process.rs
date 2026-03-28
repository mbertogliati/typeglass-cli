//! LSP Daemon Process Management
//!
//! Manages a persistent LSP server process that survives CLI invocations.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::language::Language;
use crate::domain::graph::{TypeGraph, TraversalDirection};
use crate::infrastructure::{LazyGraphBuilder, GraphCache};

/// Persistent LSP daemon that reuses connections across CLI invocations
pub struct LspDaemon {
    workspace_root: PathBuf,
    language: Language,
    /// LSP client that stays alive between queries
    /// Wrapped in Mutex for interior mutability
    lsp_client: Arc<Mutex<Option<LazyGraphBuilder>>>,
    /// Shared cache for graph results
    cache: Arc<Mutex<GraphCache>>,
}

impl LspDaemon {
    /// Create a new daemon (doesn't start LSP yet - lazy init)
    pub fn new(workspace_root: PathBuf, language: Language) -> Self {
        let cache = GraphCache::new().unwrap_or_else(|e| {
            log::warn!("Failed to initialize cache: {}, caching disabled", e);
            // Return a dummy cache that will fail operations gracefully
            GraphCache::new().unwrap()
        });
        
        Self {
            workspace_root,
            language,
            lsp_client: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    /// Query with cache support
    pub async fn query_symbol(
        &self,
        symbol: &str,
        depth: u8,
        direction: TraversalDirection,
    ) -> Result<TypeGraph, DaemonError> {
        // 1. Check cache first
        {
            let cache = self.cache.lock().await;
            if let Ok(graph) = cache.get(symbol, depth as usize, direction) {
                log::debug!("Cache HIT for symbol '{}' (depth {})", symbol, depth);
                return Ok(graph);
            }
            log::debug!("Cache MISS for symbol '{}' (depth {})", symbol, depth);
        }

        // 2. Cache miss - query LSP
        let lsp_client_arc = self.get_lsp_client().await?;
        let mut guard = lsp_client_arc.lock().await;
        let builder = guard.as_mut()
            .ok_or(DaemonError::NotRunning)?;

        let graph = builder.build_from_symbol(symbol, direction, depth)
            .await
            .map_err(|e| DaemonError::QueryFailed(e.to_string()))?;

        // 3. Write to cache
        {
            let cache = self.cache.lock().await;
            if let Err(e) = cache.set(symbol, depth as usize, direction, &graph) {
                log::warn!("Failed to write cache: {}", e);
            } else {
                log::debug!("Cache WRITE for symbol '{}' (depth {})", symbol, depth);
            }
        }

        Ok(graph)
    }

    /// Get or create the LSP client (lazy initialization)
    pub async fn get_lsp_client(&self) -> Result<Arc<Mutex<Option<LazyGraphBuilder>>>, DaemonError> {
        let mut guard = self.lsp_client.lock().await;
        
        if guard.is_none() {
            log::info!("Starting LSP daemon for {:?} in {}", self.language, self.workspace_root.display());
            
            let builder = LazyGraphBuilder::new(
                self.workspace_root.clone(),
                self.language,
            )
            .await
            .map_err(|e| DaemonError::LspStartFailed(e.to_string()))?;
            
            *guard = Some(builder);
        }
        
        drop(guard); // Release lock
        Ok(self.lsp_client.clone())
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        let guard = self.lsp_client.lock().await;
        guard.is_some()
    }

    /// Stop the daemon (shuts down LSP)
    pub async fn stop(&self) -> Result<(), DaemonError> {
        let mut guard = self.lsp_client.lock().await;
        
        if let Some(builder) = guard.take() {
            log::info!("Stopping LSP daemon");
            builder.shutdown().await
                .map_err(|e| DaemonError::ShutdownFailed(e.to_string()))?;
        }
        
        Ok(())
    }
}

impl Clone for LspDaemon {
    fn clone(&self) -> Self {
        Self {
            workspace_root: self.workspace_root.clone(),
            language: self.language,
            lsp_client: self.lsp_client.clone(), // Arc clone (cheap)
            cache: self.cache.clone(), // Arc clone (cheap)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("Failed to start LSP: {0}")]
    LspStartFailed(String),
    
    #[error("Failed to shutdown LSP: {0}")]
    ShutdownFailed(String),
    
    #[error("LSP client is not running")]
    NotRunning,
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_daemon_creation() {
        let daemon = LspDaemon::new(PathBuf::from("/tmp/test"), Language::TypeScript);
        assert_eq!(daemon.workspace_root, PathBuf::from("/tmp/test"));
        assert_eq!(daemon.language, Language::TypeScript);
    }

    #[test]
    fn test_daemon_clone() {
        let daemon = LspDaemon::new(PathBuf::from("/tmp/test"), Language::Rust);
        let cloned = daemon.clone();
        assert_eq!(cloned.workspace_root, daemon.workspace_root);
        assert_eq!(cloned.language, daemon.language);
    }

    #[tokio::test]
    async fn test_daemon_not_running_initially() {
        let daemon = LspDaemon::new(PathBuf::from("/tmp/test"), Language::Go);
        assert!(!daemon.is_running().await);
    }

    #[tokio::test]
    async fn test_daemon_stop_when_not_running() {
        let daemon = LspDaemon::new(PathBuf::from("/tmp/test"), Language::TypeScript);
        let result = daemon.stop().await;
        assert!(result.is_ok()); // Should succeed even if not running
    }
}
