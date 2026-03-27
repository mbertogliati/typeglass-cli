use crate::ux_model::intent::{UserCommandInit, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;

impl CommandAction for UserCommandInit {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandInit> for ApplicationService<A> {
    async fn execute_action(&self, _action: UserCommandInit, context: UserCommandContext) -> ApplicationOutcome<UserCommandInit> {
        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::PrepareWorkspace),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary("Workspace initialized".to_string()),
            next_step: None,
            context: UserResultContext { command_context: context },
        })
    }
}
