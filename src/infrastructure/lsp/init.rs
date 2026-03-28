use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::infrastructure::lsp::client::{LspProcess, LspProcessError};
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

/// LSP-009 fix: Represent either a response or notification
#[derive(Debug)]
enum LspMessage {
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

impl LspMessage {
    /// Parse generic JSON into either Response or Notification
    fn parse(json: &str) -> Result<Self, LspClientError> {
        // First try to parse as a generic Value to check for "id" field
        let value: Value = serde_json::from_str(json)?;
        
        if let Some(obj) = value.as_object() {
            if obj.contains_key("id") {
                // It's a response
                let response: JsonRpcResponse = serde_json::from_value(value)?;
                Ok(LspMessage::Response(response))
            } else if obj.contains_key("method") {
                // It's a notification
                let notification: JsonRpcNotification = serde_json::from_value(value)?;
                Ok(LspMessage::Notification(notification))
            } else {
                Err(LspClientError::InvalidResponse(
                    "Message has neither 'id' nor 'method' field".to_string()
                ))
            }
        } else {
            Err(LspClientError::InvalidResponse(
                "Message is not a JSON object".to_string()
            ))
        }
    }
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

        // LSP-001 fix: Wait for LSP server to finish indexing
        // rust-analyzer and other servers need time after initialize
        self.wait_for_indexing().await?;

        Ok(response)
    }

    /// Wait for LSP server to finish indexing workspace (LSP-001)
    /// Polls with lightweight workspace/symbol requests until ready
    async fn wait_for_indexing(&mut self) -> Result<(), LspClientError> {
        use tokio::time::{sleep, Duration};

        eprintln!("DEBUG: Waiting for LSP to finish indexing...");

        // Poll up to 20 times with 1 second intervals (20 seconds total)
        for attempt in 0..20 {
            if attempt > 0 {
                sleep(Duration::from_secs(1)).await;
            }

            // Try workspace/symbol query with a simple query
            match self.send_request("workspace/symbol", serde_json::json!({"query": ""})).await {
                Ok(response) => {
                    // Check if we got a valid array response
                    if let Value::Array(arr) = &response {
                        eprintln!("DEBUG: Attempt {}: Got {} symbols", attempt, arr.len());
                        if !arr.is_empty() {
                            eprintln!("DEBUG: LSP ready!");
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("DEBUG: Attempt {}: Error: {:?}", attempt, e);
                }
            }
        }

        eprintln!("DEBUG: Timeout waiting for LSP, proceeding anyway");
        // Proceed anyway if polling fails - better than blocking forever
        Ok(())
    }

    /// Send a request and wait for response
    /// LSP-009 fix: Skip notifications and only return responses
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

        // Read messages until we get the response with matching id
        // Skip any notifications that arrive in between
        loop {
            let message_json = self.process.read_message().await?;
            
            match LspMessage::parse(&message_json)? {
                LspMessage::Response(response) => {
                    // Check if it's our response
                    if response.id == id {
                        // Check for errors
                        if let Some(error) = response.error {
                            return Err(LspClientError::InvalidResponse(format!(
                                "LSP error: {}",
                                error
                            )));
                        }

                        return response
                            .result
                            .ok_or_else(|| LspClientError::InvalidResponse("No result in response".to_string()));
                    } else {
                        // Response for different request, skip
                        eprintln!("DEBUG: Received response for different request (expected {}, got {})", id, response.id);
                        continue;
                    }
                }
                LspMessage::Notification(notification) => {
                    // Log and skip notifications
                    eprintln!("DEBUG: Received notification: {}", notification.method);
                    continue;
                }
            }
        }
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

    /// Query workspace symbols (LSP-002 fix)
    /// This is the proper way to find symbol definitions, not grep
    pub async fn workspace_symbols(
        &mut self,
        query: &str,
    ) -> Result<Vec<SymbolInformation>, LspClientError> {
        if !self.initialized {
            return Err(LspClientError::NotInitialized);
        }

        let params = serde_json::json!({
            "query": query
        });

        let response = self.send_request("workspace/symbol", params).await?;

        let symbols: Vec<SymbolInformation> = match response {
            Value::Array(arr) => serde_json::from_value(Value::Array(arr))?,
            Value::Null => vec![],
            _ => {
                return Err(LspClientError::InvalidResponse(
                    "Unexpected workspace/symbol response format".to_string(),
                ))
            }
        };

        Ok(symbols)
    }

    /// Find all references to a symbol at the given position
    /// Returns locations where this symbol is used/referenced
    pub async fn find_references(
        &mut self,
        file_uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
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
            },
            "context": {
                "includeDeclaration": include_declaration
            }
        });

        eprintln!("DEBUG: Sending textDocument/references request for {}:{}:{}", file_uri, line, character);

        let response = self.send_request("textDocument/references", params).await?;

        let locations: Vec<Location> = match response {
            Value::Array(arr) => serde_json::from_value(Value::Array(arr))?,
            Value::Null => vec![],
            _ => {
                return Err(LspClientError::InvalidResponse(
                    format!("Unexpected references response format: {:?}", response),
                ))
            }
        };

        eprintln!("DEBUG: Got {} reference locations", locations.len());
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

/// LSP SymbolInformation type (for workspace/symbol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: u32, // SymbolKind enum as number
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

/// LSP Hover response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResponse {
    pub contents: HoverContents,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

/// Hover contents can be various formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HoverContents {
    Scalar(String),
    Array(Vec<String>),
    Markup(MarkupContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: String, // "plaintext" or "markdown"
    pub value: String,
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

    // LSP-001: Test that we wait for indexing after initialize
    #[tokio::test]
    #[ignore] // Requires rust-analyzer to be installed
    async fn test_lsp_init_waits_for_indexing() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = LspServerConfig::for_language(Language::Rust)
            .expect("Rust LSP config should exist");

        let mut client = LspClient::new(config);
        
        // Initialize (which now includes waiting)
        let init_result = client.initialize(workspace.clone()).await;
        assert!(init_result.is_ok(), "Initialize should succeed: {:?}", init_result.err());

        // Try an immediate query - should not fail with "file not found"
        let test_file = workspace.join("src/lib.rs");
        if test_file.exists() {
            let file_uri = format!("file://{}", test_file.display());
            let result = client.query_definition(&file_uri, 0, 0).await;
            
            // Key improvement: rust-analyzer has had time to index
            match result {
                Ok(_) => {}, // Success is good
                Err(LspClientError::InvalidResponse(msg)) => {
                    // Should not be "file not found" anymore
                    assert!(!msg.contains("file not found"), 
                        "Should not get 'file not found' after waiting for indexing: {}", msg);
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }
}
