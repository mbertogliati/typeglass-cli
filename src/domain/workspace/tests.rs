use std::path::PathBuf;

use crate::domain::workspace::*;

#[test]
fn relative_path_rejects_parent_dir() {
    let result = WorkspaceRelativePath::new(PathBuf::from("src/../secrets.txt"));
    assert!(matches!(
        result,
        Err(WorkspaceRelativePathError::PathTraversalNotAllowed { .. })
    ));
}

#[test]
fn identity_strategy_chain_rejects_duplicates() {
    let result =
        IdentityStrategyChain::new(vec![IdentityStrategy::GitHead, IdentityStrategy::GitHead]);
    assert!(matches!(
        result,
        Err(IdentityStrategyChainError::DuplicatedStrategy)
    ));
}

#[test]
fn workspace_root_ref_rejects_empty_path() {
    let result = WorkspaceRootRef::new(PathBuf::new());
    assert!(matches!(result, Err(WorkspaceRootRefError::Empty)));
}

#[test]
fn canonical_path_rejects_empty_path() {
    let result = CanonicalPath::new(PathBuf::new());
    assert!(matches!(result, Err(CanonicalPathError::Empty)));
}

#[test]
fn workspace_path_rejects_empty_path() {
    let result = WorkspacePath::new(PathBuf::new());
    assert!(matches!(result, Err(WorkspacePathError::Empty)));
}

#[test]
fn workspace_file_rejects_empty_path() {
    let workspace = WorkspacePath(PathBuf::from("/tmp/workspace"));
    let result = WorkspaceFile::new(&workspace, PathBuf::new());
    assert!(matches!(result, Err(WorkspaceFileError::Empty)));
}
