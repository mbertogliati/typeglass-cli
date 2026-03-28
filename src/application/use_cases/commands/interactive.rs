use std::path::PathBuf;

use crate::ux_model::intent::{UserCommandInteractive, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext, UserWorkspace};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext, UserLimitation, UserNextStep};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;
use crate::domain::language::Language;

impl CommandAction for UserCommandInteractive {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandInteractive> for ApplicationService<A> {
    async fn execute_action(&self, _action: UserCommandInteractive, context: UserCommandContext) -> ApplicationOutcome<UserCommandInteractive> {
        // Get workspace root
        let workspace_root = match &context.user_workspace {
            UserWorkspace::Explicit(path) => path.clone(),
            UserWorkspace::Pwd => {
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
                                "Run from a valid directory".to_string(),
                            ),
                            context: UserResultContext { command_context: context },
                        });
                    }
                }
            }
        };

        // Detect language
        let language = if workspace_root.join("Cargo.toml").exists() {
            Language::Rust
        } else if workspace_root.join("package.json").exists() {
            Language::TypeScript
        } else if workspace_root.join("go.mod").exists() {
            Language::Go
        } else {
            return UserResult::Failure(GenericFailure {
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::ErrorsAreActionable),
                ],
                summary: UserSummary("Could not detect project language".to_string()),
                limitations: vec![
                    UserLimitation("No Cargo.toml, package.json, or go.mod found".to_string()),
                ],
                next_step: UserNextStep(
                    "Run from a Rust, TypeScript, or Go project root".to_string(),
                ),
                context: UserResultContext { command_context: context },
            });
        };

        // Start interactive REPL
        println!("\n🔍 TypeGlass Interactive Mode");
        println!("Workspace: {}", workspace_root.display());
        println!("Language: {:?}", language);
        println!("\nCommands:");
        println!("  explore <symbol>     - Build graph from symbol");
        println!("  list                 - List all workspace symbols");
        println!("  help                 - Show this help");
        println!("  exit / quit          - Exit interactive mode");
        println!();

        use rustyline::error::ReadlineError;
        use rustyline::DefaultEditor;

        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(e) => {
                return UserResult::Failure(GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreActionable),
                    ],
                    summary: UserSummary("Failed to create readline editor".to_string()),
                    limitations: vec![UserLimitation(format!("Readline error: {}", e))],
                    next_step: UserNextStep(
                        "Check terminal compatibility".to_string(),
                    ),
                    context: UserResultContext { command_context: context },
                });
            }
        };

        loop {
            let readline = rl.readline("typeglass> ");
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = rl.add_history_entry(trimmed);

                    // Parse command
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    match parts[0] {
                        "exit" | "quit" => {
                            println!("Goodbye!");
                            break;
                        },
                        "help" => {
                            println!("Available commands:");
                            println!("  explore <symbol>  - Build type graph from symbol");
                            println!("  list              - List workspace symbols (coming soon)");
                            println!("  help              - Show this help");
                            println!("  exit / quit       - Exit interactive mode");
                        },
                        "explore" => {
                            if parts.len() < 2 {
                                println!("Usage: explore <symbol>");
                                continue;
                            }
                            let symbol = parts[1];
                            println!("🔍 Exploring symbol: {}", symbol);
                            println!("(LSP graph building not yet wired to REPL)");
                            println!("Hint: Use 'typeglass from --symbol {}' in non-interactive mode", symbol);
                        },
                        "list" => {
                            println!("📋 Listing symbols...");
                            println!("(Symbol listing not yet implemented in REPL)");
                            println!("Hint: Use 'typeglass from --module .' to see module symbols");
                        },
                        _ => {
                            println!("Unknown command: {}", parts[0]);
                            println!("Type 'help' for available commands");
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                },
                Err(err) => {
                    return UserResult::Failure(GenericFailure {
                        promises: vec![
                            UserPromise(UserPromiseType::NeverSilentWrong),
                            UserPromise(UserPromiseType::ErrorsAreActionable),
                        ],
                        summary: UserSummary(format!("Readline error: {}", err)),
                        limitations: vec![UserLimitation("REPL input failed".to_string())],
                        next_step: UserNextStep(
                            "Check terminal configuration".to_string(),
                        ),
                        context: UserResultContext { command_context: context },
                    });
                }
            }
        }

        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::KeepAgentFlow),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary("Interactive session completed".to_string()),
            next_step: None,
            context: UserResultContext { command_context: context },
        })
    }
}
