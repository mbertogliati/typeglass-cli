use super::{FormatterError, GraphFormatter};
use crate::domain::graph::{EdgeKind, SymbolKind, TypeGraph};

/// Graphviz DOT format formatter
pub struct DotFormatter {
    options: DotOptions,
}

#[derive(Debug, Clone)]
pub struct DotOptions {
    /// Include node shapes based on symbol kind
    pub use_shapes: bool,
    /// Include edge labels for relationship types
    pub show_edge_labels: bool,
    /// Rankdir for graph layout (TB, LR, BT, RL)
    pub rankdir: String,
}

impl Default for DotOptions {
    fn default() -> Self {
        Self {
            use_shapes: true,
            show_edge_labels: true,
            rankdir: "TB".to_string(),
        }
    }
}

impl DotFormatter {
    pub fn new() -> Self {
        Self {
            options: DotOptions::default(),
        }
    }
    
    pub fn with_options(options: DotOptions) -> Self {
        Self { options }
    }
}

impl Default for DotFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphFormatter for DotFormatter {
    fn format(&self, graph: &TypeGraph) -> Result<String, FormatterError> {
        let mut output = String::new();
        
        // Graph header
        output.push_str("digraph TypeGraph {\n");
        output.push_str(&format!("  rankdir={};\n", self.options.rankdir));
        output.push_str("  node [fontname=\"Helvetica\"];\n");
        output.push_str("  edge [fontname=\"Helvetica\"];\n\n");
        
        // Nodes
        for node in graph.nodes().values() {
            let node_id = escape_dot_id(&node.name.0);
            let label = escape_dot_label(&node.name.0);
            
            let shape = if self.options.use_shapes {
                node_shape(&node.kind)
            } else {
                "box"
            };
            
            output.push_str(&format!(
                "  {} [label=\"{}\", shape={}];\n",
                node_id, label, shape
            ));
        }
        
        output.push('\n');
        
        // Edges
        for edge in graph.edges() {
            let from_id = escape_dot_id(&edge.from.0);
            let to_id = escape_dot_id(&edge.to.0);
            
            if self.options.show_edge_labels {
                let edge_label = edge_kind_label(&edge.kind);
                let edge_style = edge_kind_style(&edge.kind);
                output.push_str(&format!(
                    "  {} -> {} [label=\"{}\", {}];\n",
                    from_id, to_id, edge_label, edge_style
                ));
            } else {
                let edge_style = edge_kind_style(&edge.kind);
                output.push_str(&format!(
                    "  {} -> {} [{}];\n",
                    from_id, to_id, edge_style
                ));
            }
        }
        
        output.push_str("}\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "dot"
    }
    
    fn name(&self) -> &str {
        "Graphviz DOT"
    }
}

/// Escape DOT identifier (node IDs can't have special chars)
fn escape_dot_id(s: &str) -> String {
    // Replace non-alphanumeric with underscore
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Escape DOT label (quoted strings need escaping)
fn escape_dot_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Map SymbolKind to DOT node shape
fn node_shape(kind: &SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Struct => "box",
        SymbolKind::Enum => "diamond",
        SymbolKind::Interface => "ellipse",
        SymbolKind::TypeAlias => "note",
        SymbolKind::Union => "box",
        SymbolKind::Primitive => "oval",
        SymbolKind::External => "hexagon",
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

/// Map EdgeKind to DOT style attributes
fn edge_kind_style(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "style=solid",
        EdgeKind::Extends => "style=bold, color=blue",
        EdgeKind::Variant => "style=dotted",
        EdgeKind::Transitions => "style=dashed",
        EdgeKind::Instantiates => "style=dashed, color=red",
        EdgeKind::ReExports => "style=dotted, color=gray",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::{
        QualifiedSymbolName, SourceLocation, SymbolName, SymbolOrigin, TypeEdge, TypeNode,
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
    fn test_escape_dot_id_removes_special_chars() {
        assert_eq!(escape_dot_id("Hello::World"), "Hello__World");
        assert_eq!(escape_dot_id("Foo<Bar>"), "Foo_Bar_");
        assert_eq!(escape_dot_id("MyType"), "MyType");
    }

    #[test]
    fn test_escape_dot_label_escapes_quotes() {
        assert_eq!(escape_dot_label("Hello \"World\""), "Hello \\\"World\\\"");
        assert_eq!(escape_dot_label("Line1\nLine2"), "Line1\\nLine2");
        assert_eq!(escape_dot_label("C:\\path"), "C:\\\\path");
    }

    #[test]
    fn test_node_shape_mapping() {
        assert_eq!(node_shape(&SymbolKind::Struct), "box");
        assert_eq!(node_shape(&SymbolKind::Enum), "diamond");
        assert_eq!(node_shape(&SymbolKind::Interface), "ellipse");
        assert_eq!(node_shape(&SymbolKind::External), "hexagon");
    }

    #[test]
    fn test_edge_kind_label() {
        assert_eq!(edge_kind_label(&EdgeKind::Contains), "contains");
        assert_eq!(edge_kind_label(&EdgeKind::Extends), "extends");
        assert_eq!(edge_kind_label(&EdgeKind::Instantiates), "instantiates");
    }

    #[test]
    fn test_format_empty_graph() {
        let formatter = DotFormatter::new();
        let graph = TypeGraph::empty();
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("digraph TypeGraph"));
        assert!(output.contains("rankdir=TB"));
        assert!(output.ends_with("}\n"));
    }

    #[test]
    fn test_format_single_node() {
        let formatter = DotFormatter::new();
        let mut graph = TypeGraph::empty();
        graph.add_node(create_test_node("MyStruct", SymbolKind::Struct));
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("MyStruct"));
        assert!(output.contains("shape=box"));
    }

    #[test]
    fn test_format_node_with_edge() {
        let formatter = DotFormatter::new();
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("Foo", SymbolKind::Struct));
        graph.add_node(create_test_node("Bar", SymbolKind::Struct));
        graph.add_edge(TypeEdge {
            from: SymbolName("Foo".to_string()),
            to: SymbolName("Bar".to_string()),
            kind: EdgeKind::Contains,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("Foo"));
        assert!(output.contains("Bar"));
        assert!(output.contains("Foo -> Bar"));
        assert!(output.contains("contains"));
    }

    #[test]
    fn test_format_different_edge_kinds() {
        let formatter = DotFormatter::new();
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("A", SymbolKind::Struct));
        graph.add_node(create_test_node("B", SymbolKind::Interface));
        graph.add_edge(TypeEdge {
            from: SymbolName("A".to_string()),
            to: SymbolName("B".to_string()),
            kind: EdgeKind::Extends,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("extends"));
        assert!(output.contains("color=blue"));
    }

    #[test]
    fn test_formatter_without_edge_labels() {
        let options = DotOptions {
            use_shapes: true,
            show_edge_labels: false,
            rankdir: "LR".to_string(),
        };
        let formatter = DotFormatter::with_options(options);
        let mut graph = TypeGraph::empty();
        
        graph.add_node(create_test_node("Foo", SymbolKind::Struct));
        graph.add_node(create_test_node("Bar", SymbolKind::Struct));
        graph.add_edge(TypeEdge {
            from: SymbolName("Foo".to_string()),
            to: SymbolName("Bar".to_string()),
            kind: EdgeKind::Contains,
        });
        
        let output = formatter.format(&graph).unwrap();
        
        assert!(output.contains("rankdir=LR"));
        assert!(output.contains("Foo -> Bar"));
        assert!(!output.contains("label=\"contains\""));
    }

    #[test]
    fn test_file_extension() {
        let formatter = DotFormatter::new();
        assert_eq!(formatter.file_extension(), "dot");
    }

    #[test]
    fn test_formatter_name() {
        let formatter = DotFormatter::new();
        assert_eq!(formatter.name(), "Graphviz DOT");
    }
}
