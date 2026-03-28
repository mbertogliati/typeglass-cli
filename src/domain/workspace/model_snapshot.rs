use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::source::ContentHash;

/// Metadata for a single file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub hash: Option<ContentHash>,
}

/// Hash of the entire file tree structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotHash(String);

impl SnapshotHash {
    pub fn compute(files: &HashMap<PathBuf, FileMetadata>) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Sort for deterministic hashing
        let mut sorted: Vec<_> = files.iter().collect();
        sorted.sort_by_key(|(path, _)| path.as_os_str());
        
        for (path, metadata) in sorted {
            path.hash(&mut hasher);
            metadata.size.hash(&mut hasher);
            if let Some(hash) = &metadata.hash {
                hash.as_str().hash(&mut hasher);
            }
        }
        
        Self(format!("{:x}", hasher.finish()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Snapshot of the file tree at a point in time
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTreeSnapshot {
    root: PathBuf,
    files: HashMap<PathBuf, FileMetadata>,
    directories: Vec<PathBuf>,
    snapshot_time: SystemTime,
    hash: SnapshotHash,
}

impl FileTreeSnapshot {
    pub fn new(
        root: PathBuf,
        files: HashMap<PathBuf, FileMetadata>,
        directories: Vec<PathBuf>,
    ) -> Self {
        let hash = SnapshotHash::compute(&files);
        Self {
            root,
            files,
            directories,
            snapshot_time: SystemTime::now(),
            hash,
        }
    }

    pub fn empty(root: PathBuf) -> Self {
        Self::new(root, HashMap::new(), Vec::new())
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn directory_count(&self) -> usize {
        self.directories.len()
    }

    pub fn snapshot_time(&self) -> SystemTime {
        self.snapshot_time
    }

    pub fn hash(&self) -> &SnapshotHash {
        &self.hash
    }

    pub fn has_changed(&self, other: &FileTreeSnapshot) -> bool {
        self.hash != other.hash
    }

    pub fn diff(&self, other: &FileTreeSnapshot) -> FileTreeDiff {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();

        // Find added and modified files
        for (path, metadata) in &other.files {
            match self.files.get(path) {
                None => added.push(path.clone()),
                Some(old_metadata) => {
                    if metadata != old_metadata {
                        modified.push(path.clone());
                    }
                }
            }
        }

        // Find deleted files
        for path in self.files.keys() {
            if !other.files.contains_key(path) {
                deleted.push(path.clone());
            }
        }

        FileTreeDiff {
            added,
            modified,
            deleted,
        }
    }

    pub fn file_metadata(&self, path: &PathBuf) -> Option<&FileMetadata> {
        self.files.get(path)
    }

    pub fn contains_file(&self, path: &PathBuf) -> bool {
        self.files.contains_key(path)
    }

    pub fn all_files(&self) -> Vec<&PathBuf> {
        self.files.keys().collect()
    }
}

/// Difference between two file tree snapshots
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTreeDiff {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

impl FileTreeDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }

    pub fn has_additions(&self) -> bool {
        !self.added.is_empty()
    }

    pub fn has_modifications(&self) -> bool {
        !self.modified.is_empty()
    }

    pub fn has_deletions(&self) -> bool {
        !self.deleted.is_empty()
    }
}

#[derive(Debug, Error)]
pub enum FileTreeError {
    #[error("Failed to read directory: {path}. Reason: {reason}")]
    DirectoryReadFailed { path: PathBuf, reason: String },
    #[error("Path is not a directory: {path}")]
    NotADirectory { path: PathBuf },
}
