use std::collections::HashMap;

use crate::domain::graph::{
    GeneratedFilePattern, GeneratedFilePatternError, GraphCompleteness, GraphIntegrityError,
    GraphStatistics, GraphWarning, SourceLocation, SourceLocationError, SymbolKind, SymbolName,
    SymbolNameError, TraversalPolicy, TraversalPolicyError, TypeEdge, TypeGraph, TypeNode,
    UnresolvedReferenceCount,
};

impl SymbolName {
    pub fn new(value: String) -> Result<Self, SymbolNameError> {
        if value.trim().is_empty() {
            return Err(SymbolNameError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl SourceLocation {
    pub fn new(
        file: std::path::PathBuf,
        line: u32,
        column: u32,
    ) -> Result<Self, SourceLocationError> {
        if line == 0 {
            return Err(SourceLocationError::InvalidLine);
        }
        if column == 0 {
            return Err(SourceLocationError::InvalidColumn);
        }
        Ok(Self { file, line, column })
    }
}

impl TypeGraph {
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            warnings: Vec::new(),
            completeness: GraphCompleteness::Complete,
        }
    }

    pub fn nodes(&self) -> &HashMap<SymbolName, TypeNode> {
        &self.nodes
    }

    pub fn edges(&self) -> &[TypeEdge] {
        &self.edges
    }

    pub fn warnings(&self) -> &[GraphWarning] {
        &self.warnings
    }

    pub fn completeness(&self) -> &GraphCompleteness {
        &self.completeness
    }

    pub fn statistics(&self) -> GraphStatistics {
        let unresolved_references = self
            .warnings
            .iter()
            .filter(|warning| matches!(warning, GraphWarning::UnresolvedReference { .. }))
            .count();

        GraphStatistics {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            warning_count: self.warnings.len(),
            unresolved_references: UnresolvedReferenceCount {
                value: unresolved_references,
            },
        }
    }

    pub fn validate_integrity(&self) -> Result<(), GraphIntegrityError> {
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) || !self.nodes.contains_key(&edge.to) {
                return Err(GraphIntegrityError::EdgeToMissingNode {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                });
            }
        }
        Ok(())
    }

    // From impl_traversal.rs - graph mutation and query methods
    /// Add a node to the graph
    pub fn add_node(&mut self, node: TypeNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: TypeEdge) {
        self.edges.push(edge);
    }

    /// Get node by symbol name
    pub fn get_node(&self, symbol: &SymbolName) -> Option<&TypeNode> {
        self.nodes.get(symbol)
    }

    /// Get all edges from a symbol
    pub fn edges_from(&self, symbol: &SymbolName) -> Vec<&TypeEdge> {
        self.edges.iter().filter(|e| &e.from == symbol).collect()
    }

    /// Get all edges to a symbol
    pub fn edges_to(&self, symbol: &SymbolName) -> Vec<&TypeEdge> {
        self.edges.iter().filter(|e| &e.to == symbol).collect()
    }

    /// Check if graph is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.completeness, GraphCompleteness::Complete)
    }

    /// Find all symbols of a specific kind
    pub fn symbols_by_kind(&self, kind: SymbolKind) -> Vec<&TypeNode> {
        self.nodes.values().filter(|n| n.kind == kind).collect()
    }
}

impl GeneratedFilePattern {
    pub fn new(glob: String) -> Result<Self, GeneratedFilePatternError> {
        if glob.trim().is_empty() {
            return Err(GeneratedFilePatternError::Empty);
        }
        Ok(Self { glob })
    }
}

impl TraversalPolicy {
    pub fn validate(&self) -> Result<(), TraversalPolicyError> {
        if self.max_depth == 0 {
            return Err(TraversalPolicyError::ZeroDepth);
        }
        if self.max_nodes == 0 {
            return Err(TraversalPolicyError::ZeroNodes);
        }
        if self.max_edges == 0 {
            return Err(TraversalPolicyError::ZeroEdges);
        }
        Ok(())
    }
}
