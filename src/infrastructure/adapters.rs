use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::application::adapters::ApplicationAdapters;
use crate::domain::language::Language;
use crate::domain::ports::*;

/// Production adapters with real infrastructure wiring
pub struct WiredAdapters {
    workspace: Arc<WorkspaceAdapter>,
    filesystem: Arc<FileSystemAdapter>,
    daemon: Arc<DaemonAdapter>,
    lsp: Arc<LspAdapter>,
    clock: Arc<ClockAdapter>,
}

impl WiredAdapters {
    pub fn new(workspace_root: PathBuf, language: Language) -> Self {
        Self {
            workspace: Arc::new(WorkspaceAdapter { root: workspace_root.clone(), language }),
            filesystem: Arc::new(FileSystemAdapter),
            daemon: Arc::new(DaemonAdapter),
            lsp: Arc::new(LspAdapter::new(workspace_root, language)),  // Use ::new() which creates daemon
            clock: Arc::new(ClockAdapter),
        }
    }
}

impl ApplicationAdapters for WiredAdapters {
    type Workspace = WorkspaceAdapter;
    type FileSystem = FileSystemAdapter;
    type Daemon = DaemonAdapter;
    type Lsp = LspAdapter;
    type Clock = ClockAdapter;

    fn workspace(&self) -> &Self::Workspace { &self.workspace }
    fn file_system(&self) -> &Self::FileSystem { &self.filesystem }
    fn daemon(&self) -> &Self::Daemon { &self.daemon }
    fn lsp(&self) -> &Self::Lsp { &self.lsp }
    fn clock(&self) -> &Self::Clock { &self.clock }
}

// ============================================================================
// WorkspaceAdapter - Minimal implementation
// ============================================================================

pub struct WorkspaceAdapter {
    root: PathBuf,
    language: Language,
}

impl WorkspacePort for WorkspaceAdapter {
    type ProbeFuture<'a> = Pin<Box<dyn Future<Output = Result<WorkspaceSnapshot, WorkspacePortError>> + Send + 'a>>;

    fn probe_workspace<'a>(&'a self, probe: WorkspaceProbe) -> Self::ProbeFuture<'a> {
        let _root = self.root.clone();
        let language = self.language;
        
        Box::pin(async move {
            let path = crate::domain::workspace::WorkspacePath::new(probe.requested_path.clone())
                .map_err(|_e| WorkspacePortError::ProbeFailed {
                    path: probe.requested_path.clone(),
                    reason: "Invalid workspace path".to_string(),
                })?;
            
            let identity = crate::domain::workspace::WorkspaceIdentity::new("temp-id".to_string())
                .map_err(|_e| WorkspacePortError::IdentityUnavailable {
                    path: probe.requested_path,
                })?;

            Ok(WorkspaceSnapshot {
                path,
                identity,
                language,
                readable: true,
            })
        })
    }
}

// ============================================================================
// FileSystemAdapter - Minimal implementation
// ============================================================================

pub struct FileSystemAdapter;

impl FileSystemPort for FileSystemAdapter {
    type MetadataFuture<'a> = Pin<Box<dyn Future<Output = Result<FileMetadataSnapshot, FileSystemPortError>> + Send + 'a>>;

    fn read_file_metadata<'a>(&'a self, file: crate::domain::workspace::WorkspaceFile) -> Self::MetadataFuture<'a> {
        Box::pin(async move {
            let file_path = file.as_path();
            let metadata = std::fs::metadata(file_path)
                .map_err(|e| FileSystemPortError::MetadataReadFailed {
                    path: file_path.to_path_buf(),
                    reason: format!("Failed to read file metadata: {}", e),
                })?;

            let modified_at_unix_ms = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis())
                .unwrap_or(0);

            Ok(FileMetadataSnapshot {
                file,
                size_bytes: metadata.len(),
                modified_at_unix_ms,
            })
        })
    }
}

// ============================================================================
// DaemonAdapter - Stub (daemon mode not implemented yet)
// ============================================================================

pub struct DaemonAdapter;

impl DaemonPort for DaemonAdapter {
    type StartFuture<'a> = Pin<Box<dyn Future<Output = Result<DaemonRuntimeSnapshot, DaemonPortError>> + Send + 'a>>;
    type StopFuture<'a> = Pin<Box<dyn Future<Output = Result<(), DaemonPortError>> + Send + 'a>>;
    type StatusFuture<'a> = Pin<Box<dyn Future<Output = Result<DaemonRuntimeSnapshot, DaemonPortError>> + Send + 'a>>;

    fn start_daemon<'a>(&'a self, spec: DaemonStartSpec) -> Self::StartFuture<'a> {
        Box::pin(async move {
            Err(DaemonPortError::StartFailed {
                workspace: spec.workspace.as_path().to_path_buf(),
                reason: "Daemon mode not yet implemented".to_string(),
            })
        })
    }

    fn stop_daemon<'a>(&'a self, pid: Option<crate::domain::daemon::Pid>) -> Self::StopFuture<'a> {
        Box::pin(async move {
            Err(DaemonPortError::StopFailed {
                pid,
                reason: "Daemon mode not yet implemented".to_string(),
            })
        })
    }

    fn daemon_status<'a>(&'a self) -> Self::StatusFuture<'a> {
        Box::pin(async move {
            Err(DaemonPortError::StatusFailed {
                reason: "Daemon mode not yet implemented".to_string(),
            })
        })
    }
}

