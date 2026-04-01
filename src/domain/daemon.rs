//! Minimal daemon types for port DTOs
//!
//! This module contains only the types needed by port DTOs.
//! Complex daemon lifecycle management has been removed (590 lines deleted).
//! Current approach: In-process LSP daemon per CLI invocation (sufficient for MVP).

use std::path::PathBuf;

/// Process ID (used in error types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub u32);

/// Cache directory path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheDir(pub PathBuf);

impl CacheDir {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
    
    pub fn as_path(&self) -> &PathBuf {
        &self.0
    }
}

/// Daemon endpoint (for future IPC if needed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonEndpoint {
    UnixSocket(PathBuf),
    WindowsNamedPipe(String),
}

/// Daemon runtime state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonState {
    NotRunning,
    Running {
        pid: Pid,
    },
}

/// Daemon execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonMode {
    Persistent,
    OneShot,
}
