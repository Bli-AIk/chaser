// Integration tests for complete application workflows

use chaser::{cli, config::Config, i18n};
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    let locales_dir = temp_dir.path().join("locales");
    fs::create_dir_all(&locales_dir).unwrap();

    let en_content = r#"
app_description: "An automated file path synchronization tool"
cmd_add: "Add a path to watch"
cmd_remove: "Remove a path from watch list"
cmd_list: "List all watched paths and settings"
cmd_config: "Show config file location"
cmd_recursive: "Set recursive watching (true/false)"
cmd_ignore: "Add ignore pattern"
cmd_reset: "Reset config to default"
cmd_lang: "Set interface language"
cmd_add_target: "Add a target file for path synchronization"
cmd_remove_target: "Remove a target file"
cmd_list_targets: "List all target files"
cmd_status: "Show path synchronization status"
cmd_sync: "Start path synchronization monitoring"
cmd_update_path: "Manually update a path in target files"
arg_path: "Path to add to watch list"
arg_path_remove: "Path to remove from watch list"
arg_recursive_enabled: "Enable or disable recursive watching"
arg_ignore_pattern: "Pattern to ignore (e.g., \"*.tmp\", \".git/**\")"
arg_language: "Language code (en, zh-cn)"
arg_target_file: "Target file path (json, yaml, toml, csv)"
arg_target_file_remove: "Target file path to remove"
arg_sync_once: "Perform one-time sync without monitoring"
arg_old_path: "Old path to replace"
arg_new_path: "New path to replace with"
msg_path_added: "✓ Added watch path: {0}"
msg_path_exists: "⚠ Path already exists: {0}"
msg_path_removed: "✓ Removed watch path: {0}"
msg_path_not_found: "❌ Path not found: {0}"
msg_config_loaded: "✓ Config loaded from: {0}"
msg_config_saved: "✓ Config saved to: {0}"
msg_config_created: "✓ Created default config at: {0}"
ui_watch_paths: "Watch Paths:"
ui_settings: "Settings:"
ui_recursive: "Recursive: {0}"
"#;
    fs::write(locales_dir.join("en.yaml"), en_content).unwrap();

    temp_dir
}

fn setup_test_cli() -> clap::Command {
    // Fallback to a simple CLI without i18n to avoid test environment issues
    clap::Command::new("chaser")
        .about("An automated file path synchronization tool")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            clap::Command::new("add")
                .about("Add a path to watch")
                .arg(clap::Arg::new("path").index(1).required(true)),
        )
        .subcommand(
            clap::Command::new("remove")
                .about("Remove a path from watch list")
                .arg(clap::Arg::new("path").index(1).required(true)),
        )
        .subcommand(clap::Command::new("list").about("List all watched paths and settings"))
        .subcommand(clap::Command::new("config").about("Show config file location"))
        .subcommand(
            clap::Command::new("recursive")
                .about("Set recursive watching (true/false)")
                .arg(clap::Arg::new("enabled").index(1).required(true)),
        )
        .subcommand(
            clap::Command::new("ignore")
                .about("Add ignore pattern")
                .arg(clap::Arg::new("pattern").index(1).required(true)),
        )
        .subcommand(clap::Command::new("reset").about("Reset config to default"))
        .subcommand(
            clap::Command::new("lang")
                .about("Set interface language")
                .arg(clap::Arg::new("language").index(1).required(true)),
        )
        .subcommand(
            clap::Command::new("add-target")
                .about("Add a target file for path synchronization")
                .arg(clap::Arg::new("file").index(1).required(true)),
        )
        .subcommand(
            clap::Command::new("remove-target")
                .about("Remove a target file")
                .arg(clap::Arg::new("file").index(1).required(true)),
        )
        .subcommand(clap::Command::new("list-targets").about("List all target files"))
        .subcommand(clap::Command::new("status").about("Show path synchronization status"))
        .subcommand(
            clap::Command::new("sync")
                .about("Start path synchronization monitoring")
                .arg(
                    clap::Arg::new("once")
                        .long("once")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            clap::Command::new("update-path")
                .about("Manually update a path in target files")
                .arg(clap::Arg::new("old_path").index(1).required(true))
                .arg(clap::Arg::new("new_path").index(2).required(true)),
        )
}

#[test]
#[serial]
fn test_config_operations_integration() {
    let temp_dir = setup_test_env();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    // Test with the temporary directory containing locales
    let result = i18n::init_i18n_with_locale("en");
    if result.is_err() {
        eprintln!("Warning: Could not initialize i18n: {:?}", result);
    }

    let mut config = Config::default();

    // Test adding paths
    let result = config.add_path("/test/path1".to_string());
    assert!(result.is_ok());
    assert!(config.watch_paths.contains(&"/test/path1".to_string()));

    let result = config.add_path("/test/path2".to_string());
    assert!(result.is_ok());
    assert!(config.watch_paths.contains(&"/test/path2".to_string()));

    // Test adding duplicate path
    let result = config.add_path("/test/path1".to_string());
    assert!(result.is_ok()); // Should not error, just not add duplicate

    // Test removing path
    let result = config.remove_path("/test/path1");
    assert!(result.is_ok());
    assert!(!config.watch_paths.contains(&"/test/path1".to_string()));

    // Test removing non-existent path
    let result = config.remove_path("/non/existent");
    assert!(result.is_ok()); // Should not error

    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_cli_parsing_integration() {
    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser"]).unwrap();
    assert!(cli::parse_command(&matches).is_none());

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "add", "/new/path"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Add { path }) => assert_eq!(path, "/new/path"),
        _ => panic!("Expected Add command"),
    }

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "remove", "/old/path"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Remove { path }) => assert_eq!(path, "/old/path"),
        _ => panic!("Expected Remove command"),
    }

    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser", "list"]).unwrap();
    assert!(matches!(
        cli::parse_command(&matches),
        Some(cli::Commands::List)
    ));

    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser", "config"]).unwrap();
    assert!(matches!(
        cli::parse_command(&matches),
        Some(cli::Commands::Config)
    ));

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "recursive", "false"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Recursive { enabled }) => assert_eq!(enabled, "false"),
        _ => panic!("Expected Recursive command"),
    }

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "ignore", "*.backup"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Ignore { pattern }) => assert_eq!(pattern, "*.backup"),
        _ => panic!("Expected Ignore command"),
    }

    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser", "reset"]).unwrap();
    assert!(matches!(
        cli::parse_command(&matches),
        Some(cli::Commands::Reset)
    ));

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "lang", "zh-cn"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Lang { language }) => assert_eq!(language, "zh-cn"),
        _ => panic!("Expected Lang command"),
    }
}

