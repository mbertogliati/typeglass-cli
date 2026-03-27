use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserMode {
    Terminal,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserPromise {
    FastByDefault,
    NeverSilentWrong,
    PartialResultsAreExplicit,
    ErrorsAreActionable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserGoal {
    UnderstandDomain,
    KeepWorkspaceClean,
    DiagnoseProblems,
    PrepareWorkspace,
    KeepAgentFlow,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FromTarget {
    Symbol(String),
    File(PathBuf),
    Module(PathBuf),
    PublicExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserCommand {
    From {
        target: FromTarget,
        depth: Option<u8>,
    },
    Gc,
    Doctor,
    Init,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserCommandKind {
    From,
    Gc,
    Doctor,
    Init,
    Interactive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRequest {
    pub mode: UserMode,
    pub command: UserCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserIntentContract {
    pub mode: UserMode,
    pub command: UserCommandKind,
    pub goal: UserGoal,
    pub promises: Vec<UserPromise>,
}

impl UserCommand {
    pub const fn kind(&self) -> UserCommandKind {
        match self {
            Self::From { .. } => UserCommandKind::From,
            Self::Gc => UserCommandKind::Gc,
            Self::Doctor => UserCommandKind::Doctor,
            Self::Init => UserCommandKind::Init,
            Self::Interactive => UserCommandKind::Interactive,
        }
    }
}

impl UserIntentContract {
    pub fn from_request(request: &UserRequest) -> Self {
        Self {
            mode: request.mode,
            command: request.command.kind(),
            goal: goal_for_command(&request.command),
            promises: promises_for_command(&request.command),
        }
    }
}

pub const fn goal_for_command(command: &UserCommand) -> UserGoal {
    match command {
        UserCommand::From { .. } => UserGoal::UnderstandDomain,
        UserCommand::Gc => UserGoal::KeepWorkspaceClean,
        UserCommand::Doctor => UserGoal::DiagnoseProblems,
        UserCommand::Init => UserGoal::PrepareWorkspace,
        UserCommand::Interactive => UserGoal::KeepAgentFlow,
    }
}

pub fn promises_for_command(command: &UserCommand) -> Vec<UserPromise> {
    let mut promises = vec![
        UserPromise::NeverSilentWrong,
        UserPromise::ErrorsAreActionable,
    ];

    match command {
        UserCommand::From { .. } => {
            promises.push(UserPromise::FastByDefault);
            promises.push(UserPromise::PartialResultsAreExplicit);
        }
        UserCommand::Gc | UserCommand::Doctor | UserCommand::Init | UserCommand::Interactive => {
            promises.push(UserPromise::FastByDefault);
        }
    }

    promises
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_contract_exposes_user_goal_and_promises() {
        let request = UserRequest {
            mode: UserMode::Terminal,
            command: UserCommand::From {
                target: FromTarget::PublicExports,
                depth: Some(2),
            },
        };

        let contract = UserIntentContract::from_request(&request);
        assert!(matches!(contract.goal, UserGoal::UnderstandDomain));
        assert!(contract
            .promises
            .contains(&UserPromise::PartialResultsAreExplicit));
    }
}
