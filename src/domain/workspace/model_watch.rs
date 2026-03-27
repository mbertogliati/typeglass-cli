use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::source::ContentHash;

/// File system change event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeEvent {
    /// File was created
    Created {
        path: PathBuf,
        timestamp: SystemTime,
    },
    /// File was modified
    Modified {
        path: PathBuf,
        timestamp: SystemTime,
        previous_hash: Option<ContentHash>,
    },
    /// File was deleted
    Deleted {
        path: PathBuf,
        timestamp: SystemTime,
    },
    /// File was renamed
    Renamed {
        from: PathBuf,
        to: PathBuf,
        timestamp: SystemTime,
    },
}

impl FileChangeEvent {
    pub fn path(&self) -> &PathBuf {
        match self {
            FileChangeEvent::Created { path, .. } => path,
            FileChangeEvent::Modified { path, .. } => path,
            FileChangeEvent::Deleted { path, .. } => path,
            FileChangeEvent::Renamed { to, .. } => to,
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        match self {
            FileChangeEvent::Created { timestamp, .. } => *timestamp,
            FileChangeEvent::Modified { timestamp, .. } => *timestamp,
            FileChangeEvent::Deleted { timestamp, .. } => *timestamp,
            FileChangeEvent::Renamed { timestamp, .. } => *timestamp,
        }
    }

    pub fn kind(&self) -> FileChangeKind {
        match self {
            FileChangeEvent::Created { .. } => FileChangeKind::Created,
            FileChangeEvent::Modified { .. } => FileChangeKind::Modified,
            FileChangeEvent::Deleted { .. } => FileChangeKind::Deleted,
            FileChangeEvent::Renamed { .. } => FileChangeKind::Renamed,
        }
    }

    pub fn is_content_change(&self) -> bool {
        matches!(
            self,
            FileChangeEvent::Created { .. }
                | FileChangeEvent::Modified { .. }
                | FileChangeEvent::Deleted { .. }
        )
    }
}

/// Kind of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Policy for file watching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileWatchPolicy {
    /// Debounce time in milliseconds
    pub debounce_ms: u32,
    /// Patterns to ignore (glob syntax)
    pub ignore_patterns: Vec<String>,
    /// Watch subdirectories recursively
    pub recursive: bool,
}

impl FileWatchPolicy {
    pub fn default_policy() -> Self {
        Self {
            debounce_ms: 100,
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
                "*.tmp".to_string(),
                "*.swp".to_string(),
            ],
            recursive: true,
        }
    }

    pub fn should_ignore(&self, path: &PathBuf) -> bool {
        // Simple pattern matching (would use glob crate in real impl)
        let path_str = path.to_string_lossy();
        self.ignore_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern.trim_end_matches("/**")))
    }
}

impl Default for FileWatchPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

/// Batch of file change events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChangeBatch {
    pub events: Vec<FileChangeEvent>,
    pub batch_time: SystemTime,
}

impl FileChangeBatch {
    pub fn new(events: Vec<FileChangeEvent>) -> Self {
        Self {
            events,
            batch_time: SystemTime::now(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn affected_files(&self) -> Vec<&PathBuf> {
        self.events.iter().map(|e| e.path()).collect()
    }

    pub fn has_deletions(&self) -> bool {
        self.events
            .iter()
            .any(|e| matches!(e, FileChangeEvent::Deleted { .. }))
    }
}
