use std::path::PathBuf;

use crate::domain::daemon::*;
use crate::domain::language::Language;
use crate::domain::workspace::WorkspacePath;

#[test]
fn transition_rejects_stop_when_not_running() {
    let transition = DaemonTransition {
        from: DaemonState::NotRunning,
        command: DaemonLifecycleCommand::Stop,
    };

    let result = transition.validate(DaemonMode::Persistent);
    assert!(matches!(
        result,
        Err(DaemonTransitionError::NotRunning { .. })
    ));
}

#[test]
fn transition_rejects_start_when_running() {
    let running = DaemonState::Running {
        pid: Pid(123),
        endpoint: DaemonEndpoint::UnixSocket(SocketPath(PathBuf::from("/tmp/sock"))),
        transport: DaemonTransportKind::UnixSocket,
        language: Language::Rust,
        workspace: WorkspacePath(PathBuf::from("/tmp/workspace")),
        process_files: DaemonProcessFiles {
            pid_file: PidFilePath(PathBuf::from("/tmp/pid")),
            lock_file: LockFilePath(PathBuf::from("/tmp/lock")),
        },
    };
    let transition = DaemonTransition {
        from: running,
        command: DaemonLifecycleCommand::Start,
    };

    let result = transition.validate(DaemonMode::Persistent);
    assert!(matches!(
        result,
        Err(DaemonTransitionError::AlreadyRunning { .. })
    ));
}

#[test]
fn process_files_require_non_empty_paths() {
    let result = DaemonProcessFiles::new(PathBuf::new(), PathBuf::from("/tmp/lock"));
    assert!(matches!(result, Err(DaemonProcessFilesError::EmptyPidFile)));
}

#[test]
fn retry_jitter_rejects_values_over_100() {
    let result = RetryJitterPercent::new(101);
    assert!(matches!(result, Err(RetryJitterError::OutOfRange)));
}

#[test]
fn retry_delay_range_rejects_inverted_range() {
    let min = RetryBackoffMs::new(200).expect("min must be valid");
    let max = RetryBackoffMs::new(100).expect("max must be valid");
    let result = RetryDelayRange::new(min, max);
    assert!(matches!(result, Err(RetryDelayRangeError::InvalidRange)));
}

#[test]
fn cache_dir_rejects_empty_path() {
    let result = CacheDir::new(PathBuf::new());
    assert!(matches!(result, Err(CacheDirError::EmptyPath)));
}
