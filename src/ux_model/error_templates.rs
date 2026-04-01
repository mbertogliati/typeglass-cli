//! Error message templates with actionable next steps
//! 
//! This module provides helper functions to create user-friendly error messages
//! with specific limitations and actionable next steps.

use crate::domain::language::Language;
use crate::ux_model::result::{UserLimitation, UserNextStep, UserSummary};

/// Create error message for LSP server not found
pub fn lsp_not_found(language: Language) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    let binary = lsp_binary_name(&language);
    let install_cmd = lsp_install_command(&language);
    
    (
        UserSummary(format!("{:?} language server not found", language)),
        vec![
            UserLimitation(format!(
                "Could not find '{}' in your system PATH",
                binary
            )),
            UserLimitation(
                "TypeGlass requires a language server to analyze your code".to_string()
            ),
        ],
        UserNextStep(format!(
            "Install the language server:\n  {}\n\nThen verify it's in PATH:\n  which {}",
            install_cmd,
            binary
        )),
    )
}

/// Create error message for symbol not found
pub fn symbol_not_found(symbol: &str, suggestions: Vec<String>) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    let next_step = if suggestions.is_empty() {
        UserNextStep(format!(
            "Try one of these:\n  \
            • Check spelling of '{}'\n  \
            • Use 'grep -r \"struct {}\\|class {}\\|type {}\" src/' to find it\n  \
            • Run 'typeglass from --public-exports' to see all available symbols",
            symbol, symbol, symbol, symbol
        ))
    } else {
        UserNextStep(format!(
            "Did you mean one of these?\n{}\n\n\
            Or run 'typeglass from --public-exports' to see all symbols",
            suggestions.iter()
                .take(5)
                .map(|s| format!("  • {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    };
    
    (
        UserSummary(format!("Symbol '{}' not found in workspace", symbol)),
        vec![
            UserLimitation(format!(
                "The symbol '{}' does not exist in this workspace",
                symbol
            )),
            UserLimitation(
                "LSP could not locate any type, struct, class, or interface with this name".to_string()
            ),
        ],
        next_step,
    )
}

/// Create error message for workspace detection failure
pub fn workspace_not_detected() -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary("Could not detect workspace type".to_string()),
        vec![
            UserLimitation(
                "No Cargo.toml, package.json, tsconfig.json, or go.mod found".to_string()
            ),
            UserLimitation(
                "TypeGlass needs a recognized project structure to work".to_string()
            ),
        ],
        UserNextStep(
            "Make sure you're in a valid project directory with one of:\n  \
            • Rust: Cargo.toml\n  \
            • TypeScript/JavaScript: package.json or tsconfig.json\n  \
            • Go: go.mod\n\n\
            Or run 'typeglass init' to create a .typeglass.toml config".to_string()
        ),
    )
}

/// Create error message for cache corruption
pub fn cache_corrupted(cache_path: &str) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary("Cache file is corrupted".to_string()),
        vec![
            UserLimitation(format!(
                "Could not read cache file at {}",
                cache_path
            )),
            UserLimitation(
                "The cache may be from an incompatible version or corrupted".to_string()
            ),
        ],
        UserNextStep(
            "Clear the cache to fix this:\n  typeglass gc\n\n\
            This will rebuild the cache on the next query.".to_string()
        ),
    )
}

/// Create error message for max depth reached
pub fn max_depth_reached(depth: u8) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary(format!("Reached maximum traversal depth of {}", depth)),
        vec![
            UserLimitation(format!(
                "Graph traversal stopped at depth {} to prevent infinite loops",
                depth
            )),
            UserLimitation(
                "Some type relationships may not be fully explored".to_string()
            ),
        ],
        UserNextStep(format!(
            "To explore deeper relationships:\n  \
            typeglass from --symbol YourType --depth {}\n\n\
            Warning: Higher depths may take longer and produce larger graphs",
            depth + 5
        )),
    )
}

