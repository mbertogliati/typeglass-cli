use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::lsp_client::{LspProcess, LspProcessError};
use crate::domain::language::LspServerConfig;

/// LSP client with initialization and request/response handling
pub struct LspClient {
    process: LspProcess,
    next_id: AtomicU64,
    initialized: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LspClientError {
    #[error("LSP process error: {0}")]
    ProcessError(#[from] LspProcessError),
    
    #[error("Failed to serialize request: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("LSP not initialized")]
    NotInitialized,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeParams {
    #[serde(rename = "rootUri")]
    root_uri: String,
    capabilities: ClientCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientCapabilities {
    #[serde(rename = "textDocument")]
    text_document: Option<TextDocumentClientCapabilities>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextDocumentClientCapabilities {
    definition: Option<DefinitionCapability>,
    references: Option<ReferencesCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DefinitionCapability {
    #[serde(rename = "linkSupport")]
    link_support: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReferencesCapability {
    #[serde(rename = "dynamicRegistration")]
    dynamic_registration: bool,
}

impl LspClient {
    /// Create a new LSP client
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            process: LspProcess::new(config),
            next_id: AtomicU64::new(1),
            initialized: false,
        }
    }

    /// Start and initialize the LSP server
    pub async fn initialize(&mut self, workspace_root: PathBuf) -> Result<Value, LspClientError> {
        // Start the process
        self.process.start().await?;

        // Send initialize request
        let root_uri = format!("file://{}", workspace_root.display());
        let params = InitializeParams {
            root_uri,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    definition: Some(DefinitionCapability {
                        link_support: false,
                    }),
                    references: Some(ReferencesCapability {
                        dynamic_registration: false,
                    }),
                }),
            },
        };

        let response = self
            .send_request("initialize", serde_json::to_value(params)?)
            .await?;

        // Send initialized notification
        self.send_notification("initialized", Value::Object(serde_json::Map::new()))
            .await?;

        self.initialized = true;

        Ok(response)
    }

    /// Send a request and wait for response
    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value, LspClientError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        self.process.send_message(&request_json).await?;

        // Read response
        let response_json = self.process.read_message().await?;
        let response: JsonRpcResponse = serde_json::from_str(&response_json)?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(LspClientError::InvalidResponse(format!(
                "LSP error: {}",
                error
            )));
        }

        response
            .result
            .ok_or_else(|| LspClientError::InvalidResponse("No result in response".to_string()))
    }

    /// Send a notification (no response expected)
    async fn send_notification(&mut self, method: &str, params: Value) -> Result<(), LspClientError> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };

        let notification_json = serde_json::to_string(&notification)?;
        self.process.send_message(&notification_json).await?;

        Ok(())
    }

    /// Shutdown the LSP server
    pub async fn shutdown(&mut self) -> Result<(), LspClientError> {
        if !self.initialized {
            return Ok(());
        }

        // Send shutdown request
        let _ = self
            .send_request("shutdown", Value::Null)
            .await;

        // Send exit notification
        let _ = self
            .send_notification("exit", Value::Null)
            .await;

        // Stop the process
        self.process.stop().await?;
        self.initialized = false;

        Ok(())
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Query type definition at position
    pub async fn query_definition(
        &mut self,
        file_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<Location>, LspClientError> {
        if !self.initialized {
            return Err(LspClientError::NotInitialized);
        }

        let params = serde_json::json!({
            "textDocument": {
                "uri": file_uri
            },
            "position": {
                "line": line,
                "character": character
            }
        });

        let response = self.send_request("textDocument/definition", params).await?;

        // Response can be Location, Location[], or LocationLink[]
        // We'll handle Location[] for now
        let locations: Vec<Location> = match response {
            Value::Array(arr) => {
                serde_json::from_value(Value::Array(arr))?
            }
            Value::Object(_) => {
                // Single location
                vec![serde_json::from_value(response)?]
            }
            Value::Null => {
                // Not found
                vec![]
            }
            _ => {
                return Err(LspClientError::InvalidResponse(
                    "Unexpected definition response format".to_string(),
                ))
            }
        };

        Ok(locations)
    }
}

/// LSP Location type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// LSP Range type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP Position type  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Best effort shutdown
        // Can't use async in Drop, process Drop will kill it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::Language;

    #[tokio::test]
    async fn test_lsp_client_creation() {
        let config = LspServerConfig::for_language(Language::Rust).unwrap();
        let client = LspClient::new(config);
        assert!(!client.is_initialized());
    }

    #[tokio::test]
    async fn test_lsp_client_with_invalid_binary() {
        let config = LspServerConfig {
            language: Language::Rust,
            name: "test",
            command: "nonexistent-binary-12345",
            args: &[],
            install_instructions: "test",
        };

        let mut client = LspClient::new(config);
        let result = client.initialize(PathBuf::from("/tmp")).await;

        assert!(result.is_err());
    }

    // Note: Real LSP initialization tests require LSP binary installed
    // Integration tests will cover the full flow with rust-analyzer
}
