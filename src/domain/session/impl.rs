use crate::domain::daemon::DaemonMode;
use crate::domain::language::Language;
use crate::domain::workspace::{ProjectIdentity, WorkspacePath};

use super::types::{
    InteractiveReady, LanguageDetected, LspReady, Session, SessionHealth, SessionHealthDecision,
    SessionHealthError, SessionHealthIssue, SessionHealthReport, SessionState, Uninitialized,
    WorkspaceVerified,
};

impl SessionState for Uninitialized {}
impl SessionState for LanguageDetected {}
impl SessionState for WorkspaceVerified {}
impl SessionState for LspReady {}
impl SessionState for InteractiveReady {}

impl Session<Uninitialized> {
    pub fn new(workspace: WorkspacePath, daemon_mode: DaemonMode) -> Self {
        Self {
            workspace,
            daemon_mode,
            state: Uninitialized,
        }
    }

    pub fn with_language(self, language: Language) -> Session<LanguageDetected> {
        Session {
            workspace: self.workspace,
            daemon_mode: self.daemon_mode,
            state: LanguageDetected { language },
        }
    }
}

impl Session<LanguageDetected> {
    pub fn with_identity(self, identity: ProjectIdentity) -> Session<WorkspaceVerified> {
        Session {
            workspace: self.workspace,
            daemon_mode: self.daemon_mode,
            state: WorkspaceVerified {
                language: self.state.language,
                identity,
            },
        }
    }
}

impl Session<WorkspaceVerified> {
    pub fn with_lsp(self) -> Session<LspReady> {
        Session {
            workspace: self.workspace,
            daemon_mode: self.daemon_mode,
            state: LspReady {
                language: self.state.language,
                identity: self.state.identity,
            },
        }
    }
}

impl Session<LspReady> {
    pub fn with_interactive(self) -> Session<InteractiveReady> {
        Session {
            workspace: self.workspace,
            daemon_mode: self.daemon_mode,
            state: InteractiveReady {
                language: self.state.language,
                identity: self.state.identity,
            },
        }
    }
}

impl SessionHealth {
    pub fn validate(&self) -> Result<(), SessionHealthError> {
        match self {
            SessionHealth::Healthy => Ok(()),
            SessionHealth::Degraded { issues } if issues.is_empty() => {
                Err(SessionHealthError::MissingIssuesForDegraded)
            }
            SessionHealth::Unhealthy { issues } if issues.is_empty() => {
                Err(SessionHealthError::MissingIssuesForUnhealthy)
            }
            _ => Ok(()),
        }
    }

    pub fn decision(&self) -> SessionHealthDecision {
        match self {
            SessionHealth::Healthy => SessionHealthDecision::ReuseDaemon,
            SessionHealth::Degraded { issues }
                if issues
                    .iter()
                    .any(|issue| matches!(issue, SessionHealthIssue::DaemonUnresponsive)) =>
            {
                SessionHealthDecision::RestartDaemon
            }
            SessionHealth::Degraded { .. } => SessionHealthDecision::ContinueDegraded,
            SessionHealth::Unhealthy { .. } => SessionHealthDecision::Abort,
        }
    }
}

impl<S: SessionState> SessionHealthReport<S> {
    pub fn new(session: Session<S>, health: SessionHealth) -> Result<Self, SessionHealthError> {
        health.validate()?;
        Ok(Self { session, health })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
