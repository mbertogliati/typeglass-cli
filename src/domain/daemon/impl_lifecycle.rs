use crate::domain::daemon::{
    DaemonLifecycleCommand, DaemonMode, DaemonState, DaemonTransition, DaemonTransitionError,
};

impl DaemonTransition {
    pub fn validate(&self, mode: DaemonMode) -> Result<(), DaemonTransitionError> {
        match (&self.from, self.command) {
            (DaemonState::NotRunning, DaemonLifecycleCommand::Stop) => {
                Err(DaemonTransitionError::NotRunning {
                    command: self.command,
                })
            }
            (DaemonState::Running { .. }, DaemonLifecycleCommand::Start) => {
                Err(DaemonTransitionError::AlreadyRunning {
                    command: self.command,
                })
            }
            (_, DaemonLifecycleCommand::Gc) if matches!(mode, DaemonMode::OneShot) => {
                Err(DaemonTransitionError::GcUnsupportedForOneShot {
                    command: self.command,
                })
            }
            _ => Ok(()),
        }
    }
}
