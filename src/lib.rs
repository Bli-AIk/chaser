pub mod cli;
pub mod config;
pub mod i18n;
pub mod path_sync;
pub mod target_files;

use notify::{Event, EventKind};

/// Check if an event should be ignored based on patterns
pub fn should_ignore_event(event: &Event, ignore_patterns: &[String]) -> bool {
    event.paths.iter().any(|path| {
        let path_str = path.to_string_lossy();
        ignore_patterns
            .iter()
            .any(|pattern| matches_ignore_pattern(&path_str, pattern))
    })
}

fn matches_ignore_pattern(path: &str, pattern: &str) -> bool {
    if is_directory_pattern(pattern) {
        matches_directory_pattern(path, pattern)
    } else if is_extension_pattern(pattern) {
        matches_extension_pattern(path, pattern)
    } else {
        path.contains(pattern)
    }
}

fn is_directory_pattern(pattern: &str) -> bool {
    pattern.contains("**")
}

fn is_extension_pattern(pattern: &str) -> bool {
    pattern.starts_with("*.")
}

fn matches_directory_pattern(path: &str, pattern: &str) -> bool {
    let dir_pattern = pattern.replace("/**", "");
    path.contains(&dir_pattern)
}

fn matches_extension_pattern(path: &str, pattern: &str) -> bool {
    if let Some(ext) = pattern.strip_prefix("*.") {
        path.ends_with(ext)
    } else {
        false
    }
}

/// Convert event type to human-readable description
pub fn get_event_description(event: &Event) -> String {
    match event.kind {
        EventKind::Create(_) => "Created".to_string(),
        EventKind::Modify(_) => "Modified".to_string(),
        EventKind::Remove(_) => "Removed".to_string(),
        EventKind::Access(_) => "Accessed".to_string(),
        EventKind::Any | EventKind::Other => "Other".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{Event, EventKind, event::CreateKind};
    use std::path::PathBuf;

    fn create_test_event(paths: Vec<&str>, kind: EventKind) -> Event {
        Event {
            kind,
            paths: paths.into_iter().map(PathBuf::from).collect(),
            attrs: Default::default(),
        }
    }

    #[test]
    fn test_should_ignore_event_extension_patterns() {
        let ignore_patterns = vec!["*.tmp".to_string(), "*.log".to_string()];

        // Test matching extension
        let event = create_test_event(
            vec!["/path/to/file.tmp"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        let event = create_test_event(
            vec!["/path/to/file.log"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        // Test non-matching extension
        let event = create_test_event(
            vec!["/path/to/file.txt"],
            EventKind::Create(CreateKind::File),
        );
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_should_ignore_event_directory_patterns() {
        let ignore_patterns = vec![".git/**".to_string(), "target/**".to_string()];

        // Test matching directory
        let event = create_test_event(
            vec!["/project/.git/config"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        let event = create_test_event(
            vec!["/project/target/debug/app"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        // Test non-matching directory
        let event = create_test_event(
            vec!["/project/src/main.rs"],
            EventKind::Create(CreateKind::File),
        );
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_should_ignore_event_substring_patterns() {
        let ignore_patterns = vec!["backup".to_string(), "temp".to_string()];

        // Test matching substring
        let event = create_test_event(
            vec!["/path/to/backup_file.txt"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        let event = create_test_event(vec!["/temp/file.txt"], EventKind::Create(CreateKind::File));
        assert!(should_ignore_event(&event, &ignore_patterns));

        // Test non-matching substring
        let event = create_test_event(
            vec!["/path/to/normal_file.txt"],
            EventKind::Create(CreateKind::File),
        );
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_should_ignore_event_multiple_paths() {
        let ignore_patterns = vec!["*.tmp".to_string()];

        // Test event with multiple paths, some matching
        let event = create_test_event(
            vec!["/path/to/file.txt", "/path/to/file.tmp"],
            EventKind::Create(CreateKind::File),
        );
        assert!(should_ignore_event(&event, &ignore_patterns));

        // Test event with multiple paths, none matching
        let event = create_test_event(
            vec!["/path/to/file1.txt", "/path/to/file2.txt"],
            EventKind::Create(CreateKind::File),
        );
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_should_ignore_event_empty_patterns() {
        let ignore_patterns = vec![];

        let event = create_test_event(vec!["/any/file.txt"], EventKind::Create(CreateKind::File));
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_should_ignore_event_empty_paths() {
        let ignore_patterns = vec!["*.tmp".to_string()];

        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![],
            attrs: Default::default(),
        };
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }

    #[test]
    fn test_get_event_description() {
        let event = create_test_event(vec!["/test"], EventKind::Create(CreateKind::File));
        assert_eq!(get_event_description(&event), "Created");

        let event = create_test_event(
            vec!["/test"],
            EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
        );
        assert_eq!(get_event_description(&event), "Modified");

        let event = create_test_event(
            vec!["/test"],
            EventKind::Remove(notify::event::RemoveKind::File),
        );
        assert_eq!(get_event_description(&event), "Removed");

        let event = create_test_event(
            vec!["/test"],
            EventKind::Access(notify::event::AccessKind::Read),
        );
        assert_eq!(get_event_description(&event), "Accessed");

        let event = create_test_event(vec!["/test"], EventKind::Any);
        assert_eq!(get_event_description(&event), "Other");

        let event = create_test_event(vec!["/test"], EventKind::Other);
        assert_eq!(get_event_description(&event), "Other");
    }

    #[test]
    fn test_complex_ignore_patterns() {
        let ignore_patterns = vec![
            "*.tmp".to_string(),
            ".git/**".to_string(),
            "node_modules".to_string(),
            "*.log".to_string(),
        ];

        // Test cases that should be ignored
        let test_cases_ignored = vec![
            "/project/file.tmp",
            "/project/.git/HEAD",
            "/project/.git/objects/abc123",
            "/project/node_modules/package/index.js",
            "/project/logs/app.log",
            "/var/log/system.log",
        ];

        for path in test_cases_ignored {
            let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
            assert!(
                should_ignore_event(&event, &ignore_patterns),
                "Expected path {} to be ignored",
                path
            );
        }

        // Test cases that should not be ignored
        let test_cases_not_ignored = vec![
            "/project/src/main.rs",
            "/project/README.md",
            "/project/Cargo.toml",
            "/project/tests/test.rs",
        ];

        for path in test_cases_not_ignored {
            let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
            assert!(
                !should_ignore_event(&event, &ignore_patterns),
                "Expected path {} not to be ignored",
                path
            );
        }
    }

    #[test]
    fn test_case_sensitivity() {
        let ignore_patterns = vec!["*.TMP".to_string()];

        // Test case sensitivity (should be case sensitive)
        let event = create_test_event(vec!["/file.tmp"], EventKind::Create(CreateKind::File));
        assert!(!should_ignore_event(&event, &ignore_patterns));

        let event = create_test_event(vec!["/file.TMP"], EventKind::Create(CreateKind::File));
        assert!(should_ignore_event(&event, &ignore_patterns));
    }
}
