use std::collections::{HashMap, HashSet, VecDeque};

use crate::domain::graph::{
    PartialGraph, PartialResultSignal, QualifiedSymbolName, SymbolName, TraversalDirection, TraversalFilter, TraversalProgress,
    TraversalResult, TypeEdge, TypeGraph, UnresolvedReference,
};

/// Graph traversal engine
pub struct GraphTraversal {
    visited: HashSet<SymbolName>,
    queue: VecDeque<(SymbolName, u8)>, // (symbol, depth)
    max_depth: u8,
}

impl GraphTraversal {
    pub fn new(max_depth: u8) -> Self {
        Self {
            visited: HashSet::new(),
            queue: VecDeque::new(),
            max_depth,
        }
    }

    /// Traverse graph from entry point
    pub fn traverse(
        &mut self,
        graph: &TypeGraph,
        entry_point: &QualifiedSymbolName,
        direction: TraversalDirection,
        filter: &TraversalFilter,
    ) -> TraversalResult {
        self.visited.clear();
        self.queue.clear();

        let entry_symbol = entry_point.symbol.clone();
        
        // Initialize traversal
        self.queue.push_back((entry_symbol.clone(), 0));
        self.visited.insert(entry_symbol.clone());

        let mut result_nodes = HashMap::new();
        let mut result_edges = Vec::new();
        let mut max_depth_reached = 0;

        // BFS traversal
        while let Some((current_symbol, current_depth)) = self.queue.pop_front() {
            max_depth_reached = max_depth_reached.max(current_depth);

            // Get current node
            if let Some(node) = graph.nodes().get(&current_symbol) {
                // Check filter
                if !filter.matches_symbol(&current_symbol, &node.kind, &node.location.file) {
                    continue;
                }

                result_nodes.insert(current_symbol.clone(), node.clone());

                // Don't traverse beyond max depth
                if current_depth >= self.max_depth {
                    continue;
                }

                // Find edges based on direction
                let edges = self.find_edges(graph, &current_symbol, direction);
                
                for edge in edges {
                    result_edges.push(edge.clone());
                    
                    let next_symbol = match direction {
                        TraversalDirection::Downstream => &edge.to,
                        TraversalDirection::Upstream => &edge.from,
                        TraversalDirection::Both => {
                            if &edge.from == &current_symbol {
                                &edge.to
                            } else {
                                &edge.from
                            }
                        }
                    };

                    if !self.visited.contains(next_symbol) {
                        let next = next_symbol.clone();
                        self.visited.insert(next.clone());
                        self.queue.push_back((next, current_depth + 1));
                    }
                }
            }
        }

        TraversalResult {
            graph: TypeGraph {
                nodes: result_nodes,
                edges: result_edges,
                warnings: graph.warnings().to_vec(),
                completeness: graph.completeness().clone(),
            },
            direction,
            filter: filter.clone(),
            entry_point: entry_point.clone(),
            max_depth_reached,
            completeness: graph.completeness().clone(),
        }
    }

    fn find_edges<'a>(
        &self,
        graph: &'a TypeGraph,
        symbol: &SymbolName,
        direction: TraversalDirection,
    ) -> Vec<&'a TypeEdge> {
        graph
            .edges()
            .iter()
            .filter(|edge| match direction {
                TraversalDirection::Downstream => &edge.from == symbol,
                TraversalDirection::Upstream => &edge.to == symbol,
                TraversalDirection::Both => &edge.from == symbol || &edge.to == symbol,
            })
            .collect()
    }

    /// Create progress snapshot
    pub fn current_progress(&self, max_depth: u8) -> TraversalProgress {
        let mut progress = TraversalProgress::new(max_depth);
        progress.update(self.visited.len(), 0, 0);
        progress
    }
}

impl PartialGraph {
    /// Add unresolved reference
    pub fn add_unresolved(&mut self, reference: UnresolvedReference) {
        self.unresolved_references.push(reference);
    }

