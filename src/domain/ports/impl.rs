use super::types::{WorkspacePortError, WorkspaceProbe};

impl WorkspaceProbe {
    pub fn validate(self) -> Result<Self, WorkspacePortError> {
        if self.requested_path.as_os_str().is_empty() {
            return Err(WorkspacePortError::ProbeFailed {
                path: self.requested_path,
                reason: "empty path".to_string(),
            });
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn workspace_probe_rejects_empty_path() {
        let probe = WorkspaceProbe {
            requested_path: PathBuf::new(),
        };

        let result = probe.validate();
        assert!(matches!(
            result,
            Err(WorkspacePortError::ProbeFailed { .. })
        ));
    }
}
