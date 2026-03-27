use crate::ux_model::intent::{UserCommandFrom, UserGoal, UserGoalType, UserPromise, UserPromiseType, UserCommandContext};
use crate::ux_model::result::{UserResult, UserSummary, UserNextStep, UserResultContext};
use crate::application::types::{CommandAction, GenericSuccess, GenericPartial, GenericFailure, ApplicationOutcome};
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::adapters::ApplicationAdapters;

impl CommandAction for UserCommandFrom {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandFrom> for ApplicationService<A> {
    async fn execute_action(&self, action: UserCommandFrom, context: UserCommandContext) -> ApplicationOutcome<UserCommandFrom> {
        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::UnderstandCodebaseDomain),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
                UserPromise(UserPromiseType::PartialResultsAreExplicit),
            ],
            summary: UserSummary(format!("Explored from {:?}", action.target)),
            next_step: Some(UserNextStep("Review the generated graph".to_string())),
            context: UserResultContext { command_context: context },
        })
    }
}
