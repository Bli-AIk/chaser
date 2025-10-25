// Performance and benchmark tests

use chaser::{config::Config, should_ignore_event};
use notify::{Event, EventKind, event::CreateKind};
use std::path::PathBuf;
use std::time::Instant;

fn create_test_event(paths: Vec<&str>, kind: EventKind) -> Event {
    Event {
        kind,
        paths: paths.into_iter().map(PathBuf::from).collect(),
        attrs: Default::default(),
    }
}

#[test]
fn benchmark_ignore_pattern_matching() {
    let mut ignore_patterns = Vec::new();

    for i in 0..100 {
        ignore_patterns.push(format!("*.tmp{}", i));
        ignore_patterns.push(format!("*.log{}", i));
        ignore_patterns.push(format!("cache{}/**", i));
        ignore_patterns.push(format!("build{}/**", i));
        ignore_patterns.push(format!("temp{}", i));
    }

    let test_paths = vec![
        "/project/src/main.rs",
        "/project/file.tmp50",
        "/project/cache50/data.txt",
        "/project/temp25",
    ];

    let iterations = 1000;

    for path in test_paths {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = should_ignore_event(&event, &ignore_patterns);
        }
        let duration = start.elapsed();

        let avg_time = duration.as_nanos() / iterations as u128;
        println!("Path: {}, Average time per check: {} ns", path, avg_time);

        assert!(
            avg_time < 1_000_000,
            "Pattern matching too slow for path: {}, took {} ns",
            path,
            avg_time
        );
    }
}

#[test]
fn benchmark_config_operations() {
    let mut config = Config::default();
    let iterations = 10000;

    let start = Instant::now();
    for i in 0..iterations {
        let path = format!("/test/path/{}", i);
        config.add_path(path).unwrap();
    }
    let add_duration = start.elapsed();

    println!("Added {} paths in {:?}", iterations, add_duration);
    println!("Average time per add: {:?}", add_duration / iterations);

    let start = Instant::now();
    for _ in 0..100 {
        let _ = config.validate_paths();
    }
    let validate_duration = start.elapsed();

    println!("Validated paths 100 times in {:?}", validate_duration);
    println!("Average validation time: {:?}", validate_duration / 100);

    let start = Instant::now();
    for i in 0..iterations {
        let path = format!("/test/path/{}", i);
        config.remove_path(&path).unwrap();
    }
    let remove_duration = start.elapsed();

    println!("Removed {} paths in {:?}", iterations, remove_duration);
    println!(
        "Average time per remove: {:?}",
        remove_duration / iterations
    );
}

#[test]
fn benchmark_serialization() {
    let mut config = Config::default();

    for i in 0..1000 {
        config.watch_paths.push(format!("/test/path/{}", i));
        config.ignore_patterns.push(format!("*.tmp{}", i));
    }

    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = serde_yaml_ng::to_string(&config).unwrap();
    }
    let serialize_duration = start.elapsed();

    println!(
        "Serialized config {} times in {:?}",
        iterations, serialize_duration
    );
    println!(
        "Average serialization time: {:?}",
        serialize_duration / iterations
    );

    let yaml_content = serde_yaml_ng::to_string(&config).unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _: Config = serde_yaml_ng::from_str(&yaml_content).unwrap();
    }
    let deserialize_duration = start.elapsed();

    println!(
        "Deserialized config {} times in {:?}",
        iterations, deserialize_duration
    );
    println!(
        "Average deserialization time: {:?}",
        deserialize_duration / iterations
    );
}

#[test]
fn test_memory_usage() {
    let mut config = Config::default();

    for i in 0..10000 {
        config.watch_paths.push(format!(
            "/very/long/path/to/test/directory/number/{}/with/many/subdirectories/and/files",
            i
        ));
        config
            .ignore_patterns
            .push(format!("*.very_long_extension_name_{}", i));
    }

    let yaml_content = serde_yaml_ng::to_string(&config).unwrap();
    let size_kb = yaml_content.len() / 1024;
    println!("Serialized config size: {} KB", size_kb);

    let start = Instant::now();
    let _loaded_config: Config = serde_yaml_ng::from_str(&yaml_content).unwrap();
    let load_time = start.elapsed();

    println!("Loaded large config in {:?}", load_time);

    assert!(
        load_time.as_secs() < 1,
        "Config loading took too long: {:?}",
        load_time
    );
}

