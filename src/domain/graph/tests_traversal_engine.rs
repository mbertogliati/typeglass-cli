//! Tests for graph traversal engine with real scenarios

#[cfg(test)]
mod traversal_engine_tests {
    use std::path::PathBuf;
    use std::collections::HashMap;

    use crate::domain::graph::{
        EdgeKind, GraphCompleteness, GraphTraversal, QualifiedSymbolName, SourceLocation,
        SymbolKind, SymbolName, SymbolOrigin, TraversalDirection, TraversalFilter,
        TypeEdge, TypeGraph, TypeNode,
    };
    use crate::domain::language::Language;

    fn create_test_node(name: &str, file: &str, line: u32) -> TypeNode {
        TypeNode {
            id: QualifiedSymbolName {
                module_path: PathBuf::from(file),
                symbol: SymbolName(name.to_string()),
            },
            name: SymbolName(name.to_string()),
            kind: SymbolKind::Struct,
            origin: SymbolOrigin::Canonical,
            location: SourceLocation {
                file: PathBuf::from(file),
                line,
                column: 0,
            },
            language: Language::Rust,
            generic_parameters: vec![],
        }
    }

    fn create_test_edge(from: &str, to: &str) -> TypeEdge {
        TypeEdge {
            from: SymbolName(from.to_string()),
            to: SymbolName(to.to_string()),
            kind: EdgeKind::Contains,
        }
    }

    /// Test: Cyclic graph should not cause infinite loop
    #[test]
    fn traversal_handles_cycles_correctly() {
        // Create a cycle: A -> B -> C -> A
        let mut nodes = HashMap::new();
        nodes.insert(SymbolName("A".to_string()), create_test_node("A", "a.rs", 1));
        nodes.insert(SymbolName("B".to_string()), create_test_node("B", "b.rs", 1));
        nodes.insert(SymbolName("C".to_string()), create_test_node("C", "c.rs", 1));

        let edges = vec![
            create_test_edge("A", "B"),
            create_test_edge("B", "C"),
            create_test_edge("C", "A"), // Cycle back to A
        ];

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("a.rs"),
            symbol: SymbolName("A".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());

        // Should visit all 3 nodes exactly once (no infinite loop)
        assert_eq!(result.graph.nodes().len(), 3);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("B".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("C".to_string())));

