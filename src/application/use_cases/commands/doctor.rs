use crate::application::adapters::ApplicationAdapters;
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::types::{ApplicationOutcome, CommandAction, GenericSuccess};
use crate::domain::language::{Language, LspInstallStatus, LspServerConfig};
use crate::ux_model::intent::{
    UserCommandContext, UserCommandDoctor, UserGoal, UserGoalType, UserPromise, UserPromiseType,
};
use crate::ux_model::result::{UserLimitation, UserResult, UserResultContext, UserSummary};

impl CommandAction for UserCommandDoctor {
    type Success = GenericSuccess;
    type Partial = crate::application::types::GenericPartial;
    type Failure = crate::application::types::GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandDoctor> for ApplicationService<A> {
    async fn execute_action(
        &self,
        _action: UserCommandDoctor,
        context: UserCommandContext,
    ) -> ApplicationOutcome<UserCommandDoctor> {
        let mut report = Vec::new();
        let mut all_installed = true;
        let mut limitations = Vec::new();

        // Check all supported languages
        let languages = [
            Language::TypeScript,
            Language::Rust,
            Language::Go,
            Language::Java,
            Language::Kotlin,
        ];

        for language in &languages {
            if let Some(config) = LspServerConfig::for_language(*language) {
                let status = config.check_installation();
                let (status_emoji, status_text, path_info) = match &status {
                    LspInstallStatus::Installed { path } => {
                        ("✅", "INSTALLED", format!(" ({})", path.display()))
                    }
                    LspInstallStatus::NotFound => {
                        all_installed = false;
                        limitations.push(UserLimitation(format!(
                            "{:?} LSP not installed: {}",
                            language,
                            config.name
                        )));
                        ("❌", "NOT FOUND", String::new())
                    }
                    LspInstallStatus::NotExecutable { path } => {
                        all_installed = false;
                        limitations.push(UserLimitation(format!(
                            "{:?} LSP not executable: {} at {}",
                            language,
                            config.name,
                            path.display()
                        )));
                        ("⚠️", "NOT EXECUTABLE", format!(" ({})", path.display()))
                    }
                };

                report.push(format!(
                    "{} {:?} - {} - {}{}\n    Command: {}\n    Install: {}",
                    status_emoji,
                    language,
                    config.name,
                    status_text,
                    path_info,
                    config.command,
                    config.install_instructions.lines().next().unwrap_or("")
                ));

                // Add full install instructions if not installed
                if !matches!(status, LspInstallStatus::Installed { .. }) {
                    report.push(format!("    Full instructions:\n    {}", 
                        config.install_instructions.replace('\n', "\n    ")));
                }
            }
        }

        let summary = if all_installed {
            format!(
                "System check complete - all {} LSP servers installed\n\n{}",
                languages.len(),
                report.join("\n\n")
            )
        } else {
            format!(
                "System check complete - some LSP servers missing\n\n{}",
                report.join("\n\n")
            )
        };

        if all_installed {
            UserResult::Success(GenericSuccess {
                goal: UserGoal(UserGoalType::DiagnoseProblems),
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::ErrorsAreActionable),
                ],
                summary: UserSummary(summary),
                next_step: Some(crate::ux_model::result::UserNextStep(
                    "All LSP servers are ready. You can start using typeglass.".to_string()
                )),
                context: UserResultContext {
                    command_context: context,
                },
            })
        } else {
            UserResult::Partial(crate::application::types::GenericPartial {
                goal: UserGoal(UserGoalType::DiagnoseProblems),
                promises: vec![
                    UserPromise(UserPromiseType::NeverSilentWrong),
                    UserPromise(UserPromiseType::PartialResultsAreExplicit),
                    UserPromise(UserPromiseType::ErrorsAreActionable),
                ],
                summary: UserSummary(summary),
                limitations,
                next_step: Some(crate::ux_model::result::UserNextStep(
                    "Install missing LSP servers using the instructions above.".to_string()
                )),
                context: UserResultContext {
                    command_context: context,
                },
            })
        }
    }
}
