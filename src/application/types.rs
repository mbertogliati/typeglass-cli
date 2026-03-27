use std::fmt::Debug;
use sealed::sealed;

use crate::ux_model::intent::{UserGoal, UserPromise};
use crate::ux_model::result::*;

/// Define la asociación entre una acción de comando y sus tipos de resultado.
pub trait CommandAction: Sized + Send + Sync {
    type Success: SuccessUserExpectations;
    type Partial: PartialSuccessUserExpectations;
    type Failure: FailureUserExpectations + AppFailureFactory;
}

/// Alias de tipo para el resultado de una acción, vinculando estáticamente 
/// la entrada con los tipos de salida esperados.
pub type ApplicationOutcome<C> = UserResult<
    <C as CommandAction>::Success,
    <C as CommandAction>::Partial,
    <C as CommandAction>::Failure,
>;

/// Factory para crear errores de aplicación que cumplan con las expectativas de UX.
pub trait AppFailureFactory {
    fn from_error(message: String, promises: Vec<UserPromise>, context: UserResultContext) -> Self;
}

// --- Tipos de Resultado Genéricos ---

#[derive(Debug)]
pub struct GenericSuccess {
    pub goal: UserGoal,
    pub promises: Vec<UserPromise>,
    pub summary: UserSummary,
    pub next_step: Option<UserNextStep>,
    pub context: UserResultContext,
}

impl SuccessUserExpectations for GenericSuccess {
    fn goal(&self) -> UserGoal { self.goal.clone() }
    fn promises(&self) -> Vec<UserPromise> { self.promises.clone() }
    fn summary(&self) -> UserSummary { self.summary.clone() }
    fn next_step(&self) -> Option<UserNextStep> { self.next_step.clone() }
    fn context(&self) -> UserResultContext { self.context.clone() }
}

#[derive(Debug)]
pub struct GenericPartial {
    pub goal: UserGoal,
    pub promises: Vec<UserPromise>,
    pub summary: UserSummary,
    pub limitations: Vec<UserLimitation>,
    pub next_step: Option<UserNextStep>,
    pub context: UserResultContext,
}

impl PartialSuccessUserExpectations for GenericPartial {
    fn goal(&self) -> UserGoal { self.goal.clone() }
    fn promises(&self) -> Vec<UserPromise> { self.promises.clone() }
    fn summary(&self) -> UserSummary { self.summary.clone() }
    fn limitations(&self) -> Vec<UserLimitation> { self.limitations.clone() }
    fn next_step(&self) -> Option<UserNextStep> { self.next_step.clone() }
    fn context(&self) -> UserResultContext { self.context.clone() }
}

#[derive(Debug)]
pub struct GenericFailure {
    pub promises: Vec<UserPromise>,
    pub summary: UserSummary,
    pub limitations: Vec<UserLimitation>,
    pub next_step: UserNextStep,
    pub context: UserResultContext,
}

impl std::fmt::Display for GenericFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary.0)
    }
}

impl std::error::Error for GenericFailure {}

impl FailureUserExpectations for GenericFailure {
    fn promises(&self) -> Vec<UserPromise> { self.promises.clone() }
    fn summary(&self) -> UserSummary { self.summary.clone() }
    fn limitations(&self) -> Vec<UserLimitation> { self.limitations.clone() }
    fn next_step(&self) -> UserNextStep { self.next_step.clone() }
    fn context(&self) -> UserResultContext { self.context.clone() }
}

impl AppFailureFactory for GenericFailure {
    fn from_error(message: String, promises: Vec<UserPromise>, context: UserResultContext) -> Self {
        Self {
            promises,
            summary: UserSummary(message),
            limitations: vec![],
            next_step: UserNextStep("Check command arguments".to_string()),
            context,
        }
    }
}
