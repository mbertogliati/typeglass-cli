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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ux_model::intent::{UserCommandContext, UserWorkspace};

    #[test]
    fn test_user_result_status() {
        assert_eq!(UserResultStatus::Success, UserResultStatus::Success);
        assert_ne!(UserResultStatus::Success, UserResultStatus::Failure);
    }
    
    #[test]
    fn test_user_summary_creation() {
        let summary = UserSummary("Test summary".to_string());
        assert_eq!(summary.0, "Test summary");
    }
    
    #[test]
    fn test_user_limitation_creation() {
        let lim = UserLimitation("Cannot do X".to_string());
        assert_eq!(lim.0, "Cannot do X");
    }
    
    #[test]
    fn test_user_next_step_creation() {
        let step = UserNextStep("Run command Y".to_string());
        assert_eq!(step.0, "Run command Y");
    }
    
    #[test]
    fn test_user_result_status_variants() {
        assert_ne!(UserResultStatus::Success, UserResultStatus::Partial);
        assert_ne!(UserResultStatus::Partial, UserResultStatus::Failure);
    }
    
    #[test]
    fn test_user_workspace_context() {
        let ctx = UserWorkspaceContext {
            used_path: PathBuf::from("/workspace"),
            override_path: None,
        };
        assert_eq!(ctx.used_path, PathBuf::from("/workspace"));
        assert!(ctx.override_path.is_none());
    }
    
    #[test]
    fn test_user_workspace_context_with_override() {
        let ctx = UserWorkspaceContext {
            used_path: PathBuf::from("/workspace"),
            override_path: Some(PathBuf::from("/override")),
        };
        assert_eq!(ctx.used_path, PathBuf::from("/workspace"));
        assert_eq!(ctx.override_path, Some(PathBuf::from("/override")));
    }

    #[test]
    fn test_user_summary() {
        let summary = UserSummary("test summary".to_string());
        assert_eq!(summary.0, "test summary");
    }

    #[test]
    fn test_user_limitation() {
        let lim = UserLimitation("limited".to_string());
        assert_eq!(lim.0, "limited");
    }

    #[test]
    fn test_user_next_step() {
        let step = UserNextStep("do this".to_string());
        assert_eq!(step.0, "do this");
    }

    #[test]
    fn test_user_result_context() {
        let ctx = UserResultContext {
            command_context: UserCommandContext {
                user_workspace: UserWorkspace::Pwd,
            },
        };
        assert_eq!(ctx.command_context.user_workspace, UserWorkspace::Pwd);
    }

    #[test]
    fn test_user_workspace_context_alt() {
        let ws_ctx = UserWorkspaceContext {
            used_path: PathBuf::from("/tmp"),
            override_path: None,
        };
        assert_eq!(ws_ctx.used_path, PathBuf::from("/tmp"));
        assert!(ws_ctx.override_path.is_none());
    }
}
