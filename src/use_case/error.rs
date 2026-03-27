use thiserror::Error;

use crate::domain::language::Language;
use crate::domain::lsp::LanguageFeature;

use super::action::UserAction;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UseCaseError {
    #[error("Action {action:?} is not supported for language {language:?}. Missing feature: {feature:?}")]
    UnsupportedForLanguage {
        action: UserAction,
        language: Language,
        feature: LanguageFeature,
    },
    #[error("Action {action:?} received invalid input. Reason: {reason}")]
    InvalidInput { action: UserAction, reason: String },
    #[error("Action {action:?} failed. Reason: {reason}")]
    ExecutionFailed { action: UserAction, reason: String },
}
