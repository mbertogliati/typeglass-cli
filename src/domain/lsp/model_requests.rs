use serde::{Deserialize, Serialize};

use super::primitives::LspRequestId;
use crate::domain::graph::{Position, SourceLocation, SourceRange, SymbolKind};

/// Document identifier for LSP requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentIdentifier {
    /// File URI (e.g., "file:///path/to/file.ts")
    pub uri: String,
    /// Optional version number for versioned documents
    pub version: Option<u32>,
}

impl DocumentIdentifier {
    pub fn new(uri: String) -> Self {
        Self { uri, version: None }
    }

    pub fn with_version(uri: String, version: u32) -> Self {
        Self {
            uri,
            version: Some(version),
        }
    }

    pub fn is_versioned(&self) -> bool {
        self.version.is_some()
    }
}

// ============================================================================
// Type Definition
// ============================================================================

/// Request to get the type definition of a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDefinitionRequest {
    pub text_document: DocumentIdentifier,
    pub position: Position,
    pub request_id: LspRequestId,
}

/// Response with type definition locations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDefinitionResponse {
    pub locations: Vec<SourceLocation>,
    pub request_id: LspRequestId,
    pub response_time_ms: u64,
}

impl TypeDefinitionResponse {
    pub fn empty(request_id: LspRequestId) -> Self {
        Self {
            locations: Vec::new(),
            request_id,
            response_time_ms: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn location_count(&self) -> usize {
        self.locations.len()
    }
}

// ============================================================================
// Implementation
// ============================================================================

/// Request to get implementations of a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationRequest {
    pub text_document: DocumentIdentifier,
    pub position: Position,
    pub request_id: LspRequestId,
}

/// Response with implementation locations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationResponse {
    pub locations: Vec<SourceLocation>,
    pub request_id: LspRequestId,
    pub response_time_ms: u64,
}

impl ImplementationResponse {
    pub fn empty(request_id: LspRequestId) -> Self {
        Self {
            locations: Vec::new(),
            request_id,
            response_time_ms: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

// ============================================================================
// References
// ============================================================================

/// Request to find all references to a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencesRequest {
    pub text_document: DocumentIdentifier,
    pub position: Position,
    pub include_declaration: bool,
    pub request_id: LspRequestId,
}

/// Response with all reference locations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencesResponse {
    pub locations: Vec<SourceLocation>,
    pub include_declaration: bool,
    pub request_id: LspRequestId,
    pub response_time_ms: u64,
}

impl ReferencesResponse {
    pub fn empty(request_id: LspRequestId, include_declaration: bool) -> Self {
        Self {
            locations: Vec::new(),
            include_declaration,
            request_id,
            response_time_ms: 0,
        }
    }

    pub fn reference_count(&self) -> usize {
        self.locations.len()
    }
}

// ============================================================================
// Document Symbol
// ============================================================================

/// Request to get all symbols in a document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbolRequest {
    pub text_document: DocumentIdentifier,
    pub request_id: LspRequestId,
}

/// A symbol in a document (hierarchical structure)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbol {
    /// Name of the symbol
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Full range of the symbol
    pub range: SourceRange,
    /// Range to select when jumping to this symbol
    pub selection_range: SourceRange,
    /// Child symbols (nested)
    pub children: Vec<DocumentSymbol>,
}

impl DocumentSymbol {
    pub fn new(name: String, kind: SymbolKind, range: SourceRange) -> Self {
        Self {
            name,
            kind,
            selection_range: range.clone(),
            range,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<DocumentSymbol>) -> Self {
        self.children = children;
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn total_symbol_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_symbol_count()).sum::<usize>()
    }
}

/// Response with document symbols
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbolResponse {
    pub symbols: Vec<DocumentSymbol>,
    pub request_id: LspRequestId,
    pub response_time_ms: u64,
}

impl DocumentSymbolResponse {
    pub fn empty(request_id: LspRequestId) -> Self {
        Self {
            symbols: Vec::new(),
            request_id,
            response_time_ms: 0,
        }
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn total_symbol_count(&self) -> usize {
        self.symbols.iter().map(|s| s.total_symbol_count()).sum()
    }
}

// ============================================================================
// Hover
// ============================================================================

/// Request to get hover information at a position
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverRequest {
    pub text_document: DocumentIdentifier,
    pub position: Position,
    pub request_id: LspRequestId,
}

/// Markup content kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

/// Markup content with kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

impl MarkupContent {
    pub fn plain_text(value: String) -> Self {
        Self {
            kind: MarkupKind::PlainText,
            value,
        }
    }

    pub fn markdown(value: String) -> Self {
        Self {
            kind: MarkupKind::Markdown,
            value,
        }
    }

    pub fn is_markdown(&self) -> bool {
        matches!(self.kind, MarkupKind::Markdown)
    }
}

/// Response with hover information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverResponse {
    pub contents: MarkupContent,
    pub range: Option<SourceRange>,
    pub request_id: LspRequestId,
    pub response_time_ms: u64,
}

impl HoverResponse {
    pub fn new(contents: MarkupContent, request_id: LspRequestId) -> Self {
        Self {
            contents,
            range: None,
            request_id,
            response_time_ms: 0,
        }
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn has_range(&self) -> bool {
        self.range.is_some()
    }
}
