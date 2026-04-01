use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    TypeScript,
    Rust,
    Go,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageProfile {
    Single(Language),
    Multiple(Vec<Language>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageExtension {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageRule {
    pub language: Language,
    pub extensions: Vec<LanguageExtension>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageRuleset {
    pub rules: Vec<LanguageRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanScope {
    FirstLevel,
    Recursive { max_depth: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanFileLimit {
    pub value: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageScanPolicy {
    pub scope: ScanScope,
    pub file_limit: ScanFileLimit,
    pub include_hidden_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageSupportStatus {
    Supported(Language),
    Unsupported {
        detected: Language,
        supported: Vec<Language>,
    },
    MultipleDetected {
        detected: Vec<Language>,
    },
    NotDetected,
}

#[derive(Debug, Error)]
pub enum LanguageDetectionError {
    #[error("No source files found under: {searched_path}")]
    NoSourceFilesFound { searched_path: PathBuf },
    #[error("Multiple languages detected in workspace: {found:?}")]
    MultipleLanguagesDetected { found: Vec<Language> },
    #[error("Unsupported language detected: {detected:?}. Supported: {supported:?}")]
    UnsupportedLanguage {
        detected: Language,
        supported: Vec<Language>,
    },
    #[error("Directory is not readable: {path}. Reason: {reason}")]
    DirectoryNotReadable { path: PathBuf, reason: io::Error },
    #[error("Language scan exceeded configured file limit: {limit}")]
    TooManyFilesToScan { limit: usize },
}

#[derive(Debug, Error)]
pub enum LanguageRulesetError {
    #[error("Language ruleset cannot be empty")]
    Empty,
    #[error("At least one language rule has no file extensions")]
    RuleWithoutExtensions,
}

#[derive(Debug, Error)]
pub enum ScanFileLimitError {
    #[error("Scan file limit must be greater than zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum LanguageExtensionError {
    #[error("Language extension cannot be empty")]
    Empty,
    #[error("Language extension must start with a dot: {value}")]
    MissingDotPrefix { value: String },
}
