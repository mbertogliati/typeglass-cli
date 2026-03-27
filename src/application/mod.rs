use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::domain::ports::*;
use crate::ux_model::intent::{
    FromTarget, UserCommand, UserCommandType, UserGoal, UserPromise, UserWorkspace,
};
use crate::ux_model::result::*;
use thiserror::Error;

/// El servicio de aplicación que orquesta la ejecución de comandos.
pub struct ApplicationService<A: ApplicationAdapters> {
    adapters: A,
}

impl<A: ApplicationAdapters> ApplicationService<A> {
    pub fn new(adapters: A) -> Self {
        Self { adapters }
    }

    /// Método principal de ejecución. Es asíncrono, recibe un `UserCommand` 
    /// y devuelve un `UserResult` cuyos tipos dependen de la acción solicitada.
    pub async fn execute<C>(&self, command: UserCommand) -> UserResult<C::Success, C::Partial, C::Failure>
    where
        C: CommandAction,
        Self: ActionExecutor<C>,
    {
        let goal = command.goal();
        let success_promises = command.success_promises();
        let error_promises = command.error_promises();
        let context = UserResultContext { original_command: command.clone() };

        match C::try_from_command(command) {
            Ok(action) => {
                self.execute_action(action).await
            }
            Err(err_msg) => {
                UserResult::Failure(C::Failure::from_error(
                    err_msg,
                    error_promises,
                    context
                ))
            }
        }
    }
}

/// Define la asociación entre un comando y sus tipos de resultado.
pub trait CommandAction: Sized {
    type Success: SuccessUserExpectations;
    type Partial: PartialSuccessUserExpectations;
    type Failure: FailureUserExpectations + AppFailureFactory;

    fn try_from_command(cmd: UserCommand) -> Result<Self, String>;
}

/// Factory para crear errores de aplicación que cumplan con las expectativas de UX.
pub trait AppFailureFactory {
    fn from_error(message: String, promises: Vec<UserPromise>, context: UserResultContext) -> Self;
}

/// Trait para ejecutar una acción específica.
pub trait ActionExecutor<C: CommandAction> {
    fn execute_action(&self, action: C) -> impl Future<Output = UserResult<C::Success, C::Partial, C::Failure>> + Send;
}

// --- Implementación de Acciones ---

pub struct FromAction {
    pub target: FromTarget,
    pub depth: Option<u8>,
    pub workspace: UserWorkspace,
}

impl CommandAction for FromAction {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;

    fn try_from_command(cmd: UserCommand) -> Result<Self, String> {
        if let UserCommandType::From { target, depth } = cmd.command_type {
            Ok(Self { target, depth, workspace: cmd.user_workspace })
        } else {
            Err("Expected 'from' command".to_string())
        }
    }
}

pub struct GcAction {
    pub workspace: UserWorkspace,
}

impl CommandAction for GcAction {
    type Success = GenericSuccess;
    type Partial = GenericPartial;
    type Failure = GenericFailure;

    fn try_from_command(cmd: UserCommand) -> Result<Self, String> {
        if matches!(cmd.command_type, UserCommandType::Gc) {
            Ok(Self { workspace: cmd.user_workspace })
        } else {
            Err("Expected 'gc' command".to_string())
        }
    }
}

// --- Tipos de Resultado Genéricos (para empezar) ---

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

// --- Adaptadores ---

pub trait ApplicationAdapters: Send + Sync {
    type Workspace: WorkspacePort + Send + Sync;
    type FileSystem: FileSystemPort + Send + Sync;
    type Daemon: DaemonPort + Send + Sync;
    type Lsp: LspPort + Send + Sync;
    type Clock: ClockPort + Send + Sync;

    fn workspace(&self) -> &Self::Workspace;
    fn file_system(&self) -> &Self::FileSystem;
    fn daemon(&self) -> &Self::Daemon;
    fn lsp(&self) -> &Self::Lsp;
    fn clock(&self) -> &Self::Clock;
}

pub struct UnwiredAdapters;

impl ApplicationAdapters for UnwiredAdapters {
    type Workspace = UnwiredWorkspace;
    type FileSystem = UnwiredFileSystem;
    type Daemon = UnwiredDaemon;
    type Lsp = UnwiredLsp;
    type Clock = UnwiredClock;

    fn workspace(&self) -> &Self::Workspace { &UnwiredWorkspace }
    fn file_system(&self) -> &Self::FileSystem { &UnwiredFileSystem }
    fn daemon(&self) -> &Self::Daemon { &UnwiredDaemon }
    fn lsp(&self) -> &Self::Lsp { &UnwiredLsp }
    fn clock(&self) -> &Self::Clock { &UnwiredClock }
}

