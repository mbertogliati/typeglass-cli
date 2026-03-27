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