    /// Add partial signal
    pub fn add_signal(&mut self, signal: PartialResultSignal) {
        self.partial_signals.push(signal);
    }

    /// Calculate completeness based on resolved vs total
    pub fn calculate_completeness(&mut self, total_expected: usize) {
        if total_expected == 0 {
            self.completeness_percentage = 100;
            return;
        }

        let resolved = self.nodes.len();
        let percentage = ((resolved as f64 / total_expected as f64) * 100.0) as u8;
        self.completeness_percentage = percentage.min(100);
    }
}

#[cfg(test)]
mod traversal_impl_tests {
    use super::*;
    use crate::domain::language::Language;
    use std::path::PathBuf;

    fn make_node(name: &str, kind: SymbolKind) -> TypeNode {
        TypeNode {
            id: QualifiedSymbolName {
                module_path: PathBuf::from("test.ts"),
                symbol: SymbolName(name.to_string()),
            },
            name: SymbolName(name.to_string()),
            kind,
            origin: SymbolOrigin::Canonical,
            location: crate::domain::graph::SourceLocation {
                file: PathBuf::from("test.ts"),
                line: 1,
                column: 1,
            },
            language: Language::TypeScript,
            generic_parameters: vec![],
        }
    }

    fn make_edge(from: &str, to: &str, kind: EdgeKind) -> TypeEdge {
        TypeEdge {
            from: SymbolName(from.to_string()),
            to: SymbolName(to.to_string()),
            kind,
        }
    }

    #[test]
    fn graph_traversal_basic() {
        let mut graph = TypeGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            warnings: Vec::new(),
            completeness: GraphCompleteness::Complete,
        };

        // A -> B -> C
        graph.add_node(make_node("A", SymbolKind::Struct));
        graph.add_node(make_node("B", SymbolKind::Struct));
        graph.add_node(make_node("C", SymbolKind::Struct));
        graph.add_edge(make_edge("A", "B", EdgeKind::Contains));
        graph.add_edge(make_edge("B", "C", EdgeKind::Contains));

        let mut traversal = GraphTraversal::new(10);
        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.ts"),
            symbol: SymbolName("A".to_string()),
        };

        let result = traversal.traverse(
            &graph,
            &entry,
            TraversalDirection::Downstream,
            &TraversalFilter::default(),
        );

        assert_eq!(result.graph.nodes().len(), 3);
        assert_eq!(result.graph.edges().len(), 2);
    }

    #[test]
    fn graph_traversal_respects_depth() {
        let mut graph = TypeGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            warnings: Vec::new(),
            completeness: GraphCompleteness::Complete,
        };

        // A -> B -> C
        graph.add_node(make_node("A", SymbolKind::Struct));
        graph.add_node(make_node("B", SymbolKind::Struct));
        graph.add_node(make_node("C", SymbolKind::Struct));
        graph.add_edge(make_edge("A", "B", EdgeKind::Contains));
        graph.add_edge(make_edge("B", "C", EdgeKind::Contains));

        let mut traversal = GraphTraversal::new(1); // Max depth 1
        let entry = QualifiedSymbolName {
            module_path: PathBuf::from("test.ts"),
            symbol: SymbolName("A".to_string()),
        };

        let result = traversal.traverse(
            &graph,
            &entry,
            TraversalDirection::Downstream,
            &TraversalFilter::default(),
        );

        // Should only get A and B, not C
        assert_eq!(result.graph.nodes().len(), 2);
    }

    #[test]
    fn partial_graph_completeness_calculation() {
        let mut partial = PartialGraph::new();
        
        partial.nodes.insert(
            SymbolName("A".to_string()),
            make_node("A", SymbolKind::Struct),
        );
        partial.nodes.insert(
            SymbolName("B".to_string()),
            make_node("B", SymbolKind::Struct),
        );

        partial.calculate_completeness(10); // 2 out of 10 expected
        assert_eq!(partial.completeness_percentage, 20);
    }
}

