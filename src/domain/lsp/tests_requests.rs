#[cfg(test)]
mod lsp_requests_tests {
    use crate::domain::graph::{SourceLocation, SourceRange};
    use crate::domain::lsp::{
        DocumentIdentifier, DocumentSymbol, HoverResponse, MarkupContent,
        ReferencesResponse, TypeDefinitionResponse,
    };
    use crate::domain::lsp::LspRequestId;
    use std::path::PathBuf;

    fn make_request_id() -> LspRequestId {
        LspRequestId("test-123".to_string())
    }

    #[test]
    fn document_identifier_creates_without_version() {
        let doc = DocumentIdentifier::new("file:///test.ts".to_string());
        assert!(!doc.is_versioned());
        assert_eq!(doc.version, None);
    }

    #[test]
    fn document_identifier_creates_with_version() {
        let doc = DocumentIdentifier::with_version("file:///test.ts".to_string(), 5);
        assert!(doc.is_versioned());
        assert_eq!(doc.version, Some(5));
    }

    #[test]
    fn type_definition_response_empty() {
        let resp = TypeDefinitionResponse::empty(make_request_id());
        assert!(resp.is_empty());
        assert_eq!(resp.location_count(), 0);
    }

    #[test]
    fn type_definition_response_with_locations() {
        let resp = TypeDefinitionResponse {
            locations: vec![
                SourceLocation {
                    file: PathBuf::from("a.ts"),
                    line: 1,
                    column: 1,
                },
                SourceLocation {
                    file: PathBuf::from("b.ts"),
                    line: 2,
                    column: 1,
                },
            ],
            request_id: make_request_id(),
            response_time_ms: 100,
        };
        assert!(!resp.is_empty());
        assert_eq!(resp.location_count(), 2);
    }

    #[test]
    fn references_response_counts() {
        let resp = ReferencesResponse {
            locations: vec![
                SourceLocation {
                    file: PathBuf::from("a.ts"),
                    line: 1,
                    column: 1,
                },
            ],
            include_declaration: true,
            request_id: make_request_id(),
            response_time_ms: 50,
        };
        assert_eq!(resp.reference_count(), 1);
    }

    #[test]
    fn document_symbol_hierarchy() {
        let child = DocumentSymbol::new(
            "method".to_string(),
            crate::domain::graph::SymbolKind::Struct,
            SourceRange::single_line(PathBuf::from("test.ts"), 2, 1, 10).unwrap(),
        );

        let parent = DocumentSymbol::new(
            "class".to_string(),
            crate::domain::graph::SymbolKind::Struct,
            SourceRange::single_line(PathBuf::from("test.ts"), 1, 1, 10).unwrap(),
        )
        .with_children(vec![child]);

        assert!(parent.has_children());
        assert_eq!(parent.child_count(), 1);
        assert_eq!(parent.total_symbol_count(), 2); // parent + child
    }

    #[test]
    fn markup_content_types() {
        let plain = MarkupContent::plain_text("text".to_string());
        assert!(!plain.is_markdown());

        let md = MarkupContent::markdown("# Header".to_string());
        assert!(md.is_markdown());
    }

    #[test]
    fn hover_response_with_range() {
        let content = MarkupContent::plain_text("info".to_string());
        let range = SourceRange::single_line(PathBuf::from("test.ts"), 1, 1, 5).unwrap();

        let resp = HoverResponse::new(content, make_request_id()).with_range(range);

        assert!(resp.has_range());
    }

    #[test]
    fn hover_response_without_range() {
        let content = MarkupContent::plain_text("info".to_string());
        let resp = HoverResponse::new(content, make_request_id());
        assert!(!resp.has_range());
    }
}
