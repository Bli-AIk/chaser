use crate::i18n::t;
use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("chaser")
        .about(&t("app_description"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("add").about(&t("cmd_add")).arg(
                Arg::new("path")
                    .help(&t("arg_path"))
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("remove").about(&t("cmd_remove")).arg(
                Arg::new("path")
                    .help(&t("arg_path_remove"))
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(Command::new("list").about(&t("cmd_list")))
        .subcommand(Command::new("config").about(&t("cmd_config")))
        .subcommand(
            Command::new("recursive").about(&t("cmd_recursive")).arg(
                Arg::new("enabled")
                    .help(&t("arg_recursive_enabled"))
                    .required(true)
                    .action(ArgAction::Set)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("ignore").about(&t("cmd_ignore")).arg(
                Arg::new("pattern")
                    .help(&t("arg_ignore_pattern"))
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(Command::new("reset").about(&t("cmd_reset")))
        .subcommand(
            Command::new("lang").about(&t("cmd_lang")).arg(
                Arg::new("language")
                    .help(&t("arg_language"))
                    .required(true)
                    .action(ArgAction::Set)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("add-target").about(&t("cmd_add_target")).arg(
                Arg::new("file")
                    .help(&t("arg_target_file"))
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("remove-target")
                .about(&t("cmd_remove_target"))
                .arg(
                    Arg::new("file")
                        .help(&t("arg_target_file_remove"))
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("list-targets").about(&t("cmd_list_targets")))
        .subcommand(Command::new("status").about(&t("cmd_status")))
        .subcommand(
            Command::new("sync").about(&t("cmd_sync")).arg(
                Arg::new("once")
                    .long("once")
                    .help(&t("arg_sync_once"))
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("update-path")
                .about(&t("cmd_update_path"))
                .arg(
                    Arg::new("old_path")
                        .help(&t("arg_old_path"))
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("new_path")
                        .help(&t("arg_new_path"))
                        .required(true)
                        .index(2),
                ),
        )
}

// 简化版CLI构建器，用于测试，不依赖国际化
pub fn build_test_cli() -> Command {
    Command::new("chaser")
        .about("An automated file path synchronization tool")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("add").about("Add a path to watch").arg(
                Arg::new("path")
                    .help("Path to add to watch list")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a path from watch list")
                .arg(
                    Arg::new("path")
                        .help("Path to remove from watch list")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("list").about("List all watched paths and settings"))
        .subcommand(Command::new("config").about("Show config file location"))
        .subcommand(
            Command::new("recursive")
                .about("Set recursive watching (true/false)")
                .arg(
                    Arg::new("enabled")
                        .help("Enable or disable recursive watching")
                        .required(true)
                        .action(ArgAction::Set)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("ignore").about("Add ignore pattern").arg(
                Arg::new("pattern")
                    .help("Pattern to ignore (e.g., \"*.tmp\", \".git/**\")")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(Command::new("reset").about("Reset config to default"))
        .subcommand(
            Command::new("lang").about("Set interface language").arg(
                Arg::new("language")
                    .help("Language code (en, zh-cn)")
                    .required(true)
                    .action(ArgAction::Set)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("add-target")
                .about("Add a target file for path synchronization")
                .arg(
                    Arg::new("file")
                        .help("Target file path (json, yaml, toml, csv)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("remove-target")
                .about("Remove a target file")
                .arg(
                    Arg::new("file")
                        .help("Target file path to remove")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("list-targets").about("List all target files"))
        .subcommand(Command::new("status").about("Show path synchronization status"))
        .subcommand(
            Command::new("sync")
                .about("Start path synchronization monitoring")
                .arg(
                    Arg::new("once")
                        .long("once")
                        .help("Perform one-time sync without monitoring")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("update-path")
                .about("Manually update a path in target files")
                .arg(
                    Arg::new("old_path")
                        .help("Old path to replace")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("new_path")
                        .help("New path to replace with")
                        .required(true)
                        .index(2),
                ),
        )
}

#[derive(Debug)]
pub enum Commands {
    Add { path: String },
    Remove { path: String },
    List,
    Config,
    Recursive { enabled: String },
    Ignore { pattern: String },
    Reset,
    Lang { language: String },
    AddTarget { file: String },
    RemoveTarget { file: String },
    ListTargets,
    Status,
    Sync { once: bool },
    UpdatePath { old_path: String, new_path: String },
}

pub fn parse_command(matches: &clap::ArgMatches) -> Option<Commands> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path").unwrap().clone();
            Some(Commands::Add { path })
        }
        Some(("remove", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path").unwrap().clone();
            Some(Commands::Remove { path })
        }
        Some(("list", _)) => Some(Commands::List),
        Some(("config", _)) => Some(Commands::Config),
        Some(("recursive", sub_matches)) => {
            let enabled = sub_matches.get_one::<String>("enabled").unwrap().clone();
            Some(Commands::Recursive { enabled })
        }
        Some(("ignore", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("pattern").unwrap().clone();
            Some(Commands::Ignore { pattern })
        }
        Some(("reset", _)) => Some(Commands::Reset),
        Some(("lang", sub_matches)) => {
            let language = sub_matches.get_one::<String>("language").unwrap().clone();
            Some(Commands::Lang { language })
        }
        Some(("add-target", sub_matches)) => {
            let file = sub_matches.get_one::<String>("file").unwrap().clone();
            Some(Commands::AddTarget { file })
        }
        Some(("remove-target", sub_matches)) => {
            let file = sub_matches.get_one::<String>("file").unwrap().clone();
            Some(Commands::RemoveTarget { file })
        }
        Some(("list-targets", _)) => Some(Commands::ListTargets),
        Some(("status", _)) => Some(Commands::Status),
        Some(("sync", sub_matches)) => {
            let once = sub_matches.get_flag("once");
            Some(Commands::Sync { once })
        }
        Some(("update-path", sub_matches)) => {
            let old_path = sub_matches.get_one::<String>("old_path").unwrap().clone();
            let new_path = sub_matches.get_one::<String>("new_path").unwrap().clone();
            Some(Commands::UpdatePath { old_path, new_path })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试辅助函数：使用简化版CLI构建器
    fn setup_test_cli() -> Command {
        build_test_cli()
    }

    #[test]
    fn test_cli_no_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser"]).unwrap();
        assert!(parse_command(&matches).is_none());
    }

    #[test]
    fn test_add_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "add", "/path/to/watch"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Add { path }) => {
                assert_eq!(path, "/path/to/watch");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_remove_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "remove", "/path/to/remove"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Remove { path }) => {
                assert_eq!(path, "/path/to/remove");
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_list_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser", "list"]).unwrap();
        match parse_command(&matches) {
            Some(Commands::List) => {}
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_config_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser", "config"]).unwrap();
        match parse_command(&matches) {
            Some(Commands::Config) => {}
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_recursive_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "recursive", "true"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Recursive { enabled }) => {
                assert_eq!(enabled, "true");
            }
            _ => panic!("Expected Recursive command"),
        }

        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "recursive", "false"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Recursive { enabled }) => {
                assert_eq!(enabled, "false");
            }
            _ => panic!("Expected Recursive command"),
        }
    }

    #[test]
    fn test_ignore_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "ignore", "*.tmp"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Ignore { pattern }) => {
                assert_eq!(pattern, "*.tmp");
            }
            _ => panic!("Expected Ignore command"),
        }
    }

    #[test]
    fn test_reset_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser", "reset"]).unwrap();
        match parse_command(&matches) {
            Some(Commands::Reset) => {}
            _ => panic!("Expected Reset command"),
        }
    }

    #[test]
    fn test_lang_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "lang", "zh-cn"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Lang { language }) => {
                assert_eq!(language, "zh-cn");
            }
            _ => panic!("Expected Lang command"),
        }
    }

    #[test]
    fn test_add_target_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "add-target", "config.json"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::AddTarget { file }) => {
                assert_eq!(file, "config.json");
            }
            _ => panic!("Expected AddTarget command"),
        }
    }

    #[test]
    fn test_remove_target_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "remove-target", "config.json"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::RemoveTarget { file }) => {
                assert_eq!(file, "config.json");
            }
            _ => panic!("Expected RemoveTarget command"),
        }
    }

    #[test]
    fn test_list_targets_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "list-targets"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::ListTargets) => {}
            _ => panic!("Expected ListTargets command"),
        }
    }

    #[test]
    fn test_status_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser", "status"]).unwrap();
        match parse_command(&matches) {
            Some(Commands::Status) => {}
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_sync_command() {
        let cli = setup_test_cli();
        let matches = cli.try_get_matches_from(&["chaser", "sync"]).unwrap();
        match parse_command(&matches) {
            Some(Commands::Sync { once }) => {
                assert!(!once);
            }
            _ => panic!("Expected Sync command"),
        }

        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "sync", "--once"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Sync { once }) => {
                assert!(once);
            }
            _ => panic!("Expected Sync command with once flag"),
        }
    }

    #[test]
    fn test_update_path_command() {
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "update-path", "/old/path", "/new/path"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::UpdatePath { old_path, new_path }) => {
                assert_eq!(old_path, "/old/path");
                assert_eq!(new_path, "/new/path");
            }
            _ => panic!("Expected UpdatePath command"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_args() {
        let cli = setup_test_cli();

        // Test Add command without path
        let result = cli.try_get_matches_from(&["chaser", "add"]);
        assert!(result.is_err());

        // Test Remove command without path
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "remove"]);
        assert!(result.is_err());

        // Test Recursive command without enabled flag
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "recursive"]);
        assert!(result.is_err());

        // Test Ignore command without pattern
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "ignore"]);
        assert!(result.is_err());

        // Test Lang command without language
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "lang"]);
        assert!(result.is_err());

        // Test UpdatePath command without paths
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "update-path"]);
        assert!(result.is_err());

        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "update-path", "/old/path"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "--help"]);
        assert!(result.is_err()); // Help flag causes parse to "fail" but shows help
    }

    #[test]
    fn test_version_flag() {
        let cli = setup_test_cli();
        let result = cli.try_get_matches_from(&["chaser", "--version"]);
        assert!(result.is_err()); // Version flag causes parse to "fail" but shows version
    }

    #[test]
    fn test_commands_with_special_characters() {
        // Test paths with spaces and special characters
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "add", "/path with spaces/test"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Add { path }) => {
                assert_eq!(path, "/path with spaces/test");
            }
            _ => panic!("Expected Add command"),
        }

        // Test ignore patterns with special characters
        let cli = setup_test_cli();
        let matches = cli
            .try_get_matches_from(&["chaser", "ignore", "*.log*"])
            .unwrap();
        match parse_command(&matches) {
            Some(Commands::Ignore { pattern }) => {
                assert_eq!(pattern, "*.log*");
            }
            _ => panic!("Expected Ignore command"),
        }
    }

    #[test]
    fn test_recursive_various_values() {
        let test_cases = vec![
            "true", "false", "1", "0", "yes", "no", "on", "off", "invalid",
        ];

        for value in test_cases {
            let cli = setup_test_cli();
            let result = cli.try_get_matches_from(&["chaser", "recursive", value]);
            assert!(
                result.is_ok(),
                "Failed to parse recursive with value: {}",
                value
            );

            match parse_command(&result.unwrap()) {
                Some(Commands::Recursive { enabled }) => {
                    assert_eq!(enabled, value);
                }
                _ => panic!("Expected Recursive command for value: {}", value),
            }
        }
    }
}
