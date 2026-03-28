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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_initialize_params_new() {
        let params = LspInitializeParams::new("file:///tmp".to_string());
        assert_eq!(params.root_uri, Some("file:///tmp".to_string()));
        assert!(params.process_id.is_none());
    }

    #[test]
    fn test_lsp_initialize_params_with_process_id() {
        let params = LspInitializeParams::new("file:///tmp".to_string())
            .with_process_id(1234);
        assert_eq!(params.process_id, Some(1234));
    }

    #[test]
    fn test_client_capabilities_full() {
        let caps = ClientCapabilities::full();
        assert!(caps.text_document.is_some());
        assert!(caps.workspace.is_some());
    }

    #[test]
    fn test_client_capabilities_minimal() {
        let caps = ClientCapabilities::minimal();
        assert!(caps.text_document.is_some());
        assert!(caps.workspace.is_none());
    }

    #[test]
    fn test_text_document_capabilities_full() {
        let caps = TextDocumentCapabilities::full();
        assert!(caps.definition);
        assert!(caps.type_definition);
        assert!(caps.hover);
        assert!(caps.completion);
        assert!(caps.references);
    }

    #[test]
    fn test_text_document_capabilities_minimal() {
        let caps = TextDocumentCapabilities::minimal();
        assert!(caps.definition);
        assert!(!caps.type_definition);
        assert!(caps.hover);
        assert!(!caps.completion);
    }

    #[test]
    fn test_workspace_capabilities_full() {
        let caps = WorkspaceCapabilities::full();
        assert!(caps.workspace_folders);
        assert!(caps.configuration);
    }

    #[test]
    fn test_server_capabilities_to_lsp() {
        let caps = ServerCapabilities {
            text_document_sync: None,
            definition_provider: true,
            type_definition_provider: false,
            hover_provider: true,
            implementation_provider: false,
            references_provider: true,
            document_symbol_provider: false,
        };
        
        let lsp_caps = caps.to_lsp_capabilities();
        assert!(lsp_caps.supports_definition);
        assert!(lsp_caps.supports_hover);
        assert!(lsp_caps.supports_references);
        assert!(!lsp_caps.supports_document_symbols);
    }

    #[test]
    fn test_initialize_result() {
        let caps = ServerCapabilities {
            text_document_sync: None,
            definition_provider: true,
            type_definition_provider: true,
            hover_provider: true,
            implementation_provider: true,
            references_provider: true,
            document_symbol_provider: true,
        };
        
        let result = InitializeResult { 
            capabilities: caps.clone(),
            server_info: None,
        };
        assert_eq!(result.capabilities, caps);
    }

    #[test]
    fn test_server_info_builder() {
        let info = ServerInfo::new("rust-analyzer".to_string())
            .with_version("0.1.0".to_string());
        assert_eq!(info.name, "rust-analyzer");
        assert_eq!(info.version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_text_document_sync_kind_default() {
        let kind = TextDocumentSyncKind::default();
        assert_eq!(kind, TextDocumentSyncKind::Full);
    }
}
