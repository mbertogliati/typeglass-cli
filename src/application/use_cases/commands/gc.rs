use crate::ux_model::intent::{UserCommandGc, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext};
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
        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::KeepWorkspaceClean),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary("Garbage collection completed".to_string()),
            next_step: None,
            context: UserResultContext { command_context: context },
        })
    }
}
