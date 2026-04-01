use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::domain::language::LspServerConfig;

/// LSP client process manager
pub struct LspProcess {
    config: LspServerConfig,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

#[derive(Debug, thiserror::Error)]
pub enum LspProcessError {
    #[error("Failed to start LSP server: {0}")]
    StartFailed(std::io::Error),
    
    #[error("LSP server binary not found: {command}")]
    BinaryNotFound { command: String },
    
    #[error("LSP process not running")]
    NotRunning,
    
    #[error("Failed to write to LSP stdin: {0}")]
    WriteFailed(std::io::Error),
    
    #[error("Failed to read from LSP stdout: {0}")]
    ReadFailed(std::io::Error),
    
    #[error("LSP server returned unexpected output. This may indicate:\n\
             - The server binary is not an LSP server\n\
             - The server crashed on startup\n\
             - The server requires additional configuration\n\n\
             Run 'typeglass doctor' to check LSP installation.\n\
             Got: {output}")]
    UnexpectedOutput { output: String },
}

impl LspProcess {
    /// Create a new LSP process (not started yet)
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            config,
            child: None,
            stdin: None,
            stdout: None,
        }
    }

    /// Start the LSP server process
    pub async fn start(&mut self) -> Result<(), LspProcessError> {
        // Check if already running
        if self.child.is_some() {
            return Ok(());
        }

        // Start the process
        let mut command = Command::new(self.config.command);
        command
            .args(self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    LspProcessError::BinaryNotFound {
                        command: self.config.command.to_string(),
                    }
                } else {
                    LspProcessError::StartFailed(e)
                }
            })?;

        // Take ownership of stdin/stdout
        let stdin = child.stdin.take().ok_or_else(|| {
            LspProcessError::StartFailed(std::io::Error::other(
                "Failed to capture stdin",
            ))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            LspProcessError::StartFailed(std::io::Error::other(
                "Failed to capture stdout",
            ))
        })?;

        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));
        self.child = Some(child);

        Ok(())
    }

    /// Send a message to the LSP server
    pub async fn send_message(&mut self, message: &str) -> Result<(), LspProcessError> {
        let stdin = self.stdin.as_mut().ok_or(LspProcessError::NotRunning)?;

        // LSP uses Content-Length header
        let content = format!(
            "Content-Length: {}\r\n\r\n{}",
            message.len(),
            message
        );

        stdin
            .write_all(content.as_bytes())
            .await
            .map_err(LspProcessError::WriteFailed)?;

        stdin
            .flush()
            .await
            .map_err(LspProcessError::WriteFailed)?;

        Ok(())
    }

    /// Read a message from the LSP server
    pub async fn read_message(&mut self) -> Result<String, LspProcessError> {
        let stdout = self.stdout.as_mut().ok_or(LspProcessError::NotRunning)?;

        // Read Content-Length header
        let mut header = String::new();
        stdout
            .read_line(&mut header)
            .await
            .map_err(LspProcessError::ReadFailed)?;

        if !header.starts_with("Content-Length:") {
            return Err(LspProcessError::UnexpectedOutput { 
                output: header.trim().to_string() 
            });
        }

        let length: usize = header
            .trim()
            .strip_prefix("Content-Length:")
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| {
                LspProcessError::ReadFailed(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid Content-Length value",
                ))
            })?;

        // EDGE CASE: Protect against LSP message size bombs (100 MB limit)
        const MAX_LSP_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
        if length > MAX_LSP_MESSAGE_SIZE {
            return Err(LspProcessError::ReadFailed(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("LSP message too large: {} bytes (max 100 MB)", length),
            )));
        }

        // Read empty line
        let mut empty = String::new();
        stdout
            .read_line(&mut empty)
            .await
            .map_err(LspProcessError::ReadFailed)?;

        // Read content
        let mut buffer = vec![0u8; length];
        stdout
            .read_exact(&mut buffer)
            .await
            .map_err(LspProcessError::ReadFailed)?;

        String::from_utf8(buffer).map_err(|e| {
            LspProcessError::ReadFailed(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8: {}", e),
            ))
        })
    }

    /// Stop the LSP server
    pub async fn stop(&mut self) -> Result<(), LspProcessError> {
        if let Some(mut child) = self.child.take() {
            // Try graceful shutdown first
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.stdin = None;
        self.stdout = None;
        Ok(())
    }

    /// Check if process is running
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Best effort kill on drop
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::Language;

    #[tokio::test]
    async fn test_lsp_process_creation() {
        let config = LspServerConfig::for_language(Language::Rust).unwrap();
        let process = LspProcess::new(config);
        assert!(!process.is_running());
    }

    #[tokio::test]
    async fn test_lsp_process_start_with_invalid_binary() {
        let config = LspServerConfig {
            language: Language::Rust,
            name: "test",
            command: "this-binary-does-not-exist-12345",
            args: &[],
            install_instructions: "test",
        };
        
        let mut process = LspProcess::new(config);
        let result = process.start().await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LspProcessError::BinaryNotFound { .. }));
    }
}
