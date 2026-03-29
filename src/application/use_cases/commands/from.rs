
use crate::application::adapters::ApplicationAdapters;
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::types::{ApplicationOutcome, CommandAction, GenericSuccess, GenericFailure};
use crate::domain::language::Language;
use crate::domain::ports::{LspPort, WorkspacePort};
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
        // Extract target query
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
                return UserResult::Failure(GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreExplicit),
                    ],
                    summary: UserSummary("Public exports traversal not yet implemented".to_string()),
                    limitations: vec![UserLimitation(
                        "Use --symbol, --file, or --module instead".to_string(),
                    )],
                    next_step: UserNextStep("Try 'typeglass from --symbol MyType'".to_string()),
                    context: UserResultContext {
                        command_context: context,
                    },
                });
            }
        };

        // Get workspace root
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

        // TODO(architecture): Validate workspace using WorkspacePort before proceeding
        // Currently using direct filesystem checks as fallback
        // Should call: self.adapters.workspace().probe_workspace(workspace_root).await?

        // Detect language (simplified - just check for Rust for now)
        let language = if workspace_root.join("Cargo.toml").exists() {
            Language::Rust
        } else if workspace_root.join("package.json").exists() {
            Language::TypeScript
        } else if workspace_root.join("go.mod").exists() {
            Language::Go
        } else {
            return UserResult::Failure(crate::application::types::GenericFailure {
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::ErrorsAreActionable),
                ],
                summary: UserSummary("Could not detect project language".to_string()),
                limitations: vec![
                    UserLimitation("No Cargo.toml, package.json, or go.mod found".to_string()),
                ],
                next_step: UserNextStep(
                    "Run from a Rust, TypeScript, or Go project root directory".to_string(),
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

