use crate::ux_model::intent::{UserCommandInteractive, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;

impl CommandAction for UserCommandInteractive {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandInteractive> for ApplicationService<A> {
    async fn execute_action(&self, _action: UserCommandInteractive, context: UserCommandContext) -> ApplicationOutcome<UserCommandInteractive> {
        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::KeepAgentFlow),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary("Interactive session started".to_string()),
            next_step: None,
            context: UserResultContext { command_context: context },
        })
    }
}
