
use crate::application::adapters::ApplicationAdapters;
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::types::{ApplicationOutcome, CommandAction, GenericSuccess, GenericFailure};
use crate::domain::language::Language;
use crate::domain::ports::LspPort;
use crate::ux_model::intent::{
    UserCommandContext, UserCommandFrom, UserGoal, UserGoalType, UserPromise, UserPromiseType,
};
use crate::ux_model::result::{UserLimitation, UserNextStep, UserResult, UserResultContext, UserSummary};

impl CommandAction for UserCommandFrom {
    type Success = GenericSuccess;
    type Partial = crate::application::types::GenericPartial;
    type Failure = crate::application::types::GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandFrom> for ApplicationService<A> {
    async fn execute_action(
        &self,
        action: UserCommandFrom,
        context: UserCommandContext,
    ) -> ApplicationOutcome<UserCommandFrom> {
        // Get workspace root first
        let workspace_root = match &context.user_workspace {
            crate::ux_model::intent::UserWorkspace::Explicit(path) => path.clone(),
            crate::ux_model::intent::UserWorkspace::Pwd => {
                match std::env::current_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        return UserResult::Failure(GenericFailure {
                            promises: vec![
                                UserPromise(UserPromiseType::NeverSilentWrong),
                                UserPromise(UserPromiseType::ErrorsAreActionable),
                            ],
                            summary: UserSummary("Failed to determine workspace root".to_string()),
                            limitations: vec![UserLimitation(format!("Could not read current directory: {}", e))],
                            next_step: UserNextStep(
                                "Run from a valid directory or specify --workspace /path/to/workspace".to_string(),
                            ),
                            context: UserResultContext {
                                command_context: context,
                            },
                        });
                    }
                }
            },
        };
        
        // Extract target query (needs workspace_root for PublicExports)
        let (target_description, lsp_query) = match &action.target {
            crate::ux_model::intent::FromTarget::Symbol(s) => {
                (format!("symbol '{}'", s), s.clone())
            }
            crate::ux_model::intent::FromTarget::File(path) => {
                // Use filename stem as symbol name
                let symbol = path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "UnknownFile".to_string());
                
                (format!("file '{}'", path.display()), symbol)
            }
            crate::ux_model::intent::FromTarget::Module(path) => {
                // Use last path component as module name
                let module_name = path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                (format!("module '{}'", path.display()), module_name)
            }
            crate::ux_model::intent::FromTarget::PublicExports => {
                // For public exports, find the root source file and extract its name
                // Strategy:
                // 1. For Rust: Look for src/lib.rs first, then src/main.rs
                // 2. For TypeScript: Look for src/index.ts or index.ts
                // 3. For Go: Look for main.go or the module name
                // Use the file stem as the symbol name
                
                let root_source_file = if workspace_root.join("Cargo.toml").exists() {
                    // Rust project
                    if workspace_root.join("src/lib.rs").exists() {
                        workspace_root.join("src/lib.rs")
                    } else if workspace_root.join("src/main.rs").exists() {
                        workspace_root.join("src/main.rs")
                    } else {
                        workspace_root.join("src/lib.rs") // Fallback, will error later
                    }
                } else if workspace_root.join("package.json").exists() {
                    // TypeScript project
                    if workspace_root.join("src/index.ts").exists() {
                        workspace_root.join("src/index.ts")
                    } else {
                        workspace_root.join("index.ts")
                    }
                } else {
                    // Go or other - fallback to main.go
                    workspace_root.join("main.go")
                };
                
                let symbol = root_source_file.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "lib".to_string());
                