pub struct UnwiredWorkspace;
impl WorkspacePort for UnwiredWorkspace {
    type ProbeFuture<'a> = std::future::Ready<Result<crate::domain::ports::WorkspaceSnapshot, crate::domain::ports::WorkspacePortError>>;
    fn probe_workspace<'a>(&'a self, _probe: crate::domain::ports::WorkspaceProbe) -> Self::ProbeFuture<'a> {
        panic!("WorkspacePort is unwired")
    }
}

pub struct UnwiredFileSystem;
impl FileSystemPort for UnwiredFileSystem {
    type MetadataFuture<'a> = std::future::Ready<Result<crate::domain::ports::FileMetadataSnapshot, crate::domain::ports::FileSystemPortError>>;
    fn read_file_metadata<'a>(&'a self, _file: crate::domain::workspace::WorkspaceFile) -> Self::MetadataFuture<'a> {
        panic!("FileSystemPort is unwired")
    }
}

pub struct UnwiredDaemon;
impl DaemonPort for UnwiredDaemon {
    type StartFuture<'a> = std::future::Ready<Result<crate::domain::ports::DaemonRuntimeSnapshot, crate::domain::ports::DaemonPortError>>;
    type StopFuture<'a> = std::future::Ready<Result<(), crate::domain::ports::DaemonPortError>>;
    type StatusFuture<'a> = std::future::Ready<Result<crate::domain::ports::DaemonRuntimeSnapshot, crate::domain::ports::DaemonPortError>>;

    fn start_daemon<'a>(&'a self, _spec: crate::domain::ports::DaemonStartSpec) -> Self::StartFuture<'a> { panic!("DaemonPort is unwired") }
    fn stop_daemon<'a>(&'a self, _pid: Option<crate::domain::daemon::Pid>) -> Self::StopFuture<'a> { panic!("DaemonPort is unwired") }
    fn daemon_status<'a>(&'a self) -> Self::StatusFuture<'a> { panic!("DaemonPort is unwired") }
}

pub struct UnwiredLsp;
impl LspPort for UnwiredLsp {
    type QueryFuture<'a> = std::future::Ready<Result<crate::domain::ports::LspQueryResponse, crate::domain::ports::LspPortError>>;
    type InvalidateFuture<'a> = std::future::Ready<Result<crate::domain::lsp::InvalidationResult, crate::domain::ports::LspPortError>>;

    fn run_query<'a>(&'a self, _request: crate::domain::ports::LspQueryRequest) -> Self::QueryFuture<'a> { panic!("LspPort is unwired") }
    fn invalidate<'a>(&'a self, _request: crate::domain::lsp::InvalidationRequest) -> Self::InvalidateFuture<'a> { panic!("LspPort is unwired") }
}

pub struct UnwiredClock;
impl ClockPort for UnwiredClock {
    type NowFuture<'a> = std::future::Ready<Result<crate::domain::ports::ClockTick, crate::domain::ports::ClockPortError>>;
    type SleepFuture<'a> = std::future::Ready<Result<(), crate::domain::ports::ClockPortError>>;

    fn now<'a>(&'a self) -> Self::NowFuture<'a> { panic!("ClockPort is unwired") }
    fn sleep<'a>(&'a self, _duration: std::time::Duration) -> Self::SleepFuture<'a> { panic!("ClockPort is unwired") }
}

// --- Implementación del Executor para cada acción ---

impl<A: ApplicationAdapters> ActionExecutor<FromAction> for ApplicationService<A> {
    async fn execute_action(&self, action: FromAction) -> UserResult<GenericSuccess, GenericPartial, GenericFailure> {
        let cmd = UserCommand::from_symbol("stub".to_string(), None); // Simplificación para el stub
        UserResult::Success(GenericSuccess {
            goal: cmd.goal(),
            promises: cmd.success_promises(),
            summary: UserSummary(format!("Explored from {:?}", action.target)),
            next_step: Some(UserNextStep("Review the generated graph".to_string())),
            context: UserResultContext { original_command: cmd },
        })
    }
}

impl<A: ApplicationAdapters> ActionExecutor<GcAction> for ApplicationService<A> {
    async fn execute_action(&self, action: GcAction) -> UserResult<GenericSuccess, GenericPartial, GenericFailure> {
        let cmd = UserCommand::gc();
        UserResult::Success(GenericSuccess {
            goal: cmd.goal(),
            promises: cmd.success_promises(),
            summary: UserSummary("Garbage collection completed".to_string()),
            next_step: None,
            context: UserResultContext { original_command: cmd },
        })
    }
}
