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
