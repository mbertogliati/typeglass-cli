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
            Language::Java => Some(Self {
                language: Language::Java,
                name: "jdtls",
                command: "jdtls",
                args: &[],
                install_instructions:
                    "Download from: https://download.eclipse.org/jdtls/snapshots/\n\
                     Or use mason.nvim, or install via package manager:\n\
                     - macOS: brew install jdtls\n\
                     - Arch: yay -S jdtls\n\
                     Documentation: https://github.com/eclipse-jdtls/eclipse.jdt.ls",
            }),
            Language::Kotlin => Some(Self {
                language: Language::Kotlin,
                name: "kotlin-language-server",
                command: "kotlin-language-server",
                args: &[],
                install_instructions:
                    "Download from: https://github.com/fwcd/kotlin-language-server/releases\n\
                     Or build from source:\n\
                     git clone https://github.com/fwcd/kotlin-language-server\n\
                     cd kotlin-language-server\n\
                     ./gradlew :server:installDist\n\
                     Add server/build/install/server/bin to PATH",
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
    fn test_java_config() {
        let config = LspServerConfig::for_language(Language::Java).unwrap();
        assert_eq!(config.language, Language::Java);
        assert_eq!(config.name, "jdtls");
        assert_eq!(config.command, "jdtls");
        assert!(
            config.install_instructions.contains("eclipse.org/jdtls")
                || config.install_instructions.contains("brew install jdtls"),
            "Java install instructions should mention eclipse.org/jdtls or brew"
        );
    }

    #[test]
    fn test_kotlin_config() {
        let config = LspServerConfig::for_language(Language::Kotlin).unwrap();
        assert_eq!(config.language, Language::Kotlin);
        assert_eq!(config.name, "kotlin-language-server");
        assert_eq!(config.command, "kotlin-language-server");
        assert!(
            config.install_instructions.contains("kotlin-language-server/releases"),
            "Kotlin install instructions should mention releases page"
        );
    }

    #[test]
    fn test_all_languages_have_lsp_config() {
        // Ensure every Language variant has an LSP config
        let languages = [
            Language::Rust,
            Language::TypeScript,
            Language::Go,
            Language::Java,
            Language::Kotlin,
        ];

        for lang in &languages {
            let config = LspServerConfig::for_language(*lang);
            assert!(
                config.is_some(),
                "Language {:?} should have LSP config",
                lang
            );

            let config = config.unwrap();
            assert!(!config.name.is_empty(), "LSP name should not be empty");
            assert!(!config.command.is_empty(), "LSP command should not be empty");
            assert!(
                !config.install_instructions.is_empty(),
                "Install instructions should not be empty"
            );
        }
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
