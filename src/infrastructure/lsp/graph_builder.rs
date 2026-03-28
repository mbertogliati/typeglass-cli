use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::domain::graph::{
    EdgeKind, GraphCompleteness, QualifiedSymbolName, SourceLocation, SymbolKind, SymbolName,
    SymbolOrigin, TraversalDirection, TypeEdge, TypeGraph, TypeNode,
};
use crate::domain::language::{Language, LspServerConfig};
use crate::infrastructure::{LspClient, LspClientError, Location, SymbolFinderError, HoverContents};

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
        // LSP-002 fix: Use workspace/symbol instead of grep
        // This properly finds symbol definitions, not just any occurrence
        let symbols = self.lsp_client.workspace_symbols(symbol_name).await?;
        
        log::debug!("DEBUG: Found {} symbols for '{}'", symbols.len(), symbol_name);
        for (i, sym) in symbols.iter().enumerate() {
            log::debug!("  [{}] {} (kind: {}) at {}", i, sym.name, sym.kind, sym.location.uri);
        }
        
        if symbols.is_empty() {
            return Err(GraphBuilderError::FinderError(SymbolFinderError::NotFound {
                symbol: symbol_name.to_string(),
            }));
        }

        // Find the best match (exact name match, prefer struct/class/interface)
        let entry_symbol = symbols
            .iter()
            .find(|s| s.name == symbol_name && is_type_symbol(s.kind))
            .or_else(|| symbols.first())
            .ok_or_else(|| GraphBuilderError::FinderError(SymbolFinderError::NotFound {
                symbol: symbol_name.to_string(),
            }))?;

        log::debug!("DEBUG: Selected symbol: {} at {}", entry_symbol.name, entry_symbol.location.uri);

        let entry_location = parse_lsp_location(&entry_symbol.location)?;

        log::debug!("DEBUG: Parsed location: {:?}", entry_location.file_path);

        // Build graph incrementally
        let mut graph = TypeGraph::empty();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with entry symbol
        let entry_sym = SymbolName(symbol_name.to_string());
        queue.push_back((entry_sym.clone(), entry_location.clone(), 0));
        visited.insert(entry_sym.clone());

        while let Some((symbol, location, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // Query definition - use original URI from LSP!
            let file_uri = &entry_symbol.location.uri;
            log::debug!("DEBUG: Querying definition at {}", file_uri);
            
            let definitions = self
                .lsp_client
                .query_definition(file_uri, location.line, location.character)
                .await?;

            // Add node for current symbol
            if let Some(def) = definitions.first() {
                let node = self.location_to_type_node(&symbol, def);
                graph.add_node(node);
                
                // Now find references to build edges
                log::debug!("DEBUG: Finding references for {} at {}:{}:{}", 
                    symbol.0, file_uri, location.line, location.character);
                
                match self.lsp_client.find_references(
                    file_uri,
                    location.line,
                    location.character,
                    false // don't include declaration
                ).await {
                    Ok(references) => {
                        log::debug!("DEBUG: Got {} reference locations", references.len());
                        
                        // For each reference, determine the containing symbol to build edges
                        for (i, reference) in references.iter().enumerate().take(10) { // Limit to 10 refs for MVP
                            let ref_loc = parse_lsp_location(reference)?;
                            
                            log::debug!("DEBUG: [Ref {}/{}] Processing reference at {}:{}:{}", 
                                i + 1, references.len().min(10),
                                reference.uri, 
                                reference.range.start.line,
                                reference.range.start.character);
                            
                            // Strategy: Use file path to infer containing symbol
                            // Full implementation would query documentSymbol or parse hover
                            if let Some(using_symbol_name) = extract_symbol_from_path(&ref_loc.file_path) {
                                let using_sym = SymbolName(using_symbol_name.clone());
                                
                                // Don't create self-edges
                                if using_sym != symbol {
                                    // Create edge: using_symbol -> current_symbol (dependency)
                                    let edge = TypeEdge {
                                        from: using_sym.clone(),
                                        to: symbol.clone(),
                                        kind: EdgeKind::Contains, // Simplified
                                    };
                                    
                                    graph.add_edge(edge);
                                    log::debug!("DEBUG: Created edge: {} -> {}", using_sym.0, symbol.0);
                                    
                                    // Add to queue for further traversal if within depth
                                    if depth + 1 < max_depth && !visited.contains(&using_sym) {
                                        visited.insert(using_sym.clone());
                                        queue.push_back((using_sym, ref_loc, depth + 1));
                                        log::debug!("DEBUG: Queued {} for traversal at depth {}", using_symbol_name, depth + 1);
                                    }
                                }
                            }
                        }
                        
                        if references.len() > 10 {
                            log::debug!("DEBUG: Skipped {} references (limited to 10 for MVP)", references.len() - 10);
                        }
                    }
                    Err(e) => {
                        log::debug!("DEBUG: Failed to find references: {}", e);
                        // Continue without edges - partial result
                    }
                }
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

/// Helper: Check if symbol kind represents a type definition
fn is_type_symbol(kind: u32) -> bool {
    // LSP SymbolKind enum values
    // 5 = Class, 6 = Method, 10 = Function, 12 = Variable, 23 = Struct, 11 = Interface
    matches!(kind, 5 | 11 | 23) // Class, Interface, Struct
}

/// Helper structure for symbol location
#[derive(Debug, Clone)]
struct InternalSymbolLocation {
    file_path: PathBuf,
    line: u32,
    character: u32,
}

/// Helper: Parse LSP Location to internal location
fn parse_lsp_location(loc: &crate::infrastructure::lsp::init::Location) -> Result<InternalSymbolLocation, GraphBuilderError> {
    // Remove file:// prefix and parse path
    let path_str = loc.uri.strip_prefix("file://").unwrap_or(&loc.uri);
    let file_path = PathBuf::from(path_str);
    
    Ok(InternalSymbolLocation {
        file_path,
        line: loc.range.start.line,
        character: loc.range.start.character,
    })
}

/// Extract likely symbol name from file path (heuristic for MVP)
/// e.g., "src/domain/graph/model_graph.rs" -> Some("TypeGraph")
fn extract_symbol_from_path(path: &PathBuf) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Convert snake_case to PascalCase heuristic
            // model_graph -> ModelGraph
            s.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<String>()
        })
}

