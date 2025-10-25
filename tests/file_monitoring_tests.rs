// File monitoring and event handling integration tests

use chaser::should_ignore_event;
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
fn test_file_watching_ignore_patterns() {
    let ignore_patterns = vec![
        "*.tmp".to_string(),
        "*.log".to_string(),
        ".git/**".to_string(),
        "target/**".to_string(),
        "node_modules/**".to_string(),
        "*.swp".to_string(),
        "~$".to_string(),
    ];

    let ignored_files = vec![
        "/project/temp.tmp",
        "/project/debug.log",
        "/project/.git/config",
        "/project/.git/objects/abc123",
        "/project/target/debug/main",
        "/project/target/release/app.exe",
        "/project/node_modules/package/index.js",
        "/project/file.swp",
        "/project/~$document.docx",
        "/var/log/system.log",
    ];

    for file_path in ignored_files {
        let event = create_test_event(vec![file_path], EventKind::Create(CreateKind::File));
        assert!(
            should_ignore_event(&event, &ignore_patterns),
            "File {} should be ignored but wasn't",
            file_path
        );
    }

    let not_ignored_files = vec![
        "/project/src/main.rs",
        "/project/README.md",
        "/project/Cargo.toml",
        "/project/package.json",
        "/project/index.html",
        "/project/styles.css",
        "/project/script.js",
        "/project/data.json",
        "/project/config.yaml",
    ];

    for file_path in not_ignored_files {
        let event = create_test_event(vec![file_path], EventKind::Create(CreateKind::File));
        assert!(
            !should_ignore_event(&event, &ignore_patterns),
            "File {} should not be ignored but was",
            file_path
        );
    }
}

#[test]
fn test_complex_path_patterns() {
    let ignore_patterns = vec![
        "cache".to_string(),
        "build/**".to_string(),
        "*.o".to_string(),
        "*.exe".to_string(),
        ".DS_Store".to_string(),
    ];

    let test_cases = vec![
        ("/project/cache/file.txt", true),
        ("/project/src/cache/data.json", true),
        ("/project/build/output.txt", true),
        ("/project/src/build.rs", true),
        ("/project/file.o", true),
        ("/project/app.exe", true),
        ("/project/.DS_Store", true),
        ("/project/normal.txt", false),
    ];

    for (path, should_ignore) in test_cases {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let ignored = should_ignore_event(&event, &ignore_patterns);
        assert_eq!(
            ignored, should_ignore,
            "Path {} ignore status mismatch. Expected: {}, Got: {}",
            path, should_ignore, ignored
        );
    }
}

#[test]
fn test_event_types() {
    let ignore_patterns = vec!["*.tmp".to_string()];

    let event_types = vec![
        EventKind::Create(CreateKind::File),
        EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Content,
        )),
        EventKind::Remove(notify::event::RemoveKind::File),
        EventKind::Access(notify::event::AccessKind::Read),
    ];

    for event_kind in event_types {
        let event = create_test_event(vec!["/file.tmp"], event_kind);
        assert!(should_ignore_event(&event, &ignore_patterns));

        let event = create_test_event(vec!["/file.txt"], event_kind);
        assert!(!should_ignore_event(&event, &ignore_patterns));
    }
}

#[test]
fn test_multiple_paths_in_event() {
    let ignore_patterns = vec!["*.tmp".to_string()];

    
    let event = create_test_event(
        vec!["/file1.txt", "/file2.txt", "/file3.txt"],
        EventKind::Create(CreateKind::File),
    );
    assert!(!should_ignore_event(&event, &ignore_patterns));

    
    let event = create_test_event(
        vec!["/file1.txt", "/temp.tmp", "/file3.txt"],
        EventKind::Create(CreateKind::File),
    );
    assert!(should_ignore_event(&event, &ignore_patterns));
}

#[test]
fn test_real_world_scenarios() {
    

    let rust_ignore_patterns = vec![
        "target/**".to_string(),
        "*.lock".to_string(),
        ".git/**".to_string(),
        "*.tmp".to_string(),
        "*.swp".to_string(),
    ];

    let rust_files = vec![
        ("/project/src/main.rs", false),
        ("/project/target/debug/app", true),
        ("/project/Cargo.lock", true),
        ("/project/.git/config", true),
        ("/project/README.md", false),
        ("/project/Cargo.toml", false),
    ];

    for (path, should_ignore) in rust_files {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let ignored = should_ignore_event(&event, &rust_ignore_patterns);
        assert_eq!(ignored, should_ignore, "Rust project: {}", path);
    }

    let node_ignore_patterns = vec![
        "node_modules/**".to_string(),
        "*.log".to_string(),
        ".git/**".to_string(),
        "dist/**".to_string(),
        "build/**".to_string(),
    ];

    let node_files = vec![
        ("/project/src/index.js", false),
        ("/project/node_modules/express/index.js", true),
        ("/project/package.json", false),
        ("/project/dist/bundle.js", true),
        ("/project/build/output.js", true),
        ("/project/debug.log", true),
    ];

    for (path, should_ignore) in node_files {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let ignored = should_ignore_event(&event, &node_ignore_patterns);
        assert_eq!(ignored, should_ignore, "Node.js project: {}", path);
    }
}

