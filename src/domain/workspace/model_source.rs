use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::path::WorkspacePath;

/// A validated source file path (exists, readable, is a file)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceFile(PathBuf);

impl SourceFile {
    pub fn new(path: PathBuf) -> Result<Self, SourceFileError> {
        // Verify exists
        if !path.exists() {
            return Err(SourceFileError::NotFound { path });
        }

        // Verify is a file
        let metadata = fs::metadata(&path)
            .map_err(|e| SourceFileError::NotReadable { path: path.clone(), reason: e })?;
        
        if !metadata.is_file() {
            return Err(SourceFileError::NotAFile { path });
        }

        // Verify readable by attempting to read metadata (permissions check)
        fs::File::open(&path)
            .map_err(|e| SourceFileError::NotReadable { path: path.clone(), reason: e })?;

        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    pub fn read_content(&self) -> Result<FileContent, SourceFileError> {
        let bytes = fs::read(&self.0)
            .map_err(|e| SourceFileError::NotReadable { 
                path: self.0.clone(), 
                reason: e 
            })?;
        
        FileContent::from_bytes(bytes, self.0.clone())
            .map_err(|e| match e {
                FileContentError::InvalidUtf8 { path } => SourceFileError::InvalidUtf8 { path },
                FileContentError::TooLarge { size_bytes, limit } => {
                    SourceFileError::TooLarge { size_bytes, limit }
                },
                _ => SourceFileError::NotReadable { 
                    path: self.0.clone(), 
                    reason: io::Error::other("Content error") 
                },
            })
    }

    pub fn metadata(&self) -> Result<FileMetadataSnapshot, SourceFileError> {
        let metadata = fs::metadata(&self.0)
            .map_err(|e| SourceFileError::NotReadable { path: self.0.clone(), reason: e })?;

        Ok(FileMetadataSnapshot {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_readonly: metadata.permissions().readonly(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadataSnapshot {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_readonly: bool,
}

#[derive(Debug, Error)]
pub enum SourceFileError {
    #[error("File not found: {path}")]
    NotFound { path: PathBuf },
    #[error("Path is not a file: {path}")]
    NotAFile { path: PathBuf },
    #[error("File is not readable: {path}. Reason: {reason}")]
    NotReadable { path: PathBuf, reason: io::Error },
    #[error("File contains invalid UTF-8: {path}")]
    InvalidUtf8 { path: PathBuf },
    #[error("File is too large: {size_bytes} bytes (limit: {limit} bytes)")]
    TooLarge { size_bytes: usize, limit: usize },
}

/// Logical module path (e.g., "@/components/Button" vs "src/components/Button.tsx")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulePath(String);

impl ModulePath {
    pub fn new(path: String) -> Result<Self, ModulePathError> {
        if path.is_empty() {
            return Err(ModulePathError::Empty);
        }

        // Check for invalid characters (basic validation)
        let invalid_chars: Vec<char> = path
            .chars()
            .filter(|c| c.is_control() || *c == '\0')
            .collect();

        if !invalid_chars.is_empty() {
            return Err(ModulePathError::InvalidCharacters { 
                path, 
                invalid: invalid_chars 
            });
        }

        Ok(Self(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_source_file(file: &SourceFile, base: &WorkspacePath) -> Option<Self> {
        file.as_path()
            .strip_prefix(base.as_path())
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| Self(s.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum ModulePathError {
    #[error("Module path cannot be empty")]
    Empty,
    #[error("Module path contains invalid characters: {path}. Invalid: {invalid:?}")]
    InvalidCharacters { path: String, invalid: Vec<char> },
}

/// File content with encoding detection and change tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileContent {
    content: String,
    encoding: Encoding,
    line_ending: LineEnding,
    hash: ContentHash,
    #[serde(skip)]
    source_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encoding {
    Utf8,
    Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    Unix,    // \n
    Windows, // \r\n
    Mixed,   // inconsistent
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(String);

impl FileContent {
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB

    pub fn from_bytes(bytes: Vec<u8>, source_path: PathBuf) -> Result<Self, FileContentError> {
        if bytes.len() > Self::MAX_FILE_SIZE {
            return Err(FileContentError::TooLarge {
                size_bytes: bytes.len(),
                limit: Self::MAX_FILE_SIZE,
            });
        }

        // Detect encoding (simple: try UTF-8 first)
        let content = String::from_utf8(bytes)
            .map_err(|_| FileContentError::InvalidUtf8 { path: source_path.clone() })?;

        let encoding = if content.is_ascii() {
            Encoding::Ascii
        } else {
            Encoding::Utf8
        };

        // Detect line endings
        let line_ending = Self::detect_line_ending(&content);

        // Compute hash
        let hash = ContentHash::compute(&content);

        Ok(Self {
            content,
            encoding,
            line_ending,
            hash,
            source_path,
        })
    }

    fn detect_line_ending(content: &str) -> LineEnding {
        let has_crlf = content.contains("\r\n");
        let has_lf = content.contains('\n');

        match (has_crlf, has_lf) {
            (true, false) => LineEnding::Windows,
            (false, true) => LineEnding::Unix,
            (true, true) => LineEnding::Mixed,
            (false, false) => LineEnding::Unix, // default
        }
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    pub fn line_at(&self, line: u32) -> Option<&str> {
        self.lines().get((line.saturating_sub(1)) as usize).copied()
    }

    pub fn hash(&self) -> &ContentHash {
        &self.hash
    }

    pub fn has_changed(&self, previous_hash: &ContentHash) -> bool {
        self.hash != *previous_hash
    }

    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

impl ContentHash {
    pub fn compute(content: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self(format!("{:x}", hasher.finish()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum FileContentError {
    #[error("File contains invalid UTF-8: {path}")]
    InvalidUtf8 { path: PathBuf },
    #[error("Unsupported encoding detected: {detected}")]
    UnsupportedEncoding { detected: String },
    #[error("File is too large: {size_bytes} bytes (limit: {limit} bytes)")]
    TooLarge { size_bytes: usize, limit: usize },
}
