use crate::domain::graph::EdgeKind;
use crate::infrastructure::lsp::init::{HoverContents, MarkupContent};

// Re-export the function under test from graph_builder
// Since it's private, we'll test it through the public API in integration tests
// For now, we'll duplicate the logic here for unit testing

/// Extract text content from HoverContents enum
fn extract_hover_text(hover_contents: &HoverContents) -> String {
    match hover_contents {
        HoverContents::Scalar(s) => s.clone(),
        HoverContents::Array(arr) => arr.join("\n"),
        HoverContents::Markup(markup) => markup.value.clone(),
    }
}

/// Infer EdgeKind from LSP hover response contents
fn infer_edge_kind_from_hover(hover_contents: &HoverContents, target_symbol: &str) -> EdgeKind {
    let text = extract_hover_text(hover_contents);
    let text_lower = text.to_lowercase();
    
    // Check for trait implementation
    if text_lower.contains("impl") && text_lower.contains(" for ") {
        return EdgeKind::Extends;
    }
    
    // Check for trait definition with supertraits
    if text_lower.contains("trait") && text_lower.contains(':') 
        && !text_lower.contains("impl") {
        return EdgeKind::Extends;
    }
    
    // Check for generic instantiation
    if (text_lower.contains("struct") || text_lower.contains("enum") || text_lower.contains("type")) 
        && text.contains('<') && text.contains('>') 
        && text.contains(target_symbol) {
        if let Some(angle_start) = text.find('<') {
            if let Some(angle_end) = text.find('>') {
                let generic_part = &text[angle_start+1..angle_end];
                if generic_part.contains(target_symbol) {
                    return EdgeKind::Instantiates;
                }
            }
        }
    }
    
    // Check for enum variant
    if text_lower.contains("enum") || text_lower.contains("variant") {
        return EdgeKind::Variant;
    }
    
    // Default: contains/uses relationship
    EdgeKind::Contains
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test extract_hover_text helper

    #[test]
    fn test_extract_hover_text_from_scalar() {
        let contents = HoverContents::Scalar("struct Foo".to_string());
        assert_eq!(extract_hover_text(&contents), "struct Foo");
    }

    #[test]
    fn test_extract_hover_text_from_array() {
        let contents = HoverContents::Array(vec![
            "```rust".to_string(),
            "struct Foo".to_string(),
            "```".to_string(),
        ]);
        assert_eq!(extract_hover_text(&contents), "```rust\nstruct Foo\n```");
    }

    #[test]
    fn test_extract_hover_text_from_markup() {
        let contents = HoverContents::Markup(MarkupContent {
            kind: "markdown".to_string(),
            value: "```rust\nstruct Foo\n```".to_string(),
        });
        assert_eq!(extract_hover_text(&contents), "```rust\nstruct Foo\n```");
    }

    // Test EdgeKind inference - Extends (trait implementation)

    #[test]
    fn test_infer_edgekind_impl_trait_for_type() {
        let hover = HoverContents::Scalar("impl Display for MyType".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Display");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_impl_trait_for_type_with_generics() {
        let hover = HoverContents::Scalar("impl<T> Iterator for MyIterator<T>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Iterator");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_impl_trait_for_type_multiline() {
        let hover = HoverContents::Array(vec![
            "```rust".to_string(),
            "impl From<String> for MyType".to_string(),
            "```".to_string(),
        ]);
        let edge_kind = infer_edge_kind_from_hover(&hover, "From");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    // Test EdgeKind inference - Extends (trait with supertrait)

    #[test]
    fn test_infer_edgekind_trait_extends_single() {
        let hover = HoverContents::Scalar("trait MyTrait: Display".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Display");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_trait_extends_multiple() {
        let hover = HoverContents::Scalar("trait MyTrait: Display + Debug".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Display");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_trait_extends_with_generics() {
        let hover = HoverContents::Scalar("trait MyTrait<T>: Iterator<Item = T>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Iterator");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    // Test EdgeKind inference - Instantiates (generic type parameter)

    #[test]
    fn test_infer_edgekind_struct_with_generic() {
        let hover = HoverContents::Scalar("struct Container<T>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "T");
        assert_eq!(edge_kind, EdgeKind::Instantiates);
    }

    #[test]
    fn test_infer_edgekind_struct_with_specific_type() {
        let hover = HoverContents::Scalar("struct Container<String>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "String");
        assert_eq!(edge_kind, EdgeKind::Instantiates);
    }

    #[test]
    fn test_infer_edgekind_enum_with_generic() {
        let hover = HoverContents::Scalar("enum Result<T, E>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "T");
        assert_eq!(edge_kind, EdgeKind::Instantiates);
    }

    #[test]
    fn test_infer_edgekind_type_alias_with_generic() {
        let hover = HoverContents::Scalar("type MyResult<T> = Result<T, MyError>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "T");
        assert_eq!(edge_kind, EdgeKind::Instantiates);
    }

    #[test]
    fn test_infer_edgekind_multiple_generics() {
        let hover = HoverContents::Scalar("struct HashMap<K, V>".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "K");
        assert_eq!(edge_kind, EdgeKind::Instantiates);
    }

    // Test EdgeKind inference - Variant (enum variants)

    #[test]
    fn test_infer_edgekind_enum_variant() {
        let hover = HoverContents::Scalar("enum MyEnum { Variant }".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Variant");
        assert_eq!(edge_kind, EdgeKind::Variant);
    }

    #[test]
    fn test_infer_edgekind_variant_keyword() {
        let hover = HoverContents::Scalar("Variant of Option".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Option");
        assert_eq!(edge_kind, EdgeKind::Variant);
    }

    // Test EdgeKind inference - Contains (default case)

    #[test]
    fn test_infer_edgekind_struct_field() {
        let hover = HoverContents::Scalar("struct MyStruct { field: String }".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "String");
        // String is not in generics, so it's a field type -> Contains
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_function_parameter() {
        let hover = HoverContents::Scalar("fn process(data: MyType)".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "MyType");
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_function_return() {
        let hover = HoverContents::Scalar("fn create() -> MyType".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "MyType");
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_method_call() {
        let hover = HoverContents::Scalar("pub fn new() -> Self".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "Self");
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_no_special_keywords() {
        let hover = HoverContents::Scalar("Some random text with MyType mentioned".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "MyType");
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    // Test edge cases

    #[test]
    fn test_infer_edgekind_empty_hover() {
        let hover = HoverContents::Scalar("".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "MyType");
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_case_insensitive() {
        // Test that detection is case-insensitive
        let hover = HoverContents::Scalar("IMPL MyTrait FOR MyType".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "MyTrait");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_complex_rust_code() {
        let hover = HoverContents::Markup(MarkupContent {
            kind: "markdown".to_string(),
            value: r#"```rust
impl<T: Display> From<T> for MyWrapper<T>
where
    T: Clone,
{
    fn from(value: T) -> Self {
        MyWrapper(value)
    }
}
```"#.to_string(),
        });
        let edge_kind = infer_edge_kind_from_hover(&hover, "From");
        assert_eq!(edge_kind, EdgeKind::Extends);
    }

    #[test]
    fn test_infer_edgekind_typescript_interface_extends() {
        // Should work for TypeScript too
        let hover = HoverContents::Scalar("interface MyInterface extends BaseInterface".to_string());
        // Note: This won't match our current Rust-specific patterns
        // But the "trait" heuristic with ":" might catch some cases
        let edge_kind = infer_edge_kind_from_hover(&hover, "BaseInterface");
        // Currently defaults to Contains, which is fine for MVP
        assert_eq!(edge_kind, EdgeKind::Contains);
    }

    #[test]
    fn test_infer_edgekind_go_struct_embed() {
        // Go struct embedding
        let hover = HoverContents::Scalar("type MyStruct struct { BaseStruct }".to_string());
        let edge_kind = infer_edge_kind_from_hover(&hover, "BaseStruct");
        // Currently defaults to Contains
        assert_eq!(edge_kind, EdgeKind::Contains);
    }
}