#[test]
fn test_case_sensitivity() {
    
    let ignore_patterns = vec![
        "*.LOG".to_string(),
        "*.Tmp".to_string(),
        "BUILD/**".to_string(),
    ];

    let test_cases = vec![
        ("/file.LOG", true),    
        ("/file.log", false),   
        ("/file.Tmp", true),    
        ("/file.tmp", false),   
        ("/BUILD/file", true),  
        ("/build/file", false), 
    ];

    for (path, should_ignore) in test_cases {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let ignored = should_ignore_event(&event, &ignore_patterns);
        assert_eq!(ignored, should_ignore, "Case sensitivity test: {}", path);
    }
}

#[test]
fn test_unicode_paths() {
    
    let ignore_patterns = vec![
        "*.临时".to_string(),
        "测试/**".to_string(),
        "*.🗑️".to_string(),
    ];

    let unicode_paths = vec![
        ("/项目/文件.临时", true),
        ("/项目/测试/数据.txt", true),
        ("/项目/垃圾.🗑️", true),
        ("/项目/正常文件.txt", false),
        ("/project/файл.tmp", false),
    ];

    for (path, should_ignore) in unicode_paths {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let ignored = should_ignore_event(&event, &ignore_patterns);
        assert_eq!(ignored, should_ignore, "Unicode path test: {}", path);
    }
}

#[test]
fn test_performance_with_many_patterns() {
    
    let mut ignore_patterns = Vec::new();

    
    for i in 0..1000 {
        ignore_patterns.push(format!("*.tmp{}", i));
        ignore_patterns.push(format!("cache{}/**", i));
    }

    
    let event = create_test_event(
        vec!["/project/src/main.rs"],
        EventKind::Create(CreateKind::File),
    );
    let start = std::time::Instant::now();
    let ignored = should_ignore_event(&event, &ignore_patterns);
    let duration = start.elapsed();

    assert!(!ignored);
    
    assert!(
        duration.as_millis() < 100,
        "Pattern matching took too long: {:?}",
        duration
    );

    
    let event = create_test_event(
        vec!["/project/file.tmp0"],
        EventKind::Create(CreateKind::File),
    );
    let start = std::time::Instant::now();
    let ignored = should_ignore_event(&event, &ignore_patterns);
    let duration = start.elapsed();

    assert!(ignored);
    assert!(
        duration.as_millis() < 100,
        "Pattern matching took too long: {:?}",
        duration
    );
}

#[test]
fn test_edge_case_patterns() {
    
    let ignore_patterns = vec![
        "".to_string(),   
        "*".to_string(),  
        "**".to_string(), 
        "*.".to_string(), 
        ".".to_string(),  
        ".*".to_string(), 
    ];

    let test_paths = vec![
        "/file.txt",
        "/hidden/.config",
        "/.bashrc",
        "/normal_file",
        "/path/to/file",
    ];

    
    for path in test_paths {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let _ignored = should_ignore_event(&event, &ignore_patterns);
        
    }
}

#[test]
fn test_empty_inputs() {
    
    let empty_patterns: Vec<String> = vec![];
    let event = create_test_event(vec!["/any/file"], EventKind::Create(CreateKind::File));
    assert!(!should_ignore_event(&event, &empty_patterns));

    let patterns = vec!["*.tmp".to_string()];
    let empty_event = Event {
        kind: EventKind::Create(CreateKind::File),
        paths: vec![],
        attrs: Default::default(),
    };
    assert!(!should_ignore_event(&empty_event, &patterns));
}

#[test]
fn test_special_characters_in_paths() {
    
    let ignore_patterns = vec![
        "* *".to_string(), 
        "*[*".to_string(), 
        "*(*".to_string(), 
        "*&*".to_string(), 
    ];

    let special_paths = vec![
        "/file with spaces.txt",
        "/file[1].txt",
        "/file(copy).txt",
        "/file&backup.txt",
        "/normal_file.txt",
    ];

    for path in special_paths {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        let _ignored = should_ignore_event(&event, &ignore_patterns);
        
    }
}
