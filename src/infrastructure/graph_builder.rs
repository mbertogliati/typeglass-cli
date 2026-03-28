use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::domain::graph::{
    EdgeKind, GraphCompleteness, QualifiedSymbolName, SourceLocation, SymbolKind, SymbolName,
    SymbolOrigin, TraversalDirection, TypeEdge, TypeGraph, TypeNode,
};
use crate::domain::language::{Language, LspServerConfig};
use crate::infrastructure::{LspClient, LspClientError, Location, SymbolFinder, SymbolFinderError};

/// Lazy graph builder - builds TypeGraph incrementally via LSP queries
pub struct LazyGraphBuilder {
    lsp_client: LspClient,
    workspace_root: PathBuf,
    language: Language,
}

#[derive(Debug, thiserror::Error)]
pub enum GraphBuilderError {
    #[error("LSP client error: {0}")]
    LspError(#[from] LspClientError),
    
    #[error("Symbol finder error: {0}")]
    FinderError(#[from] SymbolFinderError),
    
    #[error("Failed to initialize LSP")]
    InitializationFailed,
}

impl LazyGraphBuilder {
    /// Create a new lazy graph builder
    pub async fn new(
        workspace_root: PathBuf,
        language: Language,
    ) -> Result<Self, GraphBuilderError> {
        let config = LspServerConfig::for_language(language)
            .ok_or(GraphBuilderError::InitializationFailed)?;

        let mut lsp_client = LspClient::new(config);
        lsp_client.initialize(workspace_root.clone()).await?;

        Ok(Self {
            lsp_client,
            workspace_root,
            language,
        })
    }

    /// Build graph starting from entry symbol
    pub async fn build_from_symbol(
        &mut self,
        symbol_name: &str,
        direction: TraversalDirection,
        max_depth: u8,
    ) -> Result<TypeGraph, GraphBuilderError> {
        // Find entry point
        let finder = SymbolFinder::new(self.workspace_root.clone());
        let entry_location = finder.find_symbol(symbol_name)?;

        // Build graph incrementally
        let mut graph = TypeGraph::empty();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with entry symbol
        let entry_symbol = SymbolName(symbol_name.to_string());
        queue.push_back((entry_symbol.clone(), entry_location.clone(), 0));
        visited.insert(entry_symbol.clone());

        while let Some((symbol, location, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // Query definition
            let file_uri = format!("file://{}", location.file_path.display());
            let definitions = self
                .lsp_client
                .query_definition(&file_uri, location.line, location.character)
                .await?;

            // Add node for current symbol
            if let Some(def) = definitions.first() {
                let node = self.location_to_type_node(&symbol, def);
                graph.add_node(node);
            }

            // Query dependencies based on direction
            // For now, we'll use a simplified approach
            // In a real implementation, we'd query references/implementations
            match direction {
                TraversalDirection::Downstream => {
                    // Query what this symbol depends on (not implemented yet)
                    // Would need textDocument/references or custom parsing
                }
                TraversalDirection::Upstream => {
                    // Query who depends on this symbol (not implemented yet)
                    // Would need textDocument/references
                }
                TraversalDirection::Both => {
                    // Query both directions
                }
            }
        }

        Ok(graph)
    }

    fn location_to_type_node(&self, symbol: &SymbolName, location: &Location) -> TypeNode {
        // Parse URI to get file path
        let file_path = location
            .uri
            .strip_prefix("file://")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&location.uri));

        TypeNode {
            id: QualifiedSymbolName {
                module_path: file_path.clone(),
                symbol: symbol.clone(),
            },
            name: symbol.clone(),
            kind: SymbolKind::Struct, // Simplified - would need more LSP queries
            origin: SymbolOrigin::Canonical,
            location: SourceLocation {
                file: file_path,
                line: location.range.start.line,
                column: location.range.start.character,
            },
            language: self.language,
            generic_parameters: vec![],
        }
    }

    /// Shutdown LSP client
    pub async fn shutdown(mut self) -> Result<(), GraphBuilderError> {
        self.lsp_client.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Real tests would require LSP server running
    // Integration tests will cover the full flow

    #[test]
    fn test_lazy_builder_placeholder() {
        // Placeholder test - full integration test needed
        assert!(true);
    }
}
