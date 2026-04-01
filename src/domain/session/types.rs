use crate::domain::daemon::DaemonMode;
use crate::domain::language::Language;
use crate::domain::workspace::{ProjectIdentity, WorkspacePath};
use thiserror::Error;

pub trait SessionState: sealed::Sealed {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uninitialized;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageDetected {
    pub language: Language,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceVerified {
    pub language: Language,
    pub identity: ProjectIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspReady {
    pub language: Language,
    pub identity: ProjectIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractiveReady {
    pub language: Language,
    pub identity: ProjectIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session<S: SessionState> {
    pub workspace: WorkspacePath,
    pub daemon_mode: DaemonMode,
    pub state: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionHealth {
    Healthy,
    Degraded { issues: Vec<SessionHealthIssue> },
    Unhealthy { issues: Vec<SessionHealthIssue> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionHealthIssue {
    WorkspaceUnavailable,
    IdentityUnverifiable,
    DaemonUnresponsive,
    LspUnavailable,
    InteractiveBackpressure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionHealthDecision {
    ReuseDaemon,
    RestartDaemon,
    ContinueDegraded,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionHealthReport<S: SessionState> {
    pub session: Session<S>,
    pub health: SessionHealth,
}

#[derive(Debug, Error)]
pub enum SessionHealthError {
    #[error("Degraded health state must include at least one issue")]
    MissingIssuesForDegraded,
    #[error("Unhealthy state must include at least one issue")]
    MissingIssuesForUnhealthy,
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Uninitialized {}
    impl Sealed for super::LanguageDetected {}
    impl Sealed for super::WorkspaceVerified {}
    impl Sealed for super::LspReady {}
    impl Sealed for super::InteractiveReady {}
}
