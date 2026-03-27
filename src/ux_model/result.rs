use std::{error::Error, fmt::Debug, path::PathBuf};

use super::intent::{UserCommandContext, UserGoal, UserPromise};

pub trait SuccessUserExpectations: Debug {
    fn goal(&self) -> UserGoal;
    fn promises(&self) -> Vec<UserPromise>;
    fn summary(&self) -> UserSummary;
    fn next_step(&self) -> Option<UserNextStep>;
    fn context(&self) -> UserResultContext;
}

pub trait FailureUserExpectations: Error + Debug {
    fn promises(&self) -> Vec<UserPromise>;
    fn summary(&self) -> UserSummary;
    fn limitations(&self) -> Vec<UserLimitation>;
    fn next_step(&self) -> UserNextStep;
    fn context(&self) -> UserResultContext;
}

pub trait PartialSuccessUserExpectations: Debug {
    fn goal(&self) -> UserGoal;
    fn promises(&self) -> Vec<UserPromise>;
    fn summary(&self) -> UserSummary;
    fn limitations(&self) -> Vec<UserLimitation>;
    fn next_step(&self) -> Option<UserNextStep>;
    fn context(&self) -> UserResultContext;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserResultStatus {
    Success,
    Partial,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSummary(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserLimitation(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserNextStep(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserResultContext {
    pub command_context: UserCommandContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWorkspaceContext {
    pub used_path: PathBuf,
    pub override_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserResult<S, P, F>
where
    S: SuccessUserExpectations,
    P: PartialSuccessUserExpectations,
    F: FailureUserExpectations,
{
    Success(S),
    Partial(P),
    Failure(F),
}

impl<S, P, F> UserResult<S, P, F>
where
    S: SuccessUserExpectations,
    P: PartialSuccessUserExpectations,
    F: FailureUserExpectations,
{
    pub fn status(&self) -> UserResultStatus {
        match self {
            Self::Success(_) => UserResultStatus::Success,
            Self::Partial(_) => UserResultStatus::Partial,
            Self::Failure(_) => UserResultStatus::Failure,
        }
    }

    pub fn goal(&self) -> Option<UserGoal> {
        match self {
            Self::Success(success) => Some(success.goal()),
            Self::Partial(partial) => Some(partial.goal()),
            Self::Failure(_) => None,
        }
    }

    pub fn promises(&self) -> Vec<UserPromise> {
        match self {
            Self::Success(success) => success.promises(),
            Self::Partial(partial) => partial.promises(),
            Self::Failure(failure) => failure.promises(),
        }
    }

    pub fn summary(&self) -> UserSummary {
        match self {
            Self::Success(success) => success.summary(),
            Self::Partial(partial) => partial.summary(),
            Self::Failure(failure) => failure.summary(),
        }
    }
    
    pub fn limitations(&self) -> Vec<UserLimitation> {
        match self {
            Self::Success(_) => Vec::new(),
            Self::Partial(partial) => partial.limitations(),
            Self::Failure(failure) => failure.limitations(),
        }
    }

    pub fn next_step(&self) -> Option<UserNextStep> {
        match self {
            Self::Success(success) => success.next_step(),
            Self::Partial(partial) => partial.next_step(),
            Self::Failure(failure) => Some(failure.next_step()),
        }
    }

    pub fn context(&self) -> UserResultContext {
        match self {
            Self::Success(success) => success.context(),
            Self::Partial(partial) => partial.context(),
            Self::Failure(failure) => failure.context(),
        }
    }
}
