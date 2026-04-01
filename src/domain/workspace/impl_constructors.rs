use std::path::{Path, PathBuf};

use crate::domain::workspace::{
    CanonicalPath, CanonicalPathError, IdentityConfidence, IdentityReliability, IdentityStrategy,
    IdentityStrategyChain, IdentityStrategyChainError, ProjectIdentity, StructuralFingerprint,
    StructuralFingerprintError, WorkspaceFile, WorkspaceFileError, WorkspaceIdentity,
    WorkspaceIdentityValueError, WorkspacePath, WorkspacePathError, WorkspaceRelativePath,
    WorkspaceRelativePathError, WorkspaceRootRef, WorkspaceRootRefError,
};

impl WorkspacePath {
    pub fn new(path: PathBuf) -> Result<Self, WorkspacePathError> {
        if path.as_os_str().is_empty() {
            return Err(WorkspacePathError::Empty);
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl WorkspaceIdentity {
    pub fn new(value: String) -> Result<Self, WorkspaceIdentityValueError> {
        if value.trim().is_empty() {
            return Err(WorkspaceIdentityValueError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl WorkspaceRelativePath {
    pub fn new(path: PathBuf) -> Result<Self, WorkspaceRelativePathError> {
        if path.is_absolute() {
            return Err(WorkspaceRelativePathError::AbsolutePathNotAllowed { path });
        }
        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(WorkspaceRelativePathError::PathTraversalNotAllowed { path });
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl WorkspaceFile {
    pub fn new(workspace: &WorkspacePath, path: PathBuf) -> Result<Self, WorkspaceFileError> {
        if path.as_os_str().is_empty() {
            return Err(WorkspaceFileError::Empty);
        }
        if !path.starts_with(workspace.as_path()) {
            return Err(WorkspaceFileError::OutsideWorkspace {
                workspace: workspace.as_path().to_path_buf(),
                path,
            });
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl IdentityConfidence {
    pub const fn is_high(self) -> bool {
        matches!(self, Self::High)
    }
}

impl ProjectIdentity {
    pub const fn is_strong(&self) -> bool {
        matches!(self.reliability, IdentityReliability::Strong)
    }
}

impl StructuralFingerprint {
    pub fn new(value: String) -> Result<Self, StructuralFingerprintError> {
        if value.trim().is_empty() {
            return Err(StructuralFingerprintError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl IdentityStrategyChain {
    pub fn new(ordered: Vec<IdentityStrategy>) -> Result<Self, IdentityStrategyChainError> {
        if ordered.is_empty() {
            return Err(IdentityStrategyChainError::Empty);
        }

        for (index, strategy) in ordered.iter().enumerate() {
            if ordered
                .iter()
                .skip(index + 1)
                .any(|other| other == strategy)
            {
                return Err(IdentityStrategyChainError::DuplicatedStrategy);
            }
        }

        Ok(Self { ordered })
    }
}

impl WorkspaceRootRef {
    pub fn new(path: PathBuf) -> Result<Self, WorkspaceRootRefError> {
        if path.as_os_str().is_empty() {
            return Err(WorkspaceRootRefError::Empty);
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl CanonicalPath {
    pub fn new(path: PathBuf) -> Result<Self, CanonicalPathError> {
        if path.as_os_str().is_empty() {
            return Err(CanonicalPathError::Empty);
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}