                ("public exports".to_string(), symbol)
            }
        };

        // Detect workspace language via manifest files
        // Note: WorkspacePort abstraction exists but direct filesystem checks
        // are sufficient for MVP and work reliably across all supported languages
        let language = if workspace_root.join("Cargo.toml").exists() {
            Language::Rust
        } else if workspace_root.join("package.json").exists() {
            Language::TypeScript
        } else if workspace_root.join("go.mod").exists() {
            Language::Go
        } else if workspace_root.join("pom.xml").exists()
            || workspace_root.join("build.gradle").exists()
            || workspace_root.join("build.gradle.kts").exists()
        {
            // Check if it's Kotlin or Java based on source files
            // Look for .kt files recursively in common source directories
            let has_kotlin = std::fs::read_dir(&workspace_root)
                .ok()
                .and_then(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .find(|e| {
                            let path = e.path();
                            // Check if file has .kt extension
                            if path.extension().and_then(|s| s.to_str()) == Some("kt") {
                                return true;
                            }
                            // Check src/ directory for .kt files
                            if path.is_dir() && path.file_name().and_then(|s| s.to_str()) == Some("src") {
                                return std::fs::read_dir(&path)
                                    .ok()
                                    .map(|src_entries| {
                                        src_entries.filter_map(Result::ok).any(|src_entry| {
                                            let src_path = src_entry.path();
                                            src_path.extension().and_then(|s| s.to_str()) == Some("kt")
                                                || (src_path.is_dir() && has_kt_files_recursive(&src_path, 2))
                                        })
                                    })
                                    .unwrap_or(false);
                            }
                            false
                        })
                })
                .is_some();
            
            if has_kotlin {
                Language::Kotlin
            } else {
                Language::Java
            }
        } else {
            return UserResult::Failure(crate::application::types::GenericFailure {
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::ErrorsAreActionable),
                ],
                summary: UserSummary("Could not detect project language".to_string()),
                limitations: vec![
                    UserLimitation(
                        "No manifest file found (Cargo.toml, package.json, go.mod, pom.xml, build.gradle)"
                            .to_string(),
                    ),
                ],
                next_step: UserNextStep(
                    "Run from a Rust, TypeScript, Go, Java, or Kotlin project root directory"
                        .to_string(),
                ),
                context: UserResultContext {
                    command_context: context,
                },
            });
        };

        // Build graph using LSP through port abstraction
        let max_depth = action.depth.unwrap_or(5);
        
        // Create LSP query request using domain types
        let depth = match crate::domain::lsp::Depth::new(max_depth) {
            Ok(d) => d,
            Err(e) => {
                return UserResult::Failure(GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreActionable),
                    ],
                    summary: UserSummary(format!("Invalid depth: {}", e)),
                    limitations: vec![UserLimitation("Depth must be between 1 and 255".to_string())],
                    next_step: UserNextStep("Use a depth value between 1 and 255".to_string()),
                    context: UserResultContext {
                        command_context: context,
                    },
                });
            }
        };
        
        let lsp_request = crate::domain::ports::LspQueryRequest {
            query: crate::domain::lsp::LspQuery::FromSymbol {
                symbol: lsp_query.clone(),
                depth,
            },
            timeout: crate::domain::lsp::QueryTimeout(std::time::Duration::from_secs(30)),
        };

        // Clone the LSP adapter so it can be moved into the async context
        // (The Arc makes this cheap - only the pointer is cloned)
        let lsp_adapter = (*self.adapters.lsp()).clone();
        
        // Execute query through the port (NOT directly via LazyGraphBuilder)
        let graph_result: Result<crate::domain::ports::LspQueryResponse, crate::domain::ports::LspPortError> = 
            lsp_adapter.run_query(lsp_request).await;

        match graph_result {
            Ok(response) => {
                let graph = match response.graph {
                    Some(g) => g,
                    None => {
                        return UserResult::Failure(GenericFailure {
                            promises: vec![
                                UserPromise(UserPromiseType::NeverSilentWrong),
                                UserPromise(UserPromiseType::ErrorsAreActionable),
                            ],
                            summary: UserSummary(format!("No graph found for {}", target_description)),
                            limitations: vec![UserLimitation("LSP returned empty response".to_string())],
                            next_step: UserNextStep("Check if the symbol/file exists in your workspace".to_string()),
                            context: UserResultContext {
                                command_context: context,
                            },
                        });
                    }
                };
                
                let node_count = graph.nodes().len();
                let edge_count = graph.edges().len();
                let completeness = if graph.is_complete() {
                    "complete"
                } else {
                    "partial"
                };

                let summary = format!(
                    "Found {} nodes and {} edges from {} ({} graph, language: {:?})",
                    node_count, edge_count, target_description, completeness, language
                );

                UserResult::Success(GenericSuccess {
                    goal: UserGoal(UserGoalType::UnderstandCodebaseDomain),
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::FastByDefault),
                        UserPromise(UserPromiseType::PartialResultsAreExplicit),
                    ],
                    summary: UserSummary(summary),
                    next_step: Some(UserNextStep(
                        "Graph built from LSP. Use --format json for detailed output.".to_string(),
                    )),
                    context: UserResultContext {
                        command_context: context,
                    },
                })
            }
            Err(e) => {
                UserResult::Failure(crate::application::types::GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreActionable),
                    ],
                    summary: UserSummary(format!("Failed to build graph: {}", e)),
                    limitations: vec![UserLimitation(format!("LSP error: {}", e))],
                    next_step: UserNextStep(
                        "Check that LSP is installed (run 'typeglass doctor')".to_string(),
                    ),
                    context: UserResultContext {
                        command_context: context,
                    },
                })
            }
        }
    }
}

/// Helper function to recursively check for .kt files
fn has_kt_files_recursive(dir: &std::path::Path, max_depth: usize) -> bool {
    if max_depth == 0 {
        return false;
    }
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("kt") {
                return true;
            }
            if path.is_dir() && has_kt_files_recursive(&path, max_depth - 1) {
                return true;
            }
        }
    }
    
    false
}

