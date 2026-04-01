use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::graph::{GraphCompleteness, TypeGraph};
use super::symbols::{QualifiedSymbolName, SymbolKind, SymbolName};

/// Direction of graph traversal from an entry point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum TraversalDirection {
    /// Follow edges pointing TO the entry point (dependencies)
    Upstream,
    /// Follow edges pointing FROM the entry point (dependents)
    #[default]
    Downstream,
    /// Follow edges in both directions
    Both,
}


/// Filters for controlling which symbols are included during traversal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct TraversalFilter {
    /// Include only symbols matching these patterns (glob syntax)
    pub include_patterns: Vec<String>,
    /// Exclude symbols matching these patterns (glob syntax)
    pub exclude_patterns: Vec<String>,
    /// Include only these symbol kinds
    pub include_kinds: Option<Vec<SymbolKind>>,
    /// Exclude these symbol kinds
    pub exclude_kinds: Vec<SymbolKind>,
    /// Include only symbols from these module paths
    pub include_modules: Vec<PathBuf>,
    /// Exclude symbols from these module paths
    pub exclude_modules: Vec<PathBuf>,
}


impl TraversalFilter {
    pub fn matches_symbol(&self, _symbol: &SymbolName, kind: &SymbolKind, module: &PathBuf) -> bool {
        // If include_kinds is specified, symbol kind must be in the list
        if let Some(ref kinds) = self.include_kinds {
            if !kinds.contains(kind) {
                return false;
            }
        }

        // Exclude kinds always take precedence
        if self.exclude_kinds.contains(kind) {
            return false;
        }

        // Module filters
        if !self.include_modules.is_empty() && !self.include_modules.contains(module) {
            return false;
        }

        if self.exclude_modules.contains(module) {
            return false;
        }

        // Pattern matching would go here (requires glob crate)
        // For now, accept all patterns
        true
    }

    pub fn is_permissive(&self) -> bool {
        self.include_patterns.is_empty()
            && self.exclude_patterns.is_empty()
            && self.include_kinds.is_none()
            && self.exclude_kinds.is_empty()
            && self.include_modules.is_empty()
            && self.exclude_modules.is_empty()
    }
}

/// Result of a graph traversal operation with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalResult {
    /// The resulting type graph
    pub graph: TypeGraph,
    /// Direction that was traversed
    pub direction: TraversalDirection,
    /// Filter that was applied
    pub filter: TraversalFilter,
    /// Entry point of the traversal
    pub entry_point: QualifiedSymbolName,
    /// Maximum depth reached during traversal
    pub max_depth_reached: u8,
    /// Whether the traversal completed fully
    pub completeness: GraphCompleteness,
}

impl TraversalResult {
    pub fn is_complete(&self) -> bool {
        matches!(self.completeness, GraphCompleteness::Complete)
    }

    pub fn is_partial(&self) -> bool {
        !self.is_complete()
    }

    pub fn node_count(&self) -> usize {
        self.graph.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edges.len()
    }
}

/// Progress information during long-running traversals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalProgress {
    /// Number of nodes visited so far
    pub nodes_visited: usize,
    /// Number of edges followed so far
    pub edges_followed: usize,
    /// Current depth level being processed
    pub current_depth: u8,
    /// Maximum depth to traverse
    pub max_depth: u8,
    /// Estimated percentage complete (0-100)
    pub estimated_percent_complete: u8,
    /// Whether the traversal is still actively processing
    pub is_active: bool,
}

impl TraversalProgress {
    pub fn new(max_depth: u8) -> Self {
        Self {
            nodes_visited: 0,
            edges_followed: 0,
            current_depth: 0,
            max_depth,
            estimated_percent_complete: 0,
            is_active: true,
        }
    }

    pub fn update(&mut self, nodes: usize, edges: usize, depth: u8) {
        self.nodes_visited = nodes;
        self.edges_followed = edges;
        self.current_depth = depth;
        
        // Simple percentage estimate based on depth
        if self.max_depth > 0 {
            self.estimated_percent_complete = 
                ((depth as f32 / self.max_depth as f32) * 100.0).min(100.0) as u8;
        }
    }

    pub fn complete(&mut self) {
        self.is_active = false;
        self.estimated_percent_complete = 100;
    }

    pub fn is_complete(&self) -> bool {
        !self.is_active
    }
}
