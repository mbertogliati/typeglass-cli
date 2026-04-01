//! Comprehensive tests for graph operations and edge cases

use super::*;
use crate::domain::language::Language;
use std::path::PathBuf;

fn create_test_node(name: &str, kind: SymbolKind) -> TypeNode {
    TypeNode {
        id: QualifiedSymbolName {
            module_path: PathBuf::from("test.rs"),
            symbol: SymbolName(name.to_string()),
        },
        name: SymbolName(name.to_string()),
        kind,
        origin: SymbolOrigin::Canonical,
        location: SourceLocation {
            file: PathBuf::from("test.rs"),
            line: 0,
            column: 0,
        },
        language: Language::Rust,
        generic_parameters: vec![],
    }
}

#[test]
fn test_empty_graph_operations() {
    let graph = TypeGraph::empty();
    
    assert_eq!(graph.nodes().len(), 0);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_graph_is_not_empty_after_adding_node() {
    let mut graph = TypeGraph::empty();
    graph.add_node(create_test_node("TestNode", SymbolKind::Struct));
    
    assert_eq!(graph.nodes().len(), 1);
}

#[test]
fn test_adding_duplicate_node_overwrites() {
    let mut graph = TypeGraph::empty();
    let node1 = create_test_node("Node", SymbolKind::Struct);
    let mut node2 = create_test_node("Node", SymbolKind::Enum);
    node2.kind = SymbolKind::Enum;
    
    graph.add_node(node1);
    assert_eq!(graph.nodes().get(&SymbolName("Node".to_string())).unwrap().kind, SymbolKind::Struct);
    
    graph.add_node(node2);
    assert_eq!(graph.nodes().get(&SymbolName("Node".to_string())).unwrap().kind, SymbolKind::Enum);
    assert_eq!(graph.nodes().len(), 1);
}

#[test]
fn test_adding_multiple_edges_between_same_nodes() {
    let mut graph = TypeGraph::empty();
    graph.add_node(create_test_node("A", SymbolKind::Struct));
    graph.add_node(create_test_node("B", SymbolKind::Struct));
    
    graph.add_edge(TypeEdge {
        from: SymbolName("A".to_string()),
        to: SymbolName("B".to_string()),
        kind: EdgeKind::Contains,
    });
    
    graph.add_edge(TypeEdge {
        from: SymbolName("A".to_string()),
        to: SymbolName("B".to_string()),
        kind: EdgeKind::Extends,
    });
    
    // Should have both edges (different kinds)
    let edges: Vec<_> = graph.edges().iter()
        .filter(|e| e.from.0 == "A" && e.to.0 == "B")
        .collect();
    
    assert_eq!(edges.len(), 2);
}

#[test]
fn test_self_referential_edge() {
    let mut graph = TypeGraph::empty();
    graph.add_node(create_test_node("SelfRef", SymbolKind::Struct));
    
    graph.add_edge(TypeEdge {
        from: SymbolName("SelfRef".to_string()),
        to: SymbolName("SelfRef".to_string()),
        kind: EdgeKind::Contains,
    });
    
    assert_eq!(graph.edges().len(), 1);
}

#[test]
fn test_diamond_dependency_pattern() {
    let mut graph = TypeGraph::empty();
    
    // Diamond: Root -> A, B; A -> Leaf; B -> Leaf
    for name in ["Root", "A", "B", "Leaf"] {
        graph.add_node(create_test_node(name, SymbolKind::Struct));
    }
    
    graph.add_edge(TypeEdge {
        from: SymbolName("Root".to_string()),
        to: SymbolName("A".to_string()),
        kind: EdgeKind::Contains,
    });
    graph.add_edge(TypeEdge {
        from: SymbolName("Root".to_string()),
        to: SymbolName("B".to_string()),
        kind: EdgeKind::Contains,
    });
    graph.add_edge(TypeEdge {
        from: SymbolName("A".to_string()),
        to: SymbolName("Leaf".to_string()),
        kind: EdgeKind::Contains,
    });
    graph.add_edge(TypeEdge {
        from: SymbolName("B".to_string()),
        to: SymbolName("Leaf".to_string()),
        kind: EdgeKind::Contains,
    });
    
    assert_eq!(graph.nodes().len(), 4);
    assert_eq!(graph.edges().len(), 4);
}

#[test]
fn test_edge_kind_equality() {
    assert_eq!(EdgeKind::Contains, EdgeKind::Contains);
    assert_ne!(EdgeKind::Contains, EdgeKind::Extends);
    assert_ne!(EdgeKind::Extends, EdgeKind::Instantiates);
}

#[test]
fn test_edge_kind_copy_trait() {
    let kind1 = EdgeKind::Contains;
    let kind2 = kind1; // Should copy, not move
    
    // Both should still be usable
    assert_eq!(kind1, EdgeKind::Contains);
    assert_eq!(kind2, EdgeKind::Contains);
}

#[test]
fn test_symbol_name_equality() {
    let name1 = SymbolName("Test".to_string());
    let name2 = SymbolName("Test".to_string());
    let name3 = SymbolName("Different".to_string());
    
    assert_eq!(name1, name2);
    assert_ne!(name1, name3);
}

#[test]
fn test_graph_with_all_edge_kinds() {
    let mut graph = TypeGraph::empty();
    
    for i in 0..6 {
        graph.add_node(create_test_node(&format!("Node{}", i), SymbolKind::Struct));
    }
    
    let kinds = [
        EdgeKind::Contains,
        EdgeKind::Extends,
        EdgeKind::Instantiates,
        EdgeKind::Variant,
        EdgeKind::Transitions,
        EdgeKind::ReExports,
    ];
    
    for (i, kind) in kinds.iter().enumerate() {
        graph.add_edge(TypeEdge {
            from: SymbolName("Node0".to_string()),
            to: SymbolName(format!("Node{}", i + 1)),
            kind: *kind,
        });
    }
    
    assert_eq!(graph.edges().len(), 6);
    
    // Verify all kinds are present
    for kind in &kinds {
        assert!(graph.edges().iter().any(|e| e.kind == *kind));
    }
}

#[test]
fn test_large_graph_performance() {
    let mut graph = TypeGraph::empty();
    let node_count = 1000;
    
    // Add many nodes
    for i in 0..node_count {
        graph.add_node(create_test_node(&format!("Node{}", i), SymbolKind::Struct));
    }
    
    assert_eq!(graph.nodes().len(), node_count);
    
    // Add edges in a chain
    for i in 0..node_count - 1 {
        graph.add_edge(TypeEdge {
            from: SymbolName(format!("Node{}", i)),
            to: SymbolName(format!("Node{}", i + 1)),
            kind: EdgeKind::Contains,
        });
    }
    
    assert_eq!(graph.edges().len(), node_count - 1);
}

#[test]
fn test_node_with_all_symbol_kinds() {
    let kinds = [
        SymbolKind::Struct,
        SymbolKind::Enum,
        SymbolKind::Interface,
        SymbolKind::TypeAlias,
        SymbolKind::Union,
        SymbolKind::Primitive,
        SymbolKind::External,
    ];
    
    let mut graph = TypeGraph::empty();
    
    for (i, kind) in kinds.iter().enumerate() {
        graph.add_node(create_test_node(&format!("Node{}", i), kind.clone()));
    }
    
    assert_eq!(graph.nodes().len(), kinds.len());
    
    // Verify all kinds are present
    for kind in &kinds {
        assert!(graph.nodes().values().any(|n| n.kind == *kind));
    }
}

#[test]
fn test_qualified_symbol_name_with_nested_modules() {
    let qname = QualifiedSymbolName {
        module_path: PathBuf::from("src/domain/graph/model.rs"),
        symbol: SymbolName("TypeGraph".to_string()),
    };
    
    assert_eq!(qname.symbol.0, "TypeGraph");
    assert!(qname.module_path.to_string_lossy().contains("domain"));
}

#[test]
fn test_source_location_validity() {
    let loc = SourceLocation {
        file: PathBuf::from("src/lib.rs"),
        line: 42,
        column: 10,
    };
    
    assert_eq!(loc.line, 42);
    assert_eq!(loc.column, 10);
    assert!(loc.file.to_string_lossy().ends_with("lib.rs"));
}

#[test]
fn test_graph_completeness_enum() {
    let complete = GraphCompleteness::Complete;
    let partial = GraphCompleteness::Partial {
        unresolved_references: 5,
        signals: vec![],
    };
    
    // Test pattern matching
    match complete {
        GraphCompleteness::Complete => assert!(true),
        _ => panic!("Should be complete"),
    }
    
    match partial {
        GraphCompleteness::Partial { unresolved_references, .. } => {
            assert_eq!(unresolved_references, 5);
        }
        _ => panic!("Should be partial"),
    }
}

#[test]
fn test_bidirectional_edges() {
    let mut graph = TypeGraph::empty();
    graph.add_node(create_test_node("A", SymbolKind::Struct));
    graph.add_node(create_test_node("B", SymbolKind::Struct));
    
    // Add edges in both directions
    graph.add_edge(TypeEdge {
        from: SymbolName("A".to_string()),
        to: SymbolName("B".to_string()),
        kind: EdgeKind::Contains,
    });
    
    graph.add_edge(TypeEdge {
        from: SymbolName("B".to_string()),
        to: SymbolName("A".to_string()),
        kind: EdgeKind::Contains,
    });
    
    assert_eq!(graph.edges().len(), 2);
}

#[test]
fn test_orphan_nodes() {
    let mut graph = TypeGraph::empty();
    
    // Add nodes without any edges
    graph.add_node(create_test_node("Orphan1", SymbolKind::Struct));
    graph.add_node(create_test_node("Orphan2", SymbolKind::Struct));
    graph.add_node(create_test_node("Orphan3", SymbolKind::Struct));
    
    assert_eq!(graph.nodes().len(), 3);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_edge_to_nonexistent_node() {
    let mut graph = TypeGraph::empty();
    graph.add_node(create_test_node("Exists", SymbolKind::Struct));
    
    // Add edge to node that doesn't exist (allowed by API)
    graph.add_edge(TypeEdge {
        from: SymbolName("Exists".to_string()),
        to: SymbolName("DoesNotExist".to_string()),
        kind: EdgeKind::Contains,
    });
    
    assert_eq!(graph.edges().len(), 1);
    assert_eq!(graph.nodes().len(), 1); // Still only one node
}
