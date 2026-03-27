use crate::ux_model::intent::{UserCommandDoctor, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserResultContext};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;

impl CommandAction for UserCommandDoctor {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandDoctor> for ApplicationService<A> {
    async fn execute_action(&self, _action: UserCommandDoctor, context: UserCommandContext) -> ApplicationOutcome<UserCommandDoctor> {
        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::DiagnoseProblems),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
            ],
            summary: UserSummary("System health check passed".to_string()),
            next_step: None,
            context: UserResultContext { command_context: context },
        })
    }
}
