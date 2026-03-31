use std::path::PathBuf;

use super::types::Language;

/// Configuration for a specific LSP server
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspServerConfig {
    /// Language this server supports
    pub language: Language,
    /// Name of the LSP server
    pub name: &'static str,
    /// Command/binary name to execute
    pub command: &'static str,
    /// Arguments to pass when starting the server
    pub args: &'static [&'static str],
    /// Installation instructions URL or command
    pub install_instructions: &'static str,
}

/// LSP server installation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspInstallStatus {
    /// Server is installed and ready to use
    Installed { path: PathBuf },
    /// Server binary not found in PATH
    NotFound,
    /// Server binary found but not executable
    NotExecutable { path: PathBuf },
}

impl LspServerConfig {
    /// Get LSP configuration for a language
    pub fn for_language(language: Language) -> Option<Self> {
        match language {
            Language::TypeScript => Some(Self {
                language: Language::TypeScript,
                name: "typescript-language-server",
                command: "typescript-language-server",
                args: &["--stdio"],
                install_instructions: 
                    "npm install -g typescript-language-server typescript\n\
                     Or: https://github.com/typescript-language-server/typescript-language-server",
            }),
            Language::Rust => Some(Self {
                language: Language::Rust,
                name: "rust-analyzer",
                command: "rust-analyzer",
                args: &[],
                install_instructions:
                    "rustup component add rust-analyzer\n\
                     Or: https://rust-analyzer.github.io/manual.html#installation",
            }),
            Language::Go => Some(Self {
                language: Language::Go,
                name: "gopls",
                command: "gopls",
                args: &[],
                install_instructions:
                    "go install golang.org/x/tools/gopls@latest\n\
                     Or: https://github.com/golang/tools/tree/master/gopls",
            }),
        }
    }

    /// Check if this LSP server is installed
    pub fn check_installation(&self) -> LspInstallStatus {
        // Check if command exists in PATH
        match which::which(self.command) {
            Ok(path) => {
                // Verify it's executable
                if is_executable(&path) {
                    LspInstallStatus::Installed { path }
                } else {
                    LspInstallStatus::NotExecutable { path }
                }
            }
            Err(_) => LspInstallStatus::NotFound,
        }
    }
}

#[cfg(unix)]
fn is_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(_path: &PathBuf) -> bool {
    // On Windows, if it exists, we assume it's executable
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_config() {
        let config = LspServerConfig::for_language(Language::TypeScript).unwrap();
        assert_eq!(config.name, "typescript-language-server");
        assert_eq!(config.command, "typescript-language-server");
        assert!(config.install_instructions.contains("npm install"));
    }

    #[test]
    fn test_rust_config() {
        let config = LspServerConfig::for_language(Language::Rust).unwrap();
        assert_eq!(config.name, "rust-analyzer");
        assert_eq!(config.command, "rust-analyzer");
        assert!(config.install_instructions.contains("rustup"));
    }

    #[test]
    fn test_go_config() {
        let config = LspServerConfig::for_language(Language::Go).unwrap();
        assert_eq!(config.name, "gopls");
        assert_eq!(config.command, "gopls");
        assert!(config.install_instructions.contains("go install"));
    }

    #[test]
    fn test_check_installation_for_existing_binary() {
        // This test will vary by environment, but we can test the mechanism
        let config = LspServerConfig {
            language: Language::Rust,
            name: "test",
            command: "ls", // 'ls' should exist on Unix, 'cmd' on Windows
            args: &[],
            install_instructions: "test",
        };
        
        let status = config.check_installation();
        // Should be either Installed or NotFound, never NotExecutable for 'ls'
        match status {
            LspInstallStatus::Installed { .. } => {},
            LspInstallStatus::NotFound => {},
            LspInstallStatus::NotExecutable { .. } => panic!("ls should be executable"),
        }
    }

    #[test]
    fn test_check_installation_for_nonexistent() {
        let config = LspServerConfig {
            language: Language::Rust,
            name: "test",
            command: "this-command-definitely-does-not-exist-12345",
            args: &[],
            install_instructions: "test",
        };
        
        let status = config.check_installation();
        assert!(matches!(status, LspInstallStatus::NotFound));
    }
}
