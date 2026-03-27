use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolName(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QualifiedSymbolName {
    pub module_path: PathBuf,
    pub symbol: SymbolName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeParameterName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericInstantiation {
    pub generic: QualifiedSymbolName,
    pub arguments: Vec<QualifiedSymbolName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolOrigin {
    Canonical,
    ReExport { via_module: PathBuf },
    GeneratedFile,
    ExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    TypeAlias,
    Interface,
    Enum,
    Struct,
    Union,
    Primitive,
    External,
}

/// Resolution result when looking up a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolResolution {
    /// Symbol was uniquely resolved to a single definition
    Unique {
        qualified: QualifiedSymbolName,
        location: SourceLocation,
    },
    /// Multiple definitions found (requires disambiguation)
    Ambiguous {
        candidates: Vec<QualifiedSymbolName>,
        locations: Vec<SourceLocation>,
    },
    /// Symbol was not found in any known location
    NotFound { searched_symbol: SymbolName },
}

impl SymbolResolution {
    pub fn is_unique(&self) -> bool {
        matches!(self, SymbolResolution::Unique { .. })
    }

    pub fn is_ambiguous(&self) -> bool {
        matches!(self, SymbolResolution::Ambiguous { .. })
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, SymbolResolution::NotFound { .. })
    }

    pub fn unique_qualified_name(&self) -> Option<&QualifiedSymbolName> {
        match self {
            SymbolResolution::Unique { qualified, .. } => Some(qualified),
            _ => None,
        }
    }

    pub fn candidate_count(&self) -> usize {
        match self {
            SymbolResolution::Unique { .. } => 1,
            SymbolResolution::Ambiguous { candidates, .. } => candidates.len(),
            SymbolResolution::NotFound { .. } => 0,
        }
    }
}

/// Strategy for resolving ambiguous symbol references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Prefer the closest definition in the module hierarchy
    PreferClosest,
    /// Prefer canonical definitions over re-exports
    PreferCanonical,
    /// Use explicit import paths to disambiguate
    UseImportPaths,
    /// Return all candidates and let caller decide
    ReturnAllCandidates,
}

/// Reference that crosses module boundaries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossModuleReference {
    /// Source module containing the reference
    pub source_module: PathBuf,
    /// Target module containing the definition
    pub target_module: PathBuf,
    /// The symbol being referenced
    pub symbol: SymbolName,
    /// Qualified name of the referenced symbol
    pub qualified_target: QualifiedSymbolName,
    /// Location of the reference in source
    pub reference_location: SourceLocation,
    /// Location of the definition in target
    pub definition_location: SourceLocation,
    /// Import path used (if explicit)
    pub import_path: Option<String>,
    /// Whether this crosses a package/crate boundary
    pub crosses_package_boundary: bool,
}

impl CrossModuleReference {
    pub fn is_internal(&self) -> bool {
        !self.crosses_package_boundary
    }

    pub fn is_external(&self) -> bool {
        self.crosses_package_boundary
    }

    pub fn has_explicit_import(&self) -> bool {
        self.import_path.is_some()
    }
}

/// Collection of cross-module references in a graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossModuleReferenceGraph {
    /// All cross-module references
    pub references: Vec<CrossModuleReference>,
    /// Unique modules involved
    pub modules: Vec<PathBuf>,
}

impl CrossModuleReferenceGraph {
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            modules: Vec::new(),
        }
    }

    pub fn add_reference(&mut self, reference: CrossModuleReference) {
        if !self.modules.contains(&reference.source_module) {
            self.modules.push(reference.source_module.clone());
        }
        if !self.modules.contains(&reference.target_module) {
            self.modules.push(reference.target_module.clone());
        }
        self.references.push(reference);
    }

    pub fn reference_count(&self) -> usize {
        self.references.len()
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn references_from_module(&self, module: &PathBuf) -> Vec<&CrossModuleReference> {
        self.references
            .iter()
            .filter(|r| r.source_module == *module)
            .collect()
    }

    pub fn references_to_module(&self, module: &PathBuf) -> Vec<&CrossModuleReference> {
        self.references
            .iter()
            .filter(|r| r.target_module == *module)
            .collect()
    }

    pub fn external_references(&self) -> Vec<&CrossModuleReference> {
        self.references.iter().filter(|r| r.is_external()).collect()
    }

    pub fn internal_references(&self) -> Vec<&CrossModuleReference> {
        self.references.iter().filter(|r| r.is_internal()).collect()
    }
}

impl Default for CrossModuleReferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}