/// Create error message for circular dependency detected
pub fn circular_dependency(cycle_path: Vec<String>) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    let cycle_str = cycle_path.join(" → ");
    
    (
        UserSummary("Circular dependency detected".to_string()),
        vec![
            UserLimitation(format!(
                "Found cycle: {}",
                cycle_str
            )),
            UserLimitation(
                "TypeGlass stopped traversal to prevent infinite loop".to_string()
            ),
        ],
        UserNextStep(
            "This is expected for recursive types.\n\
            The graph shows the cycle but doesn't traverse it infinitely.\n\n\
            To see the full graph, increase --depth if needed.".to_string()
        ),
    )
}

/// Create error message for file not readable
pub fn file_not_readable(path: &str, reason: &str) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary(format!("Cannot read file: {}", path)),
        vec![
            UserLimitation(format!(
                "File {} is not readable: {}",
                path, reason
            )),
        ],
        UserNextStep(
            "Check file permissions:\n  \
            ls -la PATH\n  \
            chmod +r PATH\n\n\
            Or skip this file and continue analysis.".to_string()
        ),
    )
}

/// Create error message for LSP initialization failure
pub fn lsp_init_failed(language: Language, error: &str) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary(format!("{:?} LSP failed to initialize", language)),
        vec![
            UserLimitation(format!(
                "Language server error: {}",
                error
            )),
            UserLimitation(
                "The LSP server started but could not initialize the workspace".to_string()
            ),
        ],
        UserNextStep(format!(
            "Try these steps:\n  \
            1. Verify {} is installed correctly\n  \
            2. Check that workspace has valid project files\n  \
            3. Run 'typeglass doctor' to diagnose issues\n  \
            4. Check LSP logs for more details",
            lsp_binary_name(&language)
        )),
    )
}

/// Create error message for invalid configuration
pub fn invalid_config(config_path: &str, error: &str) -> (UserSummary, Vec<UserLimitation>, UserNextStep) {
    (
        UserSummary("Invalid configuration file".to_string()),
        vec![
            UserLimitation(format!(
                "Configuration file {} has errors: {}",
                config_path, error
            )),
        ],
        UserNextStep(
            "Fix the configuration file or regenerate it:\n  \
            typeglass init --force\n\n\
            See documentation for valid config options.".to_string()
        ),
    )
}

/// Get LSP binary name for a language
fn lsp_binary_name(language: &Language) -> &'static str {
    match language {
        Language::Rust => "rust-analyzer",
        Language::TypeScript => "typescript-language-server",
        Language::Go => "gopls",
    }
}

