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

/// Position in a source file (1-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn new(line: u32, column: u32) -> Result<Self, PositionError> {
        if line == 0 {
            return Err(PositionError::InvalidLine { line });
        }
        if column == 0 {
            return Err(PositionError::InvalidColumn { column });
        }
        Ok(Self { line, column })
    }

    pub fn at_line(line: u32) -> Result<Self, PositionError> {
        Self::new(line, 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PositionError {
    #[error("Line must be greater than zero, got: {line}")]
    InvalidLine { line: u32 },
    #[error("Column must be greater than zero, got: {column}")]
    InvalidColumn { column: u32 },
}

/// Precise range in source code (start to end)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    pub file: PathBuf,
    pub start: Position,
    pub end: Position,
}

impl SourceRange {
    pub fn new(file: PathBuf, start: Position, end: Position) -> Result<Self, SourceRangeError> {
        if end < start {
            return Err(SourceRangeError::EndBeforeStart { start, end });
        }
        Ok(Self { file, start, end })
    }

    pub fn single_line(
        file: PathBuf,
        line: u32,
        start_col: u32,
        end_col: u32,
    ) -> Result<Self, SourceRangeError> {
        let start = Position::new(line, start_col)
            .map_err(|e| SourceRangeError::InvalidPosition { error: e })?;
        let end = Position::new(line, end_col)
            .map_err(|e| SourceRangeError::InvalidPosition { error: e })?;
        Self::new(file, start, end)
    }

    pub fn contains(&self, pos: &Position) -> bool {
        self.start <= *pos && *pos <= self.end
    }

    pub fn overlaps(&self, other: &SourceRange) -> bool {
        if self.file != other.file {
            return false;
        }
        !(self.end < other.start || other.end < self.start)
    }

    pub fn line_count(&self) -> u32 {
        self.end.line.saturating_sub(self.start.line) + 1
    }

    pub fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourceRangeError {
    #[error("Invalid position: {error}")]
    InvalidPosition { error: PositionError },
    #[error("End position {end:?} is before start position {start:?}")]
    EndBeforeStart { start: Position, end: Position },
}

/// Source range with the actual text content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeSpan {
    pub range: SourceRange,
    pub text: String,
    pub context_before: Option<Vec<String>>,
    pub context_after: Option<Vec<String>>,
}

impl CodeSpan {
    pub fn new(range: SourceRange, text: String) -> Self {
        Self {
            range,
            text,
            context_before: None,
            context_after: None,
        }
    }

    pub fn with_context(
        range: SourceRange,
        text: String,
        context_before: Vec<String>,
        context_after: Vec<String>,
    ) -> Self {
        Self {
            range,
            text,
            context_before: Some(context_before),
            context_after: Some(context_after),
        }
    }

    pub fn has_context(&self) -> bool {
        self.context_before.is_some() || self.context_after.is_some()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_symbol_name() -> SymbolName {
        SymbolName("TestSymbol".to_string())
    }
    
    fn test_qualified_name() -> QualifiedSymbolName {
        QualifiedSymbolName {
            module_path: PathBuf::from("/test/module.rs"),
            symbol: test_symbol_name(),
        }
    }
    
    fn test_source_location() -> SourceLocation {
        SourceLocation {
            file: PathBuf::from("/test/file.rs"),
            line: 10,
            column: 5,
        }
    }
    
    #[test]
    fn test_qualified_symbol_name() {
        let qname = QualifiedSymbolName {
            module_path: PathBuf::from("/src/lib.rs"),
            symbol: SymbolName("MyType".to_string()),
        };
        assert_eq!(qname.module_path, PathBuf::from("/src/lib.rs"));
        assert_eq!(qname.symbol.0, "MyType");
    }
    
    #[test]
    fn test_symbol_kind_variants() {
        assert_ne!(SymbolKind::Struct, SymbolKind::Enum);
        assert_eq!(SymbolKind::TypeAlias, SymbolKind::TypeAlias);
        assert_ne!(SymbolKind::Interface, SymbolKind::Union);
    }
    
    #[test]
    fn test_symbol_origin_variants() {
        let canonical = SymbolOrigin::Canonical;
        let external = SymbolOrigin::ExternalDependency;
        let generated = SymbolOrigin::GeneratedFile;
        
        assert_ne!(canonical, external);
        assert_ne!(external, generated);
    }
    
    #[test]
    fn test_type_parameter_name() {
        let param = TypeParameterName("T".to_string());
        assert_eq!(param.0, "T");
    }
    
    #[test]
    fn test_generic_instantiation() {
        let generic = GenericInstantiation {
            generic: test_qualified_name(),
            arguments: vec![test_qualified_name()],
        };
        assert_eq!(generic.arguments.len(), 1);
    }
    
    #[test]
    fn test_symbol_origin_canonical() {
        let origin = SymbolOrigin::Canonical;
        assert_eq!(origin, SymbolOrigin::Canonical);
    }
    
    #[test]
    fn test_symbol_origin_reexport() {
        let origin = SymbolOrigin::ReExport { via_module: PathBuf::from("/mod.rs") };
        match origin {
            SymbolOrigin::ReExport { via_module } => assert_eq!(via_module, PathBuf::from("/mod.rs")),
            _ => panic!("Expected ReExport"),
        }
    }
    
    #[test]
    fn test_position_new_valid() {
        let pos = Position::new(10, 5).unwrap();
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
    }
    
    #[test]
    fn test_position_invalid_line() {
        assert!(matches!(
            Position::new(0, 5),
            Err(PositionError::InvalidLine { line: 0 })
        ));
    }
    
    #[test]
    fn test_position_invalid_column() {
        assert!(matches!(
            Position::new(10, 0),
            Err(PositionError::InvalidColumn { column: 0 })
        ));
    }
    
    #[test]
    fn test_position_at_line() {
        let pos = Position::at_line(15).unwrap();
        assert_eq!(pos.line, 15);
        assert_eq!(pos.column, 1);
    }
    
    #[test]
    fn test_source_range_new_valid() {
        let start = Position::new(10, 5).unwrap();
        let end = Position::new(10, 15).unwrap();
        let range = SourceRange::new(PathBuf::from("/test.rs"), start, end).unwrap();
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }
    
    #[test]
    fn test_source_range_end_before_start() {
        let start = Position::new(10, 15).unwrap();
        let end = Position::new(10, 5).unwrap();
        assert!(matches!(
            SourceRange::new(PathBuf::from("/test.rs"), start, end),
            Err(SourceRangeError::EndBeforeStart { .. })
        ));
    }
    
    #[test]
    fn test_source_range_single_line() {
        let range = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 5, 15).unwrap();
        assert_eq!(range.start.line, 10);
        assert_eq!(range.start.column, 5);
        assert_eq!(range.end.line, 10);
        assert_eq!(range.end.column, 15);
        assert!(range.is_single_line());
    }
    
    #[test]
    fn test_source_range_contains() {
        let range = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 5, 15).unwrap();
        let pos_inside = Position::new(10, 10).unwrap();
        let pos_outside = Position::new(11, 10).unwrap();
        
        assert!(range.contains(&pos_inside));
        assert!(!range.contains(&pos_outside));
    }
    
    #[test]
    fn test_source_range_overlaps() {
        let range1 = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 5, 15).unwrap();
        let range2 = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 10, 20).unwrap();
        let range3 = SourceRange::single_line(PathBuf::from("/test.rs"), 11, 1, 10).unwrap();
        
        assert!(range1.overlaps(&range2));
        assert!(!range1.overlaps(&range3));
    }
    
    #[test]
    fn test_source_range_line_count() {
        let start = Position::new(10, 5).unwrap();
        let end = Position::new(15, 10).unwrap();
        let range = SourceRange::new(PathBuf::from("/test.rs"), start, end).unwrap();
        assert_eq!(range.line_count(), 6); // Lines 10-15 inclusive
    }
    
    #[test]
    fn test_code_span_new() {
        let range = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 5, 15).unwrap();
        let span = CodeSpan::new(range.clone(), "code text".to_string());
        assert_eq!(span.text, "code text");
        assert!(!span.has_context());
    }
    
    #[test]
    fn test_code_span_with_context() {
        let range = SourceRange::single_line(PathBuf::from("/test.rs"), 10, 5, 15).unwrap();
        let span = CodeSpan::with_context(
            range,
            "code text".to_string(),
            vec!["before".to_string()],
            vec!["after".to_string()],
        );
        assert!(span.has_context());
        assert_eq!(span.context_before, Some(vec!["before".to_string()]));
        assert_eq!(span.context_after, Some(vec!["after".to_string()]));
    }
    
    #[test]
    fn test_symbol_resolution_unique() {
        let res = SymbolResolution::Unique {
            qualified: test_qualified_name(),
            location: test_source_location(),
        };
        assert!(res.is_unique());
        assert!(!res.is_ambiguous());
        assert!(!res.is_not_found());
        assert_eq!(res.candidate_count(), 1);
        assert!(res.unique_qualified_name().is_some());
    }
    
    #[test]
    fn test_symbol_resolution_ambiguous() {
        let res = SymbolResolution::Ambiguous {
            candidates: vec![test_qualified_name(), test_qualified_name()],
            locations: vec![test_source_location(), test_source_location()],
        };
        assert!(!res.is_unique());
        assert!(res.is_ambiguous());
        assert!(!res.is_not_found());
        assert_eq!(res.candidate_count(), 2);
        assert!(res.unique_qualified_name().is_none());
    }
    
    #[test]
    fn test_symbol_resolution_not_found() {
        let res = SymbolResolution::NotFound {
            searched_symbol: test_symbol_name(),
        };
        assert!(!res.is_unique());
        assert!(!res.is_ambiguous());
        assert!(res.is_not_found());
        assert_eq!(res.candidate_count(), 0);
        assert!(res.unique_qualified_name().is_none());
    }
    
    #[test]
    fn test_cross_module_reference_internal() {
        let cross_ref = CrossModuleReference {
            source_module: PathBuf::from("/src/a.rs"),
            target_module: PathBuf::from("/src/b.rs"),
            symbol: test_symbol_name(),
            qualified_target: test_qualified_name(),
            reference_location: test_source_location(),
            definition_location: test_source_location(),
            import_path: None,
            crosses_package_boundary: false,
        };
        assert!(cross_ref.is_internal());
        assert!(!cross_ref.is_external());
        assert!(!cross_ref.has_explicit_import());
    }
    
    #[test]
    fn test_cross_module_reference_external() {
        let cross_ref = CrossModuleReference {
            source_module: PathBuf::from("/src/a.rs"),
            target_module: PathBuf::from("/external/lib.rs"),
            symbol: test_symbol_name(),
            qualified_target: test_qualified_name(),
            reference_location: test_source_location(),
            definition_location: test_source_location(),
            import_path: Some("external::lib".to_string()),
            crosses_package_boundary: true,
        };
        assert!(!cross_ref.is_internal());
        assert!(cross_ref.is_external());
        assert!(cross_ref.has_explicit_import());
    }
    
    #[test]
    fn test_cross_module_reference_graph_new() {
        let graph = CrossModuleReferenceGraph::new();
        assert_eq!(graph.reference_count(), 0);
        assert_eq!(graph.module_count(), 0);
    }
    
    #[test]
    fn test_cross_module_reference_graph_add() {
        let mut graph = CrossModuleReferenceGraph::new();
        let cross_ref = CrossModuleReference {
            source_module: PathBuf::from("/src/a.rs"),
            target_module: PathBuf::from("/src/b.rs"),
            symbol: test_symbol_name(),
            qualified_target: test_qualified_name(),
            reference_location: test_source_location(),
            definition_location: test_source_location(),
            import_path: None,
            crosses_package_boundary: false,
        };
        
        graph.add_reference(cross_ref);
        assert_eq!(graph.reference_count(), 1);
        assert_eq!(graph.module_count(), 2);
    }
    
    #[test]
    fn test_cross_module_reference_graph_references_from() {
        let mut graph = CrossModuleReferenceGraph::new();
        let cross_ref = CrossModuleReference {
            source_module: PathBuf::from("/src/a.rs"),
            target_module: PathBuf::from("/src/b.rs"),
            symbol: test_symbol_name(),
            qualified_target: test_qualified_name(),
            reference_location: test_source_location(),
            definition_location: test_source_location(),
            import_path: None,
            crosses_package_boundary: false,
        };
        
        graph.add_reference(cross_ref);
        let from_a = graph.references_from_module(&PathBuf::from("/src/a.rs"));
        assert_eq!(from_a.len(), 1);
        
        let from_b = graph.references_from_module(&PathBuf::from("/src/b.rs"));
        assert_eq!(from_b.len(), 0);
    }
    
    #[test]
    fn test_cross_module_reference_graph_external_refs() {
        let mut graph = CrossModuleReferenceGraph::new();
        let external = CrossModuleReference {
            source_module: PathBuf::from("/src/a.rs"),
            target_module: PathBuf::from("/external/lib.rs"),
            symbol: test_symbol_name(),
            qualified_target: test_qualified_name(),
            reference_location: test_source_location(),
            definition_location: test_source_location(),
            import_path: None,
            crosses_package_boundary: true,
        };
        
        graph.add_reference(external);
        assert_eq!(graph.external_references().len(), 1);
        assert_eq!(graph.internal_references().len(), 0);
    }
}
