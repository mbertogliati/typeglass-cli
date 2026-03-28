use std::path::PathBuf;

use crate::ux_model::intent::{UserCommandGc, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext, UserLimitation};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;

impl CommandAction for UserCommandGc {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandGc> for ApplicationService<A> {
    async fn execute_action(&self, _action: UserCommandGc, context: UserCommandContext) -> ApplicationOutcome<UserCommandGc> {
        // Get cache directory
        let cache_dir = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(home) => PathBuf::from(home).join(".typeglass").join("cache"),
            Err(_) => {
                return UserResult::Failure(GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreActionable),
                    ],
                    summary: UserSummary("Failed to locate cache directory".to_string()),
                    limitations: vec![UserLimitation("HOME or USERPROFILE environment variable not set".to_string())],
                    next_step: crate::ux_model::result::UserNextStep(
                        "Set HOME environment variable to your user directory".to_string(),
                    ),
                    context: UserResultContext { command_context: context },
                });
            }
        };

        // Check if cache directory exists
        if !cache_dir.exists() {
            return UserResult::Success(GenericSuccess {
                goal: UserGoal(UserGoalType::KeepWorkspaceClean),
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::FastByDefault),
                ],
                summary: UserSummary("No cache directory found (nothing to clean)".to_string()),
                next_step: None,
                context: UserResultContext { command_context: context },
            });
        }

        // Scan and remove cache files
        let mut removed_count = 0;
        let mut total_bytes = 0u64;
        let mut errors = Vec::new();

        match std::fs::read_dir(&cache_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            let file_size = metadata.len();
                            match std::fs::remove_file(entry.path()) {
                                Ok(_) => {
                                    removed_count += 1;
                                    total_bytes += file_size;
                                }
                                Err(e) => {
                                    errors.push(format!("Failed to remove {}: {}", entry.path().display(), e));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return UserResult::Failure(GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreActionable),
                    ],
                    summary: UserSummary("Failed to read cache directory".to_string()),
                    limitations: vec![UserLimitation(format!("IO error: {}", e))],
                    next_step: crate::ux_model::result::UserNextStep(
                        format!("Check permissions on {}", cache_dir.display()),
                    ),
                    context: UserResultContext { command_context: context },
                });
            }
        }

        // Format size in human-readable form
        let size_str = if total_bytes < 1024 {
            format!("{} bytes", total_bytes)
        } else if total_bytes < 1024 * 1024 {
            format!("{:.1} KB", total_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", total_bytes as f64 / (1024.0 * 1024.0))
        };

        let summary = if removed_count == 0 {
            "Cache is empty (nothing to clean)".to_string()
        } else if errors.is_empty() {
            format!("Removed {} cached files ({} freed)", removed_count, size_str)
        } else {
            format!("Removed {} files ({} freed) with {} errors", removed_count, size_str, errors.len())
        };

        let next_step = if !errors.is_empty() {
            Some(crate::ux_model::result::UserNextStep(
                format!("Errors: {}", errors.join(", ")),
            ))
        } else {
            None
        };

        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::KeepWorkspaceClean),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary(summary),
            next_step,
            context: UserResultContext { command_context: context },
        })
    }
}
