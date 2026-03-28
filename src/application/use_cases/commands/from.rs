use std::path::PathBuf;

use crate::application::adapters::ApplicationAdapters;
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::types::{ApplicationOutcome, CommandAction, GenericSuccess};
use crate::domain::graph::TraversalDirection;
use crate::domain::language::Language;
use crate::infrastructure::LazyGraphBuilder;
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
        // Extract symbol name from target
        let target_str = match &action.target {
            crate::ux_model::intent::FromTarget::Symbol(s) => s.clone(),
            crate::ux_model::intent::FromTarget::File(_)
            | crate::ux_model::intent::FromTarget::Module(_)
            | crate::ux_model::intent::FromTarget::PublicExports => {
                return UserResult::Failure(crate::application::types::GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreExplicit),
                    ],
                    summary: UserSummary("Only symbol-based traversal is currently implemented".to_string()),
                    limitations: vec![UserLimitation(
                        "File, module, and public exports traversal are not yet supported".to_string(),
                    )],
                    next_step: UserNextStep("Use a symbol name instead, e.g., 'typeglass from MyType'".to_string()),
                    context: UserResultContext {
                        command_context: context,
                    },
                });
            }
        };

        // Get workspace root
        let workspace_root = match &context.user_workspace {
            crate::ux_model::intent::UserWorkspace::Explicit(path) => path.clone(),
            crate::ux_model::intent::UserWorkspace::Pwd => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };

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

        // Build graph using LSP
        let max_depth = action.depth.unwrap_or(5);
        let direction = TraversalDirection::Both;

        let builder_result = LazyGraphBuilder::new(workspace_root.clone(), language).await;
        
        let graph_result = match builder_result {
            Ok(mut builder) => {
                let graph = builder.build_from_symbol(&target_str, direction, max_depth).await;
                let _ = builder.shutdown().await;
                graph
            }
            Err(e) => Err(e),
        };

        match graph_result {
            Ok(graph) => {
                let node_count = graph.nodes().len();
                let edge_count = graph.edges().len();
                let completeness = if graph.is_complete() {
                    "complete"
                } else {
                    "partial"
                };

                let summary = format!(
                    "Found {} nodes and {} edges from '{}' ({} graph, language: {:?})",
                    node_count, edge_count, target_str, completeness, language
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

