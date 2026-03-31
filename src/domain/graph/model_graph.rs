use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::language::Language;

use super::symbols::{
    QualifiedSymbolName, SourceLocation, SymbolKind, SymbolName, SymbolOrigin, TypeParameterName,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeNode {
    pub id: QualifiedSymbolName,
    pub name: SymbolName,
    pub kind: SymbolKind,
    pub origin: SymbolOrigin,
    pub location: SourceLocation,
    pub language: Language,
    pub generic_parameters: Vec<TypeParameterName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    Contains,
    Extends,
    Variant,
    Transitions,
    Instantiates,
    ReExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeEdge {
    pub from: SymbolName,
    pub to: SymbolName,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedReferenceCount {
    pub value: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartialResultReason {
    DepthLimitReached,
    LspResponseIncomplete,
    FileNotReadable { path: PathBuf },
    PermissionDenied { path: PathBuf },
    LspIndexNotReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialResultSignal {
    pub reason: PartialResultReason,
    pub occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphCompleteness {
    Complete,
    Partial {
        unresolved_references: usize,
        signals: Vec<PartialResultSignal>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphWarning {
    CircularDependency {
        cycle: Vec<SymbolName>,
    },
    UnresolvedReference {
        symbol: SymbolName,
        from: SourceLocation,
    },
    PartialResult {
        reason: PartialResultReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeGraph {
    pub(crate) nodes: HashMap<SymbolName, TypeNode>,
    pub(crate) edges: Vec<TypeEdge>,
    pub(crate) warnings: Vec<GraphWarning>,
    pub(crate) completeness: GraphCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub warning_count: usize,
    pub unresolved_references: UnresolvedReferenceCount,
}

// ============================================================================
// Partial Graph (Phase 4)
// ============================================================================

/// Reason why a reference couldn't be resolved
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnresolveReason {
    LspTimeout,
    FileNotAccessible { path: PathBuf },
    SymbolNotFound,
    AmbiguousSymbol { candidates: Vec<SymbolName> },
}

/// An unresolved reference with context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedReference {
    pub symbol: SymbolName,
    pub referenced_from: SourceLocation,
    pub reason: UnresolveReason,
}

/// Explicitly partial graph (distinct from complete TypeGraph)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialGraph {
    pub nodes: HashMap<SymbolName, TypeNode>,
    pub edges: Vec<TypeEdge>,
    pub unresolved_references: Vec<UnresolvedReference>,
    pub partial_signals: Vec<PartialResultSignal>,
    pub completeness_percentage: u8,
}

impl PartialGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            unresolved_references: Vec::new(),
            partial_signals: Vec::new(),
            completeness_percentage: 0,
        }
    }

    pub fn with_completeness(mut self, percentage: u8) -> Self {
        self.completeness_percentage = percentage.min(100);
        self
    }

    pub fn to_type_graph_lossy(self) -> TypeGraph {
        TypeGraph {
            nodes: self.nodes,
            edges: self.edges,
            warnings: self
                .unresolved_references
                .into_iter()
                .map(|ur| GraphWarning::UnresolvedReference {
                    symbol: ur.symbol,
                    from: ur.referenced_from,
                })
                .collect(),
            completeness: GraphCompleteness::Partial {
                unresolved_references: 0,
                signals: self.partial_signals,
            },
        }
    }

    pub fn merge_with(&mut self, other: PartialGraph) {
        for (name, node) in other.nodes {
            self.nodes.entry(name).or_insert(node);
        }
        self.edges.extend(other.edges);
        self.unresolved_references
            .extend(other.unresolved_references);
        self.partial_signals.extend(other.partial_signals);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn unresolved_count(&self) -> usize {
        self.unresolved_references.len()
    }

    pub fn is_mostly_complete(&self) -> bool {
        self.completeness_percentage >= 90
    }
}

impl Default for PartialGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::domain::language::Language;

    #[test]
    fn test_edge_kind_variants() {
        assert_ne!(EdgeKind::Contains, EdgeKind::Extends);
        assert_eq!(EdgeKind::Variant, EdgeKind::Variant);
    }

    #[test]
    fn test_type_edge() {
        let edge = TypeEdge {
            from: SymbolName("A".to_string()),
            to: SymbolName("B".to_string()),
            kind: EdgeKind::Extends,
        };
        assert_eq!(edge.from.0, "A");
    }

    #[test]
    fn test_partial_graph_new() {
        let g = PartialGraph::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.completeness_percentage, 0);
    }

    #[test]
    fn test_partial_graph_with_completeness() {
        let g = PartialGraph::new().with_completeness(85);
        assert_eq!(g.completeness_percentage, 85);
        
        let clamped = PartialGraph::new().with_completeness(150);
        assert_eq!(clamped.completeness_percentage, 100);
    }

    #[test]
    fn test_partial_graph_is_mostly_complete() {
        assert!(PartialGraph::new().with_completeness(95).is_mostly_complete());
        assert!(!PartialGraph::new().with_completeness(80).is_mostly_complete());
    }

    #[test]
    fn test_graph_completeness_variants() {
        let complete = GraphCompleteness::Complete;
        assert!(matches!(complete, GraphCompleteness::Complete));
        
        let partial = GraphCompleteness::Partial {
            unresolved_references: 5,
            signals: vec![],
        };
        assert!(matches!(partial, GraphCompleteness::Partial { .. }));
    }

    #[test]
    fn test_partial_result_reason_variants() {
        let r1 = PartialResultReason::DepthLimitReached;
        let r2 = PartialResultReason::LspIndexNotReady;
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_unresolved_reference() {
        let u = UnresolvedReference {
            symbol: SymbolName("X".to_string()),
            referenced_from: SourceLocation {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 5,
            },
            reason: UnresolveReason::SymbolNotFound,
        };
        assert_eq!(u.symbol.0, "X");
    }
    
    #[test]
    fn test_unresolve_reason_variants() {
        assert_ne!(UnresolveReason::LspTimeout, UnresolveReason::SymbolNotFound);
        
        let file_err = UnresolveReason::FileNotAccessible { path: PathBuf::from("/test") };
        match file_err {
            UnresolveReason::FileNotAccessible { path } => assert_eq!(path, PathBuf::from("/test")),
            _ => panic!("Expected FileNotAccessible"),
        }
        
        let ambig = UnresolveReason::AmbiguousSymbol { candidates: vec![SymbolName("A".to_string())] };
        match ambig {
            UnresolveReason::AmbiguousSymbol { candidates } => assert_eq!(candidates.len(), 1),
            _ => panic!("Expected AmbiguousSymbol"),
        }
    }
    
    #[test]
    fn test_partial_graph_merge_with() {
        let mut g1 = PartialGraph::new();
        let mut g2 = PartialGraph::new();
        
        g1.nodes.insert(SymbolName("A".to_string()), TypeNode {
            id: QualifiedSymbolName {
                module_path: PathBuf::from("/test.rs"),
                symbol: SymbolName("A".to_string()),
            },
            name: SymbolName("A".to_string()),
            kind: SymbolKind::Struct,
            origin: SymbolOrigin::Canonical,
            location: SourceLocation {
                file: PathBuf::from("/test.rs"),
                line: 1,
                column: 1,
            },
            language: Language::Rust,
            generic_parameters: vec![],
        });
        
        g2.nodes.insert(SymbolName("B".to_string()), TypeNode {
            id: QualifiedSymbolName {
                module_path: PathBuf::from("/test.rs"),
                symbol: SymbolName("B".to_string()),
            },
            name: SymbolName("B".to_string()),
            kind: SymbolKind::Enum,
            origin: SymbolOrigin::Canonical,
            location: SourceLocation {
                file: PathBuf::from("/test.rs"),
                line: 10,
                column: 1,
            },
            language: Language::Rust,
            generic_parameters: vec![],
        });
        
        assert_eq!(g1.node_count(), 1);
        g1.merge_with(g2);
        assert_eq!(g1.node_count(), 2);
    }
    
    #[test]
    fn test_partial_graph_to_type_graph_lossy() {
        let mut pg = PartialGraph::new();
        pg.unresolved_references.push(UnresolvedReference {
            symbol: SymbolName("X".to_string()),
            referenced_from: SourceLocation {
                file: PathBuf::from("/test.rs"),
                line: 5,
                column: 10,
            },
            reason: UnresolveReason::SymbolNotFound,
        });
        pg.partial_signals.push(PartialResultSignal {
            reason: PartialResultReason::DepthLimitReached,
            occurrences: 1,
        });
        
        let tg = pg.to_type_graph_lossy();
        assert_eq!(tg.warnings().len(), 1);
        match &tg.warnings()[0] {
            GraphWarning::UnresolvedReference { symbol, .. } => assert_eq!(symbol.0, "X"),
            _ => panic!("Expected UnresolvedReference warning"),
        }
    }
    
    #[test]
    fn test_partial_graph_counts() {
        let mut pg = PartialGraph::new();
        assert_eq!(pg.node_count(), 0);
        assert_eq!(pg.edge_count(), 0);
        assert_eq!(pg.unresolved_count(), 0);
        
        pg.unresolved_references.push(UnresolvedReference {
            symbol: SymbolName("X".to_string()),
            referenced_from: SourceLocation {
                file: PathBuf::from("/test.rs"),
                line: 1,
                column: 1,
            },
            reason: UnresolveReason::SymbolNotFound,
        });
        
        assert_eq!(pg.unresolved_count(), 1);
    }
    
    #[test]
    fn test_partial_result_signal() {
        let sig = PartialResultSignal {
            reason: PartialResultReason::LspIndexNotReady,
            occurrences: 3,
        };
        assert_eq!(sig.occurrences, 3);
        match sig.reason {
            PartialResultReason::LspIndexNotReady => {},
            _ => panic!("Expected LspIndexNotReady"),
        }
    }
    
    #[test]
    fn test_graph_warning_variants() {
        let circular = GraphWarning::CircularDependency {
            cycle: vec![SymbolName("A".to_string()), SymbolName("B".to_string())],
        };
        match circular {
            GraphWarning::CircularDependency { cycle } => assert_eq!(cycle.len(), 2),
            _ => panic!("Expected CircularDependency"),
        }
        
        let unresolved = GraphWarning::UnresolvedReference {
            symbol: SymbolName("X".to_string()),
            from: SourceLocation {
                file: PathBuf::from("/test.rs"),
                line: 1,
                column: 1,
            },
        };
        match unresolved {
            GraphWarning::UnresolvedReference { symbol, .. } => assert_eq!(symbol.0, "X"),
            _ => panic!("Expected UnresolvedReference"),
        }
    }
}