#[test]
fn stress_test_pattern_matching() {
    let mut ignore_patterns = Vec::new();

    for i in 0..500 {
        ignore_patterns.push(format!("*.ext{}", i));
        ignore_patterns.push(format!("dir{}/**", i));
        ignore_patterns.push(format!("*file{}", i));
        ignore_patterns.push(format!("prefix{}*", i));
    }

    let mut test_paths = Vec::new();
    for i in 0..1000 {
        test_paths.push(format!("/project/file{}.txt", i));
        test_paths.push(format!("/project/dir{}/subfile.txt", i));
        test_paths.push(format!("/project/data.ext{}", i));
        test_paths.push(format!("/project/prefix{}suffix.txt", i));
    }

    let start = Instant::now();
    let mut matched_count = 0;

    for path in &test_paths {
        let event = create_test_event(vec![path], EventKind::Create(CreateKind::File));
        if should_ignore_event(&event, &ignore_patterns) {
            matched_count += 1;
        }
    }

    let duration = start.elapsed();

    println!(
        "Processed {} paths against {} patterns in {:?}",
        test_paths.len(),
        ignore_patterns.len(),
        duration
    );
    println!("Matched {} paths", matched_count);
    println!(
        "Average time per path: {:?}",
        duration / test_paths.len() as u32
    );

    assert!(
        duration.as_secs() < 5,
        "Stress test took too long: {:?}",
        duration
    );
}

#[test]
fn test_concurrent_access_simulation() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let config = Arc::new(Mutex::new(Config::default()));
    let mut handles = vec![];

    for i in 0..10 {
        let config_clone = Arc::clone(&config);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let path = format!("/thread{}/path{}", i, j);
                {
                    let mut cfg = config_clone.lock().unwrap();
                    cfg.add_path(path.clone()).unwrap();
                }

                std::thread::sleep(std::time::Duration::from_micros(10));

                {
                    let mut cfg = config_clone.lock().unwrap();
                    cfg.remove_path(&path).unwrap();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_config = config.lock().unwrap();
    println!(
        "Final config has {} watch paths",
        final_config.watch_paths.len()
    );

    assert_eq!(final_config.watch_paths.len(), 1);
}

#[test]
fn benchmark_different_pattern_types() {
    let extension_patterns = (0..1000).map(|i| format!("*.ext{}", i)).collect::<Vec<_>>();
    let directory_patterns = (0..1000)
        .map(|i| format!("dir{}/**", i))
        .collect::<Vec<_>>();
    let substring_patterns = (0..1000)
        .map(|i| format!("substr{}", i))
        .collect::<Vec<_>>();

    let test_path = "/project/file.ext500";
    let event = create_test_event(vec![test_path], EventKind::Create(CreateKind::File));
    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = should_ignore_event(&event, &extension_patterns);
    }
    let ext_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = should_ignore_event(&event, &directory_patterns);
    }
    let dir_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = should_ignore_event(&event, &substring_patterns);
    }
    let substr_duration = start.elapsed();

    println!("Extension patterns: {:?}", ext_duration);
    println!("Directory patterns: {:?}", dir_duration);
    println!("Substring patterns: {:?}", substr_duration);

    assert!(ext_duration.as_millis() < 1000);
    assert!(dir_duration.as_millis() < 1000);
    assert!(substr_duration.as_millis() < 1000);
}

#[test]
fn test_pathological_cases() {
    let long_path = "/".to_string() + &"very_long_directory_name/".repeat(100) + "file.txt";
    let patterns = vec!["*.txt".to_string()];
    let event = create_test_event(vec![&long_path], EventKind::Create(CreateKind::File));

    let start = Instant::now();
    let _ = should_ignore_event(&event, &patterns);
    let duration = start.elapsed();

    println!("Long path processing time: {:?}", duration);
    assert!(duration.as_millis() < 100, "Long path processing too slow");

    let similar_patterns = (0..1000)
        .map(|i| format!("very_similar_prefix_that_is_quite_long_{}.tmp", i))
        .collect::<Vec<_>>();

    let test_path = "/project/different_file.txt";
    let event = create_test_event(vec![test_path], EventKind::Create(CreateKind::File));

    let start = Instant::now();
    let _ = should_ignore_event(&event, &similar_patterns);
    let duration = start.elapsed();

    println!("Similar patterns processing time: {:?}", duration);
    assert!(
        duration.as_millis() < 100,
        "Similar patterns processing too slow"
    );

    let special_patterns = vec![
        "*.!@#$%^&*()".to_string(),
        "*[]{}<>?".to_string(),
        "*+=|\\".to_string(),
        "*`~".to_string(),
    ];

    let special_path = "/project/file.!@#$%^&*()";
    let event = create_test_event(vec![special_path], EventKind::Create(CreateKind::File));

    let start = Instant::now();
    let _ = should_ignore_event(&event, &special_patterns);
    let duration = start.elapsed();

    println!("Special characters processing time: {:?}", duration);
    assert!(
        duration.as_millis() < 100,
        "Special characters processing too slow"
    );
}