        // Should have all 3 edges
        assert_eq!(result.graph.edges().len(), 3);
    }

    /// Test: Max depth should stop traversal at correct level
    #[test]
    fn traversal_respects_max_depth() {
        // Create chain: A -> B -> C -> D -> E
        let mut nodes = HashMap::new();
        for name in &["A", "B", "C", "D", "E"] {
            nodes.insert(SymbolName(name.to_string()), create_test_node(name, "test.rs", 1));
        }

        let edges = vec![
            create_test_edge("A", "B"),
            create_test_edge("B", "C"),
            create_test_edge("C", "D"),
            create_test_edge("D", "E"),
        ];

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("A".to_string()),
        };

        // Depth 0: only A
        let mut traversal = GraphTraversal::new(0);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());
        assert_eq!(result.graph.nodes().len(), 1);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));

        // Depth 1: A and B
        let mut traversal = GraphTraversal::new(1);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());
        assert_eq!(result.graph.nodes().len(), 2);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("B".to_string())));
        assert!(!result.graph.nodes().contains_key(&SymbolName("C".to_string())));

        // Depth 2: A, B, and C
        let mut traversal = GraphTraversal::new(2);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());
        assert_eq!(result.graph.nodes().len(), 3);
        assert!(result.graph.nodes().contains_key(&SymbolName("C".to_string())));
        assert!(!result.graph.nodes().contains_key(&SymbolName("D".to_string())));
    }

    /// Test: Diamond dependency should visit shared node once
    #[test]
    fn traversal_handles_diamond_dependencies() {
        // Create diamond: A -> B,C  and  B,C -> D
        let mut nodes = HashMap::new();
        for name in &["A", "B", "C", "D"] {
            nodes.insert(SymbolName(name.to_string()), create_test_node(name, "test.rs", 1));
        }

        let edges = vec![
            create_test_edge("A", "B"),
            create_test_edge("A", "C"),
            create_test_edge("B", "D"),
            create_test_edge("C", "D"),
        ];

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("A".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());

        // Should visit D only once despite two paths to it
        assert_eq!(result.graph.nodes().len(), 4);
        
        // D should appear exactly once in nodes
        assert_eq!(
            result.graph.nodes().keys().filter(|k| k.0 == "D").count(),
            1,
            "D should be visited exactly once"
        );

        // Should have all 4 edges
        assert_eq!(result.graph.edges().len(), 4);
    }

    /// Test: Upstream traversal goes in reverse
    #[test]
    fn traversal_upstream_goes_backward() {
        // Create chain: A -> B -> C
        let mut nodes = HashMap::new();
        for name in &["A", "B", "C"] {
            nodes.insert(SymbolName(name.to_string()), create_test_node(name, "test.rs", 1));
        }

        let edges = vec![
            create_test_edge("A", "B"),
            create_test_edge("B", "C"),
        ];

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        // Start from C and go upstream
        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("C".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Upstream, &TraversalFilter::default());

        // Should find C, B, and A (going backwards)
        assert_eq!(result.graph.nodes().len(), 3);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("B".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("C".to_string())));
    }

    /// Test: Both direction explores all connected nodes
    #[test]
    fn traversal_both_explores_bidirectionally() {
        // Create: A -> B <- C (B in the middle)
        let mut nodes = HashMap::new();
        for name in &["A", "B", "C"] {
            nodes.insert(SymbolName(name.to_string()), create_test_node(name, "test.rs", 1));
        }

        let edges = vec![
            create_test_edge("A", "B"),
            create_test_edge("C", "B"),
        ];

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        // Start from B and go both directions
        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("B".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Both, &TraversalFilter::default());

        // Should find A, B, and C (both directions from B)
        assert_eq!(result.graph.nodes().len(), 3);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("B".to_string())));
        assert!(result.graph.nodes().contains_key(&SymbolName("C".to_string())));
    }

    /// Test: Empty graph returns only entry node
    #[test]
    fn traversal_handles_empty_graph() {
        let mut nodes = HashMap::new();
        nodes.insert(SymbolName("A".to_string()), create_test_node("A", "test.rs", 1));

        let graph = TypeGraph {
            nodes,
            edges: vec![], // No edges
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("A".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());

        // Should only have entry node
        assert_eq!(result.graph.nodes().len(), 1);
        assert!(result.graph.nodes().contains_key(&SymbolName("A".to_string())));
        assert_eq!(result.graph.edges().len(), 0);
    }

    /// Test: Non-existent entry point returns empty result
    #[test]
    fn traversal_handles_missing_entry_point() {
        let mut nodes = HashMap::new();
        nodes.insert(SymbolName("A".to_string()), create_test_node("A", "test.rs", 1));

        let graph = TypeGraph {
            nodes,
            edges: vec![],
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("NonExistent".to_string()), // Doesn't exist
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());

        // Should return empty result
        assert_eq!(result.graph.nodes().len(), 0);
        assert_eq!(result.graph.edges().len(), 0);
    }

    /// Test: Large graph with many nodes performs efficiently
    #[test]
    fn traversal_handles_large_graph() {
        // Create star topology: Center connected to 100 nodes
        let mut nodes = HashMap::new();
        nodes.insert(SymbolName("Center".to_string()), create_test_node("Center", "test.rs", 1));
        
        let mut edges = vec![];
        for i in 0..100 {
            let name = format!("Node{}", i);
            nodes.insert(SymbolName(name.clone()), create_test_node(&name, "test.rs", i as u32));
            edges.push(create_test_edge("Center", &name));
        }

        let graph = TypeGraph {
            nodes,
            edges,
            warnings: vec![],
            completeness: GraphCompleteness::Complete,
        };

        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName("Center".to_string()),
        };

        let mut traversal = GraphTraversal::new(10);
        let result = traversal.traverse(&graph, &entry, TraversalDirection::Downstream, &TraversalFilter::default());

        // Should find all 101 nodes (center + 100 connected)
        assert_eq!(result.graph.nodes().len(), 101);
        assert_eq!(result.graph.edges().len(), 100);
    }
}
