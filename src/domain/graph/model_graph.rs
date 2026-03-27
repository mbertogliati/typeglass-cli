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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
