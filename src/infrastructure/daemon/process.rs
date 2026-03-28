//! LSP Daemon Process Management
//!
//! Manages a persistent LSP server process that survives CLI invocations.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::language::Language;
use crate::infrastructure::LazyGraphBuilder;

/// Persistent LSP daemon that reuses connections across CLI invocations
pub struct LspDaemon {
    workspace_root: PathBuf,
    language: Language,
    /// LSP client that stays alive between queries
    /// Wrapped in Mutex for interior mutability
    lsp_client: Arc<Mutex<Option<LazyGraphBuilder>>>,
}

impl LspDaemon {
    /// Create a new daemon (doesn't start LSP yet - lazy init)
    pub fn new(workspace_root: PathBuf, language: Language) -> Self {
        Self {
            workspace_root,
            language,
            lsp_client: Arc::new(Mutex::new(None)),
        }
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
}
