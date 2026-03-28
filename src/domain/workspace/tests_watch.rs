#[cfg(test)]
mod workspace_watch_tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use crate::domain::workspace::{
        FileChangeBatch, FileChangeEvent, FileChangeKind, FileWatchPolicy,
    };

    #[test]
    fn file_change_event_extracts_path() {
        let path = PathBuf::from("test.ts");
        let event = FileChangeEvent::Created {
            path: path.clone(),
            timestamp: SystemTime::now(),
        };
        assert_eq!(event.path(), &path);
    }

    #[test]
    fn file_change_event_identifies_kind() {
        let created = FileChangeEvent::Created {
            path: PathBuf::from("test.ts"),
            timestamp: SystemTime::now(),
        };
        assert_eq!(created.kind(), FileChangeKind::Created);

        let modified = FileChangeEvent::Modified {
            path: PathBuf::from("test.ts"),
            timestamp: SystemTime::now(),
            previous_hash: None,
        };
        assert_eq!(modified.kind(), FileChangeKind::Modified);
    }

    #[test]
    fn file_change_event_identifies_content_changes() {
        let created = FileChangeEvent::Created {
            path: PathBuf::from("test.ts"),
            timestamp: SystemTime::now(),
        };
        assert!(created.is_content_change());

        let renamed = FileChangeEvent::Renamed {
            from: PathBuf::from("old.ts"),
            to: PathBuf::from("new.ts"),
            timestamp: SystemTime::now(),
        };
        assert!(!renamed.is_content_change());
    }

    #[test]
    fn file_watch_policy_default_has_ignore_patterns() {
        let policy = FileWatchPolicy::default();
        assert!(!policy.ignore_patterns.is_empty());
        assert!(policy.recursive);
        assert!(policy.debounce_ms > 0);
    }

    #[test]
    fn file_watch_policy_filters_ignored_paths() {
        let policy = FileWatchPolicy::default();

        assert!(policy.should_ignore(&PathBuf::from("node_modules/package/index.js")));
        assert!(policy.should_ignore(&PathBuf::from("target/debug/build")));
        assert!(!policy.should_ignore(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn file_change_batch_tracks_events() {
        let mut batch = FileChangeBatch::new(vec![]);
        assert!(batch.is_empty());
        assert_eq!(batch.event_count(), 0);

        batch.events.push(FileChangeEvent::Created {
            path: PathBuf::from("test.ts"),
            timestamp: SystemTime::now(),
        });

        assert!(!batch.is_empty());
        assert_eq!(batch.event_count(), 1);
    }

    #[test]
    fn file_change_batch_lists_affected_files() {
        let batch = FileChangeBatch::new(vec![
            FileChangeEvent::Created {
                path: PathBuf::from("a.ts"),
                timestamp: SystemTime::now(),
            },
            FileChangeEvent::Modified {
                path: PathBuf::from("b.ts"),
                timestamp: SystemTime::now(),
                previous_hash: None,
            },
        ]);

        let files = batch.affected_files();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn file_change_batch_detects_deletions() {
        let batch = FileChangeBatch::new(vec![
            FileChangeEvent::Created {
                path: PathBuf::from("a.ts"),
                timestamp: SystemTime::now(),
            },
            FileChangeEvent::Deleted {
                path: PathBuf::from("b.ts"),
                timestamp: SystemTime::now(),
            },
        ]);

        assert!(batch.has_deletions());
    }

    #[test]
    fn file_change_batch_no_deletions() {
        let batch = FileChangeBatch::new(vec![FileChangeEvent::Created {
            path: PathBuf::from("a.ts"),
            timestamp: SystemTime::now(),
        }]);

        assert!(!batch.has_deletions());
    }
}
