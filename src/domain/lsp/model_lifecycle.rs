use serde::{Deserialize, Serialize};

use super::capabilities::LspCapabilities;

/// LSP initialize parameters (sent by client)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspInitializeParams {
    /// Process ID of the client
    pub process_id: Option<u32>,
    /// Root URI of the workspace
    pub root_uri: Option<String>,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
}

impl LspInitializeParams {
    pub fn new(root_uri: String) -> Self {
        Self {
            process_id: None,
            root_uri: Some(root_uri),
            capabilities: ClientCapabilities::default(),
        }
    }

    pub fn with_process_id(mut self, pid: u32) -> Self {
        self.process_id = Some(pid);
        self
    }
}

/// Client capabilities (what the client supports)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub workspace: Option<WorkspaceCapabilities>,
    pub text_document: Option<TextDocumentCapabilities>,
}

impl ClientCapabilities {
    pub fn full() -> Self {
        Self {
            workspace: Some(WorkspaceCapabilities::full()),
            text_document: Some(TextDocumentCapabilities::full()),
        }
    }

    pub fn minimal() -> Self {
        Self {
            workspace: None,
            text_document: Some(TextDocumentCapabilities::minimal()),
        }
    }
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self::full()
    }
}

/// Workspace-level client capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCapabilities {
    pub apply_edit: bool,
    pub workspace_folders: bool,
    pub configuration: bool,
}

impl WorkspaceCapabilities {
    pub fn full() -> Self {
        Self {
            apply_edit: true,
            workspace_folders: true,
            configuration: true,
        }
    }
}

/// Text document client capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDocumentCapabilities {
    pub synchronization: Option<SynchronizationCapabilities>,
    pub completion: bool,
    pub hover: bool,
    pub definition: bool,
    pub type_definition: bool,
    pub implementation: bool,
    pub references: bool,
    pub document_symbol: bool,
}

impl TextDocumentCapabilities {
    pub fn full() -> Self {
        Self {
            synchronization: Some(SynchronizationCapabilities::full()),
            completion: true,
            hover: true,
            definition: true,
            type_definition: true,
            implementation: true,
            references: true,
            document_symbol: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
            synchronization: Some(SynchronizationCapabilities::minimal()),
            completion: false,
            hover: true,
            definition: true,
            type_definition: false,
            implementation: false,
            references: true,
            document_symbol: false,
        }
    }
}

/// Document synchronization capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynchronizationCapabilities {
    pub did_open: bool,
    pub did_change: bool,
    pub did_save: bool,
    pub did_close: bool,
}

impl SynchronizationCapabilities {
    pub fn full() -> Self {
        Self {
            did_open: true,
            did_change: true,
            did_save: true,
            did_close: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
            did_open: true,
            did_change: false,
            did_save: false,
            did_close: true,
        }
    }
}

/// Server capabilities (what the server supports)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub text_document_sync: Option<TextDocumentSyncKind>,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub type_definition_provider: bool,
    pub implementation_provider: bool,
    pub references_provider: bool,
    pub document_symbol_provider: bool,
}

impl ServerCapabilities {
    pub fn to_lsp_capabilities(&self) -> LspCapabilities {
        LspCapabilities {
            supports_document_symbols: self.document_symbol_provider,
            supports_references: self.references_provider,
            supports_definition: self.definition_provider,
            supports_hover: self.hover_provider,
        }
    }

    pub fn supports_feature(&self, feature: LspFeature) -> bool {
        match feature {
            LspFeature::Hover => self.hover_provider,
            LspFeature::Definition => self.definition_provider,
            LspFeature::TypeDefinition => self.type_definition_provider,
            LspFeature::Implementation => self.implementation_provider,
            LspFeature::References => self.references_provider,
            LspFeature::DocumentSymbol => self.document_symbol_provider,
        }
    }
}

/// LSP feature enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LspFeature {
    Hover,
    Definition,
    TypeDefinition,
    Implementation,
    References,
    DocumentSymbol,
}

/// Text document synchronization kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDocumentSyncKind {
    /// Documents should not be synced
    None,
    /// Documents are synced by always sending full content
    Full,
    /// Documents are synced by sending incremental updates
    Incremental,
}

impl Default for TextDocumentSyncKind {
    fn default() -> Self {
        Self::Full
    }
}

/// Initialize result from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: Option<ServerInfo>,
}

/// Server information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

impl ServerInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}
