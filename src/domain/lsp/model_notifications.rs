use serde::{Deserialize, Serialize};

use super::requests::DocumentIdentifier;
use crate::domain::graph::SourceRange;

/// LSP notification (sent from client to server, no response expected)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LspNotification {
    /// Document was opened
    DidOpen { document: TextDocument },
    /// Document content changed
    DidChange {
        document: DocumentIdentifier,
        changes: Vec<TextDocumentContentChange>,
    },
    /// Document was saved
    DidSave { document: DocumentIdentifier },
    /// Document was closed
    DidClose { document: DocumentIdentifier },
}

impl LspNotification {
    pub fn document_uri(&self) -> &str {
        match self {
            LspNotification::DidOpen { document } => &document.uri,
            LspNotification::DidChange { document, .. } => &document.uri,
            LspNotification::DidSave { document } => &document.uri,
            LspNotification::DidClose { document } => &document.uri,
        }
    }

    pub fn is_content_change(&self) -> bool {
        matches!(
            self,
            LspNotification::DidOpen { .. } | LspNotification::DidChange { .. }
        )
    }

    pub fn is_open(&self) -> bool {
        matches!(self, LspNotification::DidOpen { .. })
    }

    pub fn is_close(&self) -> bool {
        matches!(self, LspNotification::DidClose { .. })
    }
}

/// Full text document (for didOpen)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDocument {
    /// Document URI
    pub uri: String,
    /// Language identifier (e.g., "typescript", "rust")
    pub language_id: String,
    /// Document version
    pub version: u32,
    /// Full document text
    pub text: String,
}

impl TextDocument {
    pub fn new(uri: String, language_id: String, version: u32, text: String) -> Self {
        Self {
            uri,
            language_id,
            version,
            text,
        }
    }

    pub fn line_count(&self) -> usize {
        self.text.lines().count()
    }

    pub fn char_count(&self) -> usize {
        self.text.len()
    }
}

/// Change to document content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDocumentContentChange {
    /// Range being replaced (None = full document)
    pub range: Option<SourceRange>,
    /// New text
    pub text: String,
}

impl TextDocumentContentChange {
    pub fn full_document(text: String) -> Self {
        Self { range: None, text }
    }

    pub fn incremental(range: SourceRange, text: String) -> Self {
        Self {
            range: Some(range),
            text,
        }
    }

    pub fn is_full_document(&self) -> bool {
        self.range.is_none()
    }

    pub fn is_incremental(&self) -> bool {
        self.range.is_some()
    }
}

/// Batch of notifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationBatch {
    pub notifications: Vec<LspNotification>,
}

impl NotificationBatch {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn add(&mut self, notification: LspNotification) {
        self.notifications.push(notification);
    }

    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    pub fn count(&self) -> usize {
        self.notifications.len()
    }
}

impl Default for NotificationBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lsp_notification_document_uri() {
        let doc = TextDocument::new(
            "file:///test.rs".to_string(),
            "rust".to_string(),
            1,
            "content".to_string(),
        );
        let notif = LspNotification::DidOpen { document: doc };
        assert_eq!(notif.document_uri(), "file:///test.rs");
    }

    #[test]
    fn test_lsp_notification_is_content_change() {
        let doc = TextDocument::new("file:///test.rs".into(), "rust".into(), 1, "".into());
        assert!(LspNotification::DidOpen { document: doc.clone() }.is_content_change());
        
        let doc_id = DocumentIdentifier { uri: "file:///test.rs".into(), version: Some(2) };
        assert!(LspNotification::DidChange { document: doc_id.clone(), changes: vec![] }.is_content_change());
        assert!(!LspNotification::DidSave { document: doc_id.clone() }.is_content_change());
    }

    #[test]
    fn test_lsp_notification_is_open_close() {
        let doc = TextDocument::new("file:///test.rs".into(), "rust".into(), 1, "".into());
        let open = LspNotification::DidOpen { document: doc };
        assert!(open.is_open());
        assert!(!open.is_close());
        
        let doc_id = DocumentIdentifier { uri: "file:///test.rs".into(), version: None };
        let close = LspNotification::DidClose { document: doc_id };
        assert!(!close.is_open());
        assert!(close.is_close());
    }

    #[test]
    fn test_text_document_line_count() {
        let doc = TextDocument::new(
            "file:///test".into(),
            "rust".into(),
            1,
            "line1\nline2\nline3".into(),
        );
        assert_eq!(doc.line_count(), 3);
    }

    #[test]
    fn test_text_document_char_count() {
        let doc = TextDocument::new(
            "file:///test".into(),
            "rust".into(),
            1,
            "hello world".into(),
        );
        assert_eq!(doc.char_count(), 11);
    }

    #[test]
    fn test_text_document_content_change_full() {
        let change = TextDocumentContentChange::full_document("new text".into());
        assert!(change.is_full_document());
        assert!(!change.is_incremental());
    }

    #[test]
    fn test_text_document_content_change_incremental() {
        let range = SourceRange {
            file: PathBuf::from("test.rs"),
            start: crate::domain::graph::Position { line: 0, column: 0 },
            end: crate::domain::graph::Position { line: 0, column: 5 },
        };
        let change = TextDocumentContentChange::incremental(range, "text".into());
        assert!(!change.is_full_document());
        assert!(change.is_incremental());
    }

    #[test]
    fn test_notification_batch_operations() {
        let mut batch = NotificationBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.count(), 0);
        
        let doc = TextDocument::new("file:///test".into(), "rust".into(), 1, "".into());
        batch.add(LspNotification::DidOpen { document: doc });
        
        assert!(!batch.is_empty());
        assert_eq!(batch.count(), 1);
        assert_eq!(batch.notifications.len(), 1);
    }
}
