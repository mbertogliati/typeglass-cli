use std::path::PathBuf;

use sealed::sealed;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserPromise(pub UserPromiseType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserGoalType {
    UnderstandCodebaseDomain,
    KeepWorkspaceClean,
    DiagnoseProblems,
    PrepareWorkspace,
    KeepAgentFlow,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserGoal(pub UserGoalType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserWorkspace {
    Explicit(PathBuf),
    Pwd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandContext {
    pub user_workspace: UserWorkspace,
}

#[sealed]
pub trait UserCommand {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandFrom {
    pub target: FromTarget,
    pub depth: Option<u8>,
}

#[sealed]
impl UserCommand for UserCommandFrom {}

impl UserCommandFrom {
    pub fn symbol(symbol: String, depth: Option<u8>) -> Self {
        Self {
            target: FromTarget::Symbol(symbol),
            depth,
        }
    }

    pub fn file(file: PathBuf, depth: Option<u8>) -> Self {
        Self {
            target: FromTarget::File(file),
            depth,
        }
    }

    pub fn module(module: PathBuf, depth: Option<u8>) -> Self {
        Self {
            target: FromTarget::Module(module),
            depth,
        }
    }

    pub fn public_exports(depth: Option<u8>) -> Self {
        Self {
            target: FromTarget::PublicExports,
            depth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandGc;

#[sealed]
impl UserCommand for UserCommandGc {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandDoctor;

#[sealed]
impl UserCommand for UserCommandDoctor {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandInit;

#[sealed]
impl UserCommand for UserCommandInit {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandInteractive;

#[sealed]
impl UserCommand for UserCommandInteractive {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FromTarget {
    Symbol(String),
    File(PathBuf),
    Module(PathBuf),
    PublicExports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRequest<C: UserCommand> {
    pub mode: UserMode,
    pub command: C,
    pub context: UserCommandContext,
}

impl<C: UserCommand> UserRequest<C> {
    pub fn terminal(command: C, context: UserCommandContext) -> Self {
        Self {
            mode: UserMode::Terminal,
            command,
            context,
        }
    }

    pub fn interactive(command: C, context: UserCommandContext) -> Self {
        Self {
            mode: UserMode::Interactive,
            command,
            context,
        }
    }

    pub const fn mode(&self) -> UserMode {
        self.mode
    }

    pub fn command(&self) -> &C {
        &self.command
    }

    pub fn context(&self) -> &UserCommandContext {
        &self.context
    }

    pub fn into_parts(self) -> (C, UserCommandContext) {
        (self.command, self.context)
    }
}

impl std::convert::From<UserGoalType> for UserGoal {
    fn from(value: UserGoalType) -> Self {
        Self(value)
    }
}

pub fn promises_for_generic_error() -> Vec<UserPromise> {
    vec![
        UserPromiseType::ErrorsAreActionable,
        UserPromiseType::ErrorsAreExplicit,
        UserPromiseType::FastByDefault,
    ]
    .into_iter()
    .map(UserPromise)
    .collect()
}