// ============================================================================
// LspAdapter - Real LSP integration
// ============================================================================

// ============================================================================
// LspAdapter - Uses PERSISTENT LSP daemon (not one-shot LazyGraphBuilder)
// ============================================================================

#[derive(Clone)]
pub struct LspAdapter {
    #[allow(dead_code)]
    workspace_root: PathBuf,
    #[allow(dead_code)]
    language: Language,
    daemon: Arc<crate::infrastructure::LspDaemon>,  // Persistent daemon
}

impl LspAdapter {
    pub fn new(workspace_root: PathBuf, language: Language) -> Self {
        Self {
            workspace_root: workspace_root.clone(),
            language,
            daemon: Arc::new(crate::infrastructure::LspDaemon::new(workspace_root, language)),
        }
    }
}

impl LspPort for LspAdapter {
    type QueryFuture<'a> = Pin<Box<dyn Future<Output = Result<LspQueryResponse, LspPortError>> + Send + 'a>>;
    type InvalidateFuture<'a> = Pin<Box<dyn Future<Output = Result<crate::domain::lsp::InvalidationResult, LspPortError>> + Send + 'a>>;

    fn run_query<'a>(&'a self, request: LspQueryRequest) -> Self::QueryFuture<'a> {
        let daemon = self.daemon.clone();

        Box::pin(async move {
            // Execute query based on type
            let graph = match &request.query {
                crate::domain::lsp::LspQuery::FromSymbol { symbol, depth } => {
                    let direction = crate::domain::graph::TraversalDirection::Both;
                    
                    // Use daemon's query_symbol which has cache integration
                    daemon.query_symbol(symbol, depth.get(), direction)
                        .await
                        .map_err(|e| LspPortError::QueryFailed {
                            source: crate::domain::lsp::LspError::WorkspaceError {
                                message: format!("Daemon query failed: {}", e),
                            },
                        })?
                },
                _ => {
                    return Err(LspPortError::QueryFailed {
                        source: crate::domain::lsp::LspError::WorkspaceError {
                            message: "Only FromSymbol query is currently supported".to_string(),
                        },
                    });
                }
            };

            Ok(LspQueryResponse {
                graph: Some(graph),
                capabilities: None,
            })
        })
    }

    fn invalidate<'a>(&'a self, request: crate::domain::lsp::InvalidationRequest) -> Self::InvalidateFuture<'a> {
        let daemon = self.daemon.clone();
        Box::pin(async move {
            // Convert WorkspaceFile to PathBuf
            let files: Vec<PathBuf> = request.files.iter()
                .map(|wf| wf.as_path().to_path_buf())
                .collect();
            
            // Invalidate cache for changed files
            let invalidated_count = daemon.invalidate_cache(&files)
                .await
                .map_err(|e| LspPortError::InvalidateFailed {
                    reason: format!("Cache invalidation failed: {}", e),
                })?;
            
            Ok(crate::domain::lsp::InvalidationResult {
                reindexed: false,  // No LSP reindex yet, just cache clear
                invalidated_files: invalidated_count,
            })
        })
    }
}

// ============================================================================
// ClockAdapter - Real system time
// ============================================================================

pub struct ClockAdapter;

impl ClockPort for ClockAdapter {
    type NowFuture<'a> = Pin<Box<dyn Future<Output = Result<ClockTick, ClockPortError>> + Send + 'a>>;
    type SleepFuture<'a> = Pin<Box<dyn Future<Output = Result<(), ClockPortError>> + Send + 'a>>;

    fn now<'a>(&'a self) -> Self::NowFuture<'a> {
        Box::pin(async move {
            let unix_time_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);

            Ok(ClockTick { unix_time_ms })
        })
    }

    fn sleep<'a>(&'a self, duration: Duration) -> Self::SleepFuture<'a> {
        Box::pin(async move {
            tokio::time::sleep(duration).await;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_wired_adapters_creation() {
        let _adapters = WiredAdapters::new(PathBuf::from("/tmp/test"), Language::TypeScript);
        // Just check it doesn't panic - test passes if no panic occurs
    }

    #[test]
    fn test_lsp_adapter_creation() {
        let adapter = LspAdapter::new(PathBuf::from("/tmp/test"), Language::Rust);
        // Verify it's clonable
        let _cloned = adapter.clone();
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_clock_adapter_now() {
        let clock = ClockAdapter;
        let result = clock.now().await;
        assert!(result.is_ok());
        let tick = result.unwrap();
        assert!(tick.unix_time_ms > 0);
    }

    #[tokio::test]
    async fn test_clock_adapter_sleep() {
        let clock = ClockAdapter;
        let start = std::time::Instant::now();
        let result = clock.sleep(Duration::from_millis(10)).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(10));
    }
}