/// Get installation command for LSP server
fn lsp_install_command(language: &Language) -> &'static str {
    match language {
        Language::Rust => "rustup component add rust-analyzer",
        Language::TypeScript => "npm install -g typescript-language-server typescript",
        Language::Go => "go install golang.org/x/tools/gopls@latest",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_not_found_rust() {
        let (summary, limitations, next_step) = lsp_not_found(Language::Rust);
        
        assert!(summary.0.contains("Rust"));
        assert!(limitations.iter().any(|l| l.0.contains("rust-analyzer")));
        assert!(next_step.0.contains("rustup"));
        assert!(next_step.0.contains("which rust-analyzer"));
    }

    #[test]
    fn test_lsp_not_found_typescript() {
        let (summary, limitations, next_step) = lsp_not_found(Language::TypeScript);
        
        assert!(summary.0.contains("TypeScript"));
        assert!(limitations.iter().any(|l| l.0.contains("typescript-language-server")));
        assert!(next_step.0.contains("npm install"));
    }

    #[test]
    fn test_symbol_not_found_no_suggestions() {
        let (summary, limitations, next_step) = symbol_not_found("MyType", vec![]);
        
        assert!(summary.0.contains("MyType"));
        assert!(limitations.iter().any(|l| l.0.contains("MyType")));
        assert!(next_step.0.contains("grep"));
        assert!(next_step.0.contains("--public-exports"));
    }

    #[test]
    fn test_symbol_not_found_with_suggestions() {
        let suggestions = vec!["MyClass".to_string(), "MyStruct".to_string(), "MyInterface".to_string()];
        let (summary, limitations, next_step) = symbol_not_found("MyType", suggestions);
        
        assert!(summary.0.contains("MyType"));
        assert!(next_step.0.contains("Did you mean"));
        assert!(next_step.0.contains("MyClass"));
        assert!(next_step.0.contains("MyStruct"));
    }

    #[test]
    fn test_workspace_not_detected() {
        let (summary, limitations, next_step) = workspace_not_detected();
        
        assert!(summary.0.contains("workspace"));
        assert!(limitations.iter().any(|l| l.0.contains("Cargo.toml")));
        assert!(next_step.0.contains("Rust: Cargo.toml"));
        assert!(next_step.0.contains("typeglass init"));
    }

    #[test]
    fn test_cache_corrupted() {
        let (summary, limitations, next_step) = cache_corrupted("/tmp/cache.bin");
        
        assert!(summary.0.contains("corrupted"));
        assert!(limitations.iter().any(|l| l.0.contains("/tmp/cache.bin")));
        assert!(next_step.0.contains("typeglass gc"));
    }

    #[test]
    fn test_max_depth_reached() {
        let (summary, limitations, next_step) = max_depth_reached(10);
        
        assert!(summary.0.contains("10"));
        assert!(limitations.iter().any(|l| l.0.contains("depth 10")));
        assert!(next_step.0.contains("--depth 15"));
    }

    #[test]
    fn test_circular_dependency() {
        let cycle = vec!["A".to_string(), "B".to_string(), "C".to_string(), "A".to_string()];
        let (summary, limitations, next_step) = circular_dependency(cycle);
        
        assert!(summary.0.contains("Circular"));
        assert!(limitations.iter().any(|l| l.0.contains("A → B → C → A")));
        assert!(next_step.0.contains("recursive"));
    }

    #[test]
    fn test_file_not_readable() {
        let (summary, limitations, next_step) = file_not_readable("/tmp/test.rs", "Permission denied");
        
        assert!(summary.0.contains("/tmp/test.rs"));
        assert!(limitations.iter().any(|l| l.0.contains("Permission denied")));
        assert!(next_step.0.contains("chmod"));
    }

    #[test]
    fn test_lsp_init_failed() {
        let (summary, limitations, next_step) = lsp_init_failed(Language::Rust, "workspace error");
        
        assert!(summary.0.contains("Rust"));
        assert!(limitations.iter().any(|l| l.0.contains("workspace error")));
        assert!(next_step.0.contains("typeglass doctor"));
    }

    #[test]
    fn test_invalid_config() {
        let (summary, limitations, next_step) = invalid_config(".typeglass.toml", "invalid syntax");
        
        assert!(summary.0.contains("configuration"));
        assert!(limitations.iter().any(|l| l.0.contains("invalid syntax")));
        assert!(next_step.0.contains("typeglass init --force"));
    }

    #[test]
    fn test_all_messages_have_actionable_next_steps() {
        // Verify all error templates have non-empty next steps
        let tests = vec![
            lsp_not_found(Language::Rust).2,
            symbol_not_found("Test", vec![]).2,
            workspace_not_detected().2,
            cache_corrupted("/tmp/test").2,
            max_depth_reached(5).2,
            circular_dependency(vec!["A".to_string()]).2,
            file_not_readable("/tmp/test", "error").2,
            lsp_init_failed(Language::Rust, "error").2,
            invalid_config(".test", "error").2,
        ];
        
        for next_step in tests {
            assert!(!next_step.0.is_empty(), "Next step should not be empty");
            assert!(next_step.0.len() > 20, "Next step should be descriptive");
        }
    }
}
