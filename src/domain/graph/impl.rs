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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    // Helper to create a minimal TypeNode for testing
    fn test_node(name: &str) -> TypeNode {
        TypeNode {
            id: crate::domain::graph::QualifiedSymbolName {
                symbol: SymbolName::new(name.to_string()).unwrap(),
                module_path: PathBuf::from("/test/file.rs"),
            },
            name: SymbolName::new(name.to_string()).unwrap(),
            kind: SymbolKind::Struct,
            origin: crate::domain::graph::SymbolOrigin::Canonical,
            location: SourceLocation::new(PathBuf::from("/test/file.rs"), 1, 1).unwrap(),
            language: crate::domain::language::Language::Rust,
            generic_parameters: Vec::new(),
        }
    }
    
    #[test]
    fn test_symbol_name_new_valid() {
        let name = SymbolName::new("MyType".to_string()).unwrap();
        assert_eq!(name.as_str(), "MyType");
    }
    
    #[test]
    fn test_symbol_name_new_empty() {
        assert!(matches!(
            SymbolName::new("".to_string()),
            Err(SymbolNameError::Empty)
        ));
        assert!(matches!(
            SymbolName::new("   ".to_string()),
            Err(SymbolNameError::Empty)
        ));
    }
    
    #[test]
    fn test_source_location_new_valid() {
        let loc = SourceLocation::new(PathBuf::from("/test.rs"), 10, 5).unwrap();
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.file, PathBuf::from("/test.rs"));
    }
    
    #[test]
    fn test_source_location_invalid_line() {
        assert!(matches!(
            SourceLocation::new(PathBuf::from("/test.rs"), 0, 5),
            Err(SourceLocationError::InvalidLine)
        ));
    }
    
    #[test]
    fn test_source_location_invalid_column() {
        assert!(matches!(
            SourceLocation::new(PathBuf::from("/test.rs"), 1, 0),
            Err(SourceLocationError::InvalidColumn)
        ));
    }
    
    #[test]
    fn test_type_graph_empty() {
        let graph = TypeGraph::empty();
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
        assert_eq!(graph.warnings().len(), 0);
        assert_eq!(*graph.completeness(), GraphCompleteness::Complete);
    }
    
    #[test]
    fn test_type_graph_add_node() {
        let mut graph = TypeGraph::empty();
        let node = test_node("TypeA");
        graph.add_node(node.clone());
        
        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.get_node(&SymbolName::new("TypeA".to_string()).unwrap()).is_some());
    }
    
    #[test]
    fn test_type_graph_get_node() {
        let mut graph = TypeGraph::empty();
        let node = test_node("TypeA");
        graph.add_node(node);
        
        let found = graph.get_node(&SymbolName::new("TypeA".to_string()).unwrap());
        assert!(found.is_some());
        assert_eq!(found.unwrap().name.as_str(), "TypeA");
    }
    
    #[test]
    fn test_type_graph_statistics_empty() {
        let graph = TypeGraph::empty();
        let stats = graph.statistics();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.unresolved_references.value, 0);
    }
    
    #[test]
    fn test_type_graph_statistics_with_data() {
        let mut graph = TypeGraph::empty();
        graph.add_node(test_node("TypeA"));
        graph.add_node(test_node("TypeB"));
        
        let stats = graph.statistics();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 0);
    }
    
    #[test]
    fn test_type_graph_validate_integrity_empty() {
        let graph = TypeGraph::empty();
        assert!(graph.validate_integrity().is_ok());
    }
    
    #[test]
    fn test_type_graph_is_complete() {
        let graph = TypeGraph::empty();
        assert!(graph.is_complete());
    }
    
    #[test]
    fn test_type_graph_add_edge() {
        let mut graph = TypeGraph::empty();
        graph.add_node(test_node("TypeA"));
        graph.add_node(test_node("TypeB"));
        
        let edge = TypeEdge {
            from: SymbolName::new("TypeA".to_string()).unwrap(),
            to: SymbolName::new("TypeB".to_string()).unwrap(),
            kind: crate::domain::graph::EdgeKind::Contains,
        };
        graph.add_edge(edge);
        
        assert_eq!(graph.edges().len(), 1);
    }
    
    #[test]
    fn test_type_graph_edges_from() {
        let mut graph = TypeGraph::empty();
        graph.add_node(test_node("TypeA"));
        graph.add_node(test_node("TypeB"));
        graph.add_node(test_node("TypeC"));
        
        let edge1 = TypeEdge {
            from: SymbolName::new("TypeA".to_string()).unwrap(),
            to: SymbolName::new("TypeB".to_string()).unwrap(),
            kind: crate::domain::graph::EdgeKind::Contains,
        };
        let edge2 = TypeEdge {
            from: SymbolName::new("TypeA".to_string()).unwrap(),
            to: SymbolName::new("TypeC".to_string()).unwrap(),
            kind: crate::domain::graph::EdgeKind::Extends,
        };
        graph.add_edge(edge1);
        graph.add_edge(edge2);
        
        let edges = graph.edges_from(&SymbolName::new("TypeA".to_string()).unwrap());
        assert_eq!(edges.len(), 2);
    }
    
    #[test]
    fn test_type_graph_edges_to() {
        let mut graph = TypeGraph::empty();
        graph.add_node(test_node("TypeA"));
        graph.add_node(test_node("TypeB"));
        
        let edge = TypeEdge {
            from: SymbolName::new("TypeA".to_string()).unwrap(),
            to: SymbolName::new("TypeB".to_string()).unwrap(),
            kind: crate::domain::graph::EdgeKind::Contains,
        };
        graph.add_edge(edge);
        
        let edges = graph.edges_to(&SymbolName::new("TypeB".to_string()).unwrap());
        assert_eq!(edges.len(), 1);
    }
    
    #[test]
    fn test_type_graph_symbols_by_kind() {
        let mut graph = TypeGraph::empty();
        let mut node_struct = test_node("TypeA");
        node_struct.kind = SymbolKind::Struct;
        graph.add_node(node_struct);
        
        let mut node_enum = test_node("TypeB");
        node_enum.kind = SymbolKind::Enum;
        graph.add_node(node_enum);
        
        let structs = graph.symbols_by_kind(SymbolKind::Struct);
        assert_eq!(structs.len(), 1);
        
        let enums = graph.symbols_by_kind(SymbolKind::Enum);
        assert_eq!(enums.len(), 1);
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

#[cfg(test)]
mod policy_tests {
    use super::*;
    
    #[test]
    fn test_generated_file_pattern_valid() {
        let pattern = GeneratedFilePattern::new("*.generated.ts".to_string()).unwrap();
        assert_eq!(pattern.glob, "*.generated.ts");
    }
    
    #[test]
    fn test_generated_file_pattern_empty() {
        assert!(matches!(
            GeneratedFilePattern::new("".to_string()),
            Err(GeneratedFilePatternError::Empty)
        ));
        assert!(matches!(
            GeneratedFilePattern::new("  ".to_string()),
            Err(GeneratedFilePatternError::Empty)
        ));
    }
}
