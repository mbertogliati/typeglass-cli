use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserMode {
    Terminal,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserPromiseType {
    FastByDefault,
    NeverSilentWrong,
    PartialResultsAreExplicit,
    ErrorsAreActionable,
    ErrorsAreExplicit,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UserPromise(UserPromiseType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserGoalType {
    UnderstandCodebaseDomain,
    KeepWorkspaceClean,
    DiagnoseProblems,
    PrepareWorkspace,
    KeepAgentFlow,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UserGoal(UserGoalType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserWorkspace {
    Explicit(PathBuf),
    Pwd,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommand {
    command_type: UserCommandType,
    user_workspace: UserWorkspace,
}

impl UserCommand {
    fn new(command_type: UserCommandType, user_workspace: UserWorkspace) -> Self {
        Self {command_type, user_workspace}
    }
    
    fn new_with_default_workspace(command_type: UserCommandType) -> Self {
        Self::new(command_type, UserWorkspace::Pwd)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserCommandType {
    From {
        target: FromTarget,
        depth: Option<u8>,
    },
    Gc,
    Doctor,
    Init,
    Interactive,
}

impl UserCommand {
    pub const fn goal(&self) -> UserGoal {
        goal_for_command(self)
    }
    pub fn success_promises(&self) -> Vec<UserPromise> {
        promises_for_command_success(self)
    }
    pub fn error_promises(&self) -> Vec<UserPromise> {
        promises_for_command_error(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum FromTarget {
    Symbol(String),
    File(PathBuf),
    Module(PathBuf),
    PublicExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRequest {
    mode: UserMode,
    command: UserCommand,
}

impl UserRequest {
    pub fn terminal(command: UserCommand) -> Self {
        Self {
            mode: UserMode::Terminal,
            command,
        }
    }

    pub fn interactive(command: UserCommand) -> Self {
        Self {
            mode: UserMode::Interactive,
            command,
        }
    }

    pub const fn mode(&self) -> UserMode {
        self.mode
    }

    pub const fn command(&self) -> &UserCommand {
        &self.command
    }
}

const fn goal_for_command(command: &UserCommand) -> UserGoal {
    UserGoal(match command.command_type {
        UserCommandType::From => UserGoalType::UnderstandCodebaseDomain,
        UserCommandType::Gc => UserGoalType::KeepWorkspaceClean,
        UserCommandType::Doctor => UserGoalType::DiagnoseProblems,
        UserCommandType::Init => UserGoalType::PrepareWorkspace,
        UserCommandType::Interactive => UserGoalType::KeepAgentFlow,
    })
}

fn promises_for_command_success(command: &UserCommand) -> Vec<UserPromise> {
    let mut promises = vec![UserPromiseType::NeverSilentWrong];

    match command.command_type {
        UserCommandType::From => {
            promises.push(UserPromiseType::FastByDefault);
            promises.push(UserPromiseType::PartialResultsAreExplicit);
        }
        UserCommandType::Gc | UserCommandType::Doctor | UserCommandType::Init | UserCommandType::Interactive => {
            promises.push(UserPromiseType::FastByDefault);
        }
    }

    promises.map(UserPromise::from)
}

fn promises_for_command_error(_command: &UserCommand) -> Vec<UserPromise> {
    vec![
        UserPromiseType::ErrorsAreActionable,
        UserPromiseType::ErrorsAreExplicit,
        UserPromiseType::FastByDefault,
    ]
    .map(UserPromise::from)
}
