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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_mode_variants() {
        assert_eq!(UserMode::Terminal, UserMode::Terminal);
        assert_ne!(UserMode::Terminal, UserMode::Interactive);
    }

    #[test]
    fn test_user_promise_creation() {
        let promise = UserPromise(UserPromiseType::FastByDefault);
        assert_eq!(promise.0, UserPromiseType::FastByDefault);
    }

    #[test]
    fn test_user_goal_from() {
        let goal: UserGoal = UserGoalType::UnderstandCodebaseDomain.into();
        assert_eq!(goal.0, UserGoalType::UnderstandCodebaseDomain);
    }

    #[test]
    fn test_user_workspace_explicit() {
        let ws = UserWorkspace::Explicit(PathBuf::from("/tmp/test"));
        assert_eq!(ws, UserWorkspace::Explicit(PathBuf::from("/tmp/test")));
    }

    #[test]
    fn test_user_command_context() {
        let ctx = UserCommandContext {
            user_workspace: UserWorkspace::Pwd,
        };
        assert_eq!(ctx.user_workspace, UserWorkspace::Pwd);
    }

    #[test]
    fn test_user_command_from_symbol() {
        let cmd = UserCommandFrom::symbol("MyType".to_string(), Some(3));
        assert_eq!(cmd.target, FromTarget::Symbol("MyType".to_string()));
        assert_eq!(cmd.depth, Some(3));
    }

    #[test]
    fn test_user_command_from_file() {
        let cmd = UserCommandFrom::file(PathBuf::from("test.ts"), None);
        assert_eq!(cmd.target, FromTarget::File(PathBuf::from("test.ts")));
        assert_eq!(cmd.depth, None);
    }

    #[test]
    fn test_user_command_from_module() {
        let cmd = UserCommandFrom::module(PathBuf::from("src/"), Some(2));
        assert!(matches!(cmd.target, FromTarget::Module(_)));
    }

    #[test]
    fn test_user_command_from_public_exports() {
        let cmd = UserCommandFrom::public_exports(Some(5));
        assert_eq!(cmd.target, FromTarget::PublicExports);
    }

    #[test]
    fn test_user_request_terminal() {
        let ctx = UserCommandContext { user_workspace: UserWorkspace::Pwd };
        let cmd = UserCommandFrom::symbol("Test".to_string(), None);
        let req = UserRequest::terminal(cmd.clone(), ctx.clone());
        
        assert_eq!(req.mode(), UserMode::Terminal);
        assert_eq!(req.command(), &cmd);
        assert_eq!(req.context(), &ctx);
    }

    #[test]
    fn test_user_request_interactive() {
        let ctx = UserCommandContext { user_workspace: UserWorkspace::Pwd };
        let cmd = UserCommandGc;
        let req = UserRequest::interactive(cmd, ctx);
        
        assert_eq!(req.mode(), UserMode::Interactive);
    }

    #[test]
    fn test_user_request_into_parts() {
        let ctx = UserCommandContext { user_workspace: UserWorkspace::Pwd };
        let cmd = UserCommandDoctor;
        let req = UserRequest::terminal(cmd, ctx.clone());
        
        let (extracted_cmd, extracted_ctx) = req.into_parts();
        assert_eq!(extracted_ctx.user_workspace, ctx.user_workspace);
    }

    #[test]
    fn test_promises_for_generic_error() {
        let promises = promises_for_generic_error();
        assert_eq!(promises.len(), 3);
        assert!(promises.iter().any(|p| matches!(p.0, UserPromiseType::ErrorsAreActionable)));
    }
}
