use super::{FormatterError, GraphFormatter};
use crate::domain::graph::{EdgeKind, TypeGraph};

/// Mermaid diagram format formatter
pub struct MermaidFormatter {
    options: MermaidOptions,
}

#[derive(Debug, Clone)]
pub struct MermaidOptions {
    /// Graph direction (TD, LR, BT, RL)
    pub direction: String,
    /// Include edge labels
    pub show_edge_labels: bool,
}

impl Default for MermaidOptions {
    fn default() -> Self {
        Self {
            direction: "TD".to_string(),
            show_edge_labels: true,
        }
    }
}

impl MermaidFormatter {
    pub fn new() -> Self {
        Self {
            options: MermaidOptions::default(),
        }
    }
    
    pub fn with_options(options: MermaidOptions) -> Self {
        Self { options }
    }
}

impl Default for MermaidFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphFormatter for MermaidFormatter {
    fn format(&self, graph: &TypeGraph) -> Result<String, FormatterError> {
        let mut output = String::new();
        
        // Graph header
        output.push_str(&format!("graph {}\n", self.options.direction));
        
        // Edges (Mermaid defines nodes implicitly through edges)
        for edge in graph.edges() {
            let from_id = sanitize_mermaid_id(&edge.from.0);
            let to_id = sanitize_mermaid_id(&edge.to.0);
            let from_label = &edge.from.0;
            let to_label = &edge.to.0;
            
            let edge_arrow = edge_kind_arrow(&edge.kind);
            
            if self.options.show_edge_labels {
                let edge_label = edge_kind_label(&edge.kind);
                output.push_str(&format!(
                    "  {}[{}] {}|{}| {}[{}]\n",
                    from_id, from_label, edge_arrow, edge_label, to_id, to_label
                ));
            } else {
                output.push_str(&format!(
                    "  {}[{}] {} {}[{}]\n",
                    from_id, from_label, edge_arrow, to_id, to_label
                ));
            }
        }
        
        // Add isolated nodes (nodes with no edges)
        for node in graph.nodes().values() {
            let has_edges = graph.edges().iter().any(|e| {
                e.from == node.name || e.to == node.name
            });
            
            if !has_edges {
                let node_id = sanitize_mermaid_id(&node.name.0);
                let node_label = &node.name.0;
                output.push_str(&format!("  {}[{}]\n", node_id, node_label));
            }
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "mmd"
    }
    
    fn name(&self) -> &str {
        "Mermaid"
    }
}

/// Sanitize identifier for Mermaid (alphanumeric + underscore)
fn sanitize_mermaid_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Map EdgeKind to Mermaid arrow type
fn edge_kind_arrow(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "-->",
        EdgeKind::Extends => "==>",       // Thick arrow
        EdgeKind::Variant => "-.->",      // Dotted arrow
        EdgeKind::Transitions => "--",    // Dash (no arrow)
        EdgeKind::Instantiates => "==>",  // Thick arrow
        EdgeKind::ReExports => "-.->",    // Dotted arrow
    }
}

/// Map EdgeKind to human-readable label
fn edge_kind_label(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Extends => "extends",
        EdgeKind::Variant => "variant",
        EdgeKind::Transitions => "transitions",
        EdgeKind::Instantiates => "instantiates",
        EdgeKind::ReExports => "re-exports",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::{
        QualifiedSymbolName, SourceLocation, SymbolKind, SymbolName, SymbolOrigin, TypeEdge, TypeNode,
    };
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
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("Hello::World"), "Hello__World");
        assert_eq!(sanitize_mermaid_id("Foo<Bar>"), "Foo_Bar_");
        assert_eq!(sanitize_mermaid_id("MyType"), "MyType");
    }

    #[test]
    fn test_edge_kind_arrow() {
        assert_eq!(edge_kind_arrow(&EdgeKind::Contains), "-->");
        assert_eq!(edge_kind_arrow(&EdgeKind::Extends), "==>");
        assert_eq!(edge_kind_arrow(&EdgeKind::Variant), "-.->")
    }

    #[test]
    fn test_format_empty_graph() {
        let formatter = MermaidFormatter::new();
        let graph = TypeGraph::empty();
        let output = formatter.format(&graph).unwrap();
        
        assert_eq!(output, "graph TD\n");
    }

    #[test]
    fn test_format_single_isolated_node() {
        let formatter = MermaidFormatter::new();
        let mut graph = TypeGraph::empty();
        graph.add_node(create_test_node("MyStruct", SymbolKind::Struct));
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("graph TD"));
        assert!(output.contains("MyStruct[MyStruct]"));
    }

    #[test]
    fn test_format_node_with_edge() {
        let formatter = MermaidFormatter::new();
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("Foo", SymbolKind::Struct));
        graph.add_node(create_test_node("Bar", SymbolKind::Struct));
        graph.add_edge(TypeEdge {
            from: SymbolName("Foo".to_string()),
            to: SymbolName("Bar".to_string()),
            kind: EdgeKind::Contains,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("Foo[Foo]"));
        assert!(output.contains("Bar[Bar]"));
        assert!(output.contains("-->"));
        assert!(output.contains("contains"));
    }

    #[test]
    fn test_format_extends_relationship() {
        let formatter = MermaidFormatter::new();
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("A", SymbolKind::Struct));
        graph.add_node(create_test_node("B", SymbolKind::Interface));
        graph.add_edge(TypeEdge {
            from: SymbolName("A".to_string()),
            to: SymbolName("B".to_string()),
            kind: EdgeKind::Extends,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("==>"));
        assert!(output.contains("extends"));
    }

    #[test]
    fn test_formatter_without_edge_labels() {
        let options = MermaidOptions {
            direction: "LR".to_string(),
            show_edge_labels: false,
        };
        let formatter = MermaidFormatter::with_options(options);
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("Foo", SymbolKind::Struct));
        graph.add_node(create_test_node("Bar", SymbolKind::Struct));
        graph.add_edge(TypeEdge {
            from: SymbolName("Foo".to_string()),
            to: SymbolName("Bar".to_string()),
            kind: EdgeKind::Contains,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("graph LR"));
        assert!(output.contains("-->"));
        assert!(!output.contains("|contains|"));
    }

    #[test]
    fn test_file_extension() {
        let formatter = MermaidFormatter::new();
        assert_eq!(formatter.file_extension(), "mmd");
    }

    #[test]
    fn test_formatter_name() {
        let formatter = MermaidFormatter::new();
        assert_eq!(formatter.name(), "Mermaid");
    }

    #[test]
    fn test_format_multiple_edges() {
        let formatter = MermaidFormatter::new();
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("A", SymbolKind::Struct));
        graph.add_node(create_test_node("B", SymbolKind::Struct));
        graph.add_node(create_test_node("C", SymbolKind::Struct));
        
        graph.add_edge(TypeEdge {
            from: SymbolName("A".to_string()),
            to: SymbolName("B".to_string()),
            kind: EdgeKind::Contains,
        });
        graph.add_edge(TypeEdge {
            from: SymbolName("B".to_string()),
            to: SymbolName("C".to_string()),
            kind: EdgeKind::Extends,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // Header + 2 edges
        assert!(output.contains("A[A] -->|contains| B[B]"));
        assert!(output.contains("B[B] ==>|extends| C[C]"));
    }
}
