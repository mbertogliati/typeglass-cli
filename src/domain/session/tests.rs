use std::path::PathBuf;

use super::*;

use crate::domain::workspace::WorkspaceIdentity;

#[test]
fn typestate_transitions_preserve_required_data() {
    let session = Session::<Uninitialized>::new(
        WorkspacePath(PathBuf::from("/tmp/project")),
        DaemonMode::OneShot,
    )
    .with_language(Language::Rust)
    .with_identity(ProjectIdentity {
        value: WorkspaceIdentity("project-a".to_string()),
        source: crate::domain::workspace::IdentitySource::StructuralHeuristic {
            confidence: crate::domain::workspace::IdentityConfidence::Low,
        },
        reliability: crate::domain::workspace::IdentityReliability::Weak,
    })
    .with_lsp()
    .with_interactive();

    assert!(matches!(session.state.language, Language::Rust));
    assert_eq!(session.state.identity.value.as_str(), "project-a");
}

#[test]
fn degraded_health_requires_issues() {
    let health = SessionHealth::Degraded { issues: vec![] };
    let result = health.validate();
    assert!(matches!(
        result,
        Err(SessionHealthError::MissingIssuesForDegraded)
    ));
}

#[test]
fn daemon_unresponsive_health_decides_restart() {
    let health = SessionHealth::Degraded {
        issues: vec![SessionHealthIssue::DaemonUnresponsive],
    };
    assert!(matches!(
        health.decision(),
        SessionHealthDecision::RestartDaemon
    ));
}