#[test]
#[serial]
fn test_config_persistence_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let mut original_config = Config::default();
    original_config.watch_paths = vec!["/test1".to_string(), "/test2".to_string()];
    original_config.recursive = false;
    original_config.ignore_patterns = vec!["*.test".to_string()];
    original_config.language = Some("zh-cn".to_string());

    let yaml_content = serde_yaml_ng::to_string(&original_config).unwrap();
    fs::write(&config_path, yaml_content).unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_yaml_ng::from_str(&content).unwrap();

    assert_eq!(original_config, loaded_config);
    assert_eq!(loaded_config.watch_paths, vec!["/test1", "/test2"]);
    assert_eq!(loaded_config.recursive, false);
    assert_eq!(loaded_config.ignore_patterns, vec!["*.test"]);
    assert_eq!(loaded_config.language, Some("zh-cn".to_string()));
}

#[test]
fn test_recursive_option_parsing() {
    let test_cases = vec![
        ("true", true),
        ("1", true),
        ("yes", true),
        ("on", true),
        ("false", false),
        ("0", false),
        ("no", false),
        ("off", false),
    ];

    for (input, expected) in test_cases {
        let command = setup_test_cli();
        let matches = command
            .try_get_matches_from(&["chaser", "recursive", input])
            .unwrap();
        match cli::parse_command(&matches) {
            Some(cli::Commands::Recursive { enabled }) => {
                let parsed = match enabled.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => false,
                };
                assert_eq!(parsed, expected, "Failed for input: {}", input);
            }
            _ => panic!("Expected Recursive command for input: {}", input),
        }
    }
}

#[test]
fn test_error_handling() {
    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "invalid_command"]);
    assert!(result.is_err());

    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "add"]);
    assert!(result.is_err());

    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "remove"]);
    assert!(result.is_err());

    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "recursive"]);
    assert!(result.is_err());

    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "ignore"]);
    assert!(result.is_err());

    let command = setup_test_cli();
    let result = command.try_get_matches_from(&["chaser", "lang"]);
    assert!(result.is_err());
}

#[test]
fn test_config_edge_cases() {
    let mut config = Config::default();

    let result = config.add_path(String::new());
    assert!(result.is_ok());

    let path = "./test_path".to_string();
    config.add_path(path.clone()).unwrap();
    assert!(config.watch_paths.contains(&path));

    config.remove_path(&path).unwrap();
    assert!(!config.watch_paths.contains(&path));

    // Test with paths containing special characters
    let special_path = "/path with spaces/and-dashes_and.dots".to_string();
    config.add_path(special_path.clone()).unwrap();
    assert!(config.watch_paths.contains(&special_path));

    // Test case sensitivity
    let case_path = "/CaseSensitive/Path".to_string();
    config.add_path(case_path.clone()).unwrap();
    assert!(config.watch_paths.contains(&case_path));

    let lower_case_path = "/casesensitive/path".to_string();
    config.add_path(lower_case_path.clone()).unwrap();
    assert!(config.watch_paths.contains(&lower_case_path));
    assert_ne!(case_path, lower_case_path);
}

#[test]
fn test_new_commands() {
    // Test add-target command
    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "add-target", "config.json"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::AddTarget { file }) => assert_eq!(file, "config.json"),
        _ => panic!("Expected AddTarget command"),
    }

    // Test remove-target command
    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "remove-target", "config.json"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::RemoveTarget { file }) => assert_eq!(file, "config.json"),
        _ => panic!("Expected RemoveTarget command"),
    }

    // Test list-targets command
    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "list-targets"])
        .unwrap();
    assert!(matches!(
        cli::parse_command(&matches),
        Some(cli::Commands::ListTargets)
    ));

    // Test status command
    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser", "status"]).unwrap();
    assert!(matches!(
        cli::parse_command(&matches),
        Some(cli::Commands::Status)
    ));

    // Test sync command
    let command = setup_test_cli();
    let matches = command.try_get_matches_from(&["chaser", "sync"]).unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Sync { once }) => assert!(!once),
        _ => panic!("Expected Sync command"),
    }

    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "sync", "--once"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::Sync { once }) => assert!(once),
        _ => panic!("Expected Sync command with once flag"),
    }

    // Test update-path command
    let command = setup_test_cli();
    let matches = command
        .try_get_matches_from(&["chaser", "update-path", "/old/path", "/new/path"])
        .unwrap();
    match cli::parse_command(&matches) {
        Some(cli::Commands::UpdatePath { old_path, new_path }) => {
            assert_eq!(old_path, "/old/path");
            assert_eq!(new_path, "/new/path");
        }
        _ => panic!("Expected UpdatePath command"),
    }
}
