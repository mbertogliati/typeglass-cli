use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::symbols::{QualifiedSymbolName, SourceLocation, SymbolName};

/// Query for finding all references to a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceQuery {
    /// Find references by exact symbol name
    BySymbol { symbol: SymbolName },
    /// Find references by qualified name
    ByQualifiedName { qualified: QualifiedSymbolName },
    /// Find references in a specific file
    InFile {
        symbol: SymbolName,
        file: PathBuf,
    },
    /// Find references in a module and its descendants
    InModule {
        symbol: SymbolName,
        module_path: PathBuf,
        recursive: bool,
    },
}

/// Kind of reference usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Symbol is being read/referenced
    Read,
    /// Symbol is being written/modified (if applicable)
    Write,
    /// Symbol is being called as a function
    Call,
    /// Symbol is being imported
    Import,
    /// Symbol is being exported
    Export,
    /// Symbol is being extended/implemented
    Extends,
    /// Symbol is part of a type annotation
    TypeAnnotation,
}

/// A single reference to a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolReference {
    /// The symbol being referenced
    pub symbol: SymbolName,
    /// Location of the reference
    pub location: SourceLocation,
    /// Kind of usage
    pub kind: ReferenceKind,
    /// Surrounding context (e.g., containing function name)
    pub context: Option<String>,
}

/// Result of a reference query
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceResult {
    /// The query that was executed
    pub query: ReferenceQuery,
    /// All found references
    pub references: Vec<SymbolReference>,
    /// Whether the result is complete or partial
    pub is_complete: bool,
    /// Number of files scanned
    pub files_scanned: usize,
}

impl ReferenceResult {
    pub fn reference_count(&self) -> usize {
        self.references.len()
    }

    pub fn references_by_kind(&self, kind: ReferenceKind) -> Vec<&SymbolReference> {
        self.references
            .iter()
            .filter(|r| r.kind == kind)
            .collect()
    }

    pub fn references_in_file(&self, file: &PathBuf) -> Vec<&SymbolReference> {
        self.references
            .iter()
            .filter(|r| r.location.file == *file)
            .collect()
    }

    pub fn unique_files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self
            .references
            .iter()
            .map(|r| r.location.file.clone())
            .collect();
        files.sort();
        files.dedup();
        files
    }
}

/// Batch query for multiple reference lookups
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReferenceQuery {
    /// Multiple queries to execute in parallel
    pub queries: Vec<ReferenceQuery>,
    /// Maximum number of concurrent queries
    pub max_concurrency: Option<usize>,
}

/// Result of a batch reference query
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReferenceResult {
    /// Results for each query (same order as input)
    pub results: Vec<ReferenceResult>,
    /// Total execution time in milliseconds
    pub total_time_ms: Option<u64>,
}
