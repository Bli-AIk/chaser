use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chaser")]
#[command(about = "An automated file path synchronization tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a path to watch
    Add {
        /// Path to add to watch list
        path: String,
    },
    /// Remove a path from watch list
    Remove {
        /// Path to remove from watch list
        path: String,
    },
    /// List all watched paths and settings
    List,
    /// Show config file location
    Config,
    /// Set recursive watching (true/false)
    Recursive {
        /// Enable or disable recursive watching
        #[arg(action = clap::ArgAction::Set)]
        enabled: String,
    },
    /// Add ignore pattern
    Ignore {
        /// Pattern to ignore (e.g., "*.tmp", ".git/**")
        pattern: String,
    },
    /// Reset config to default
    Reset,
    /// Set interface language
    Lang {
        /// Language code (en, zh-cn)
        #[arg(action = clap::ArgAction::Set)]
        language: String,
    },
    /// Add a target file for path synchronization
    AddTarget {
        /// Target file path (json, yaml, toml, csv)
        file: String,
    },
    /// Remove a target file
    RemoveTarget {
        /// Target file path to remove
        file: String,
    },
    /// List all target files
    ListTargets,
    /// Show path synchronization status
    Status,
    /// Start path synchronization monitoring
    Sync {
        /// Perform one-time sync without monitoring
        #[arg(long)]
        once: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_no_command() {
        let cli = Cli::try_parse_from(&["chaser"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_add_command() {
        let cli = Cli::try_parse_from(&["chaser", "add", "/path/to/watch"]).unwrap();
        match cli.command {
            Some(Commands::Add { path }) => {
                assert_eq!(path, "/path/to/watch");
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_remove_command() {
        let cli = Cli::try_parse_from(&["chaser", "remove", "/path/to/remove"]).unwrap();
        match cli.command {
            Some(Commands::Remove { path }) => {
                assert_eq!(path, "/path/to/remove");
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_list_command() {
        let cli = Cli::try_parse_from(&["chaser", "list"]).unwrap();
        match cli.command {
            Some(Commands::List) => {}
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_config_command() {
        let cli = Cli::try_parse_from(&["chaser", "config"]).unwrap();
        match cli.command {
            Some(Commands::Config) => {}
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_recursive_command() {
        let cli = Cli::try_parse_from(&["chaser", "recursive", "true"]).unwrap();
        match cli.command {
            Some(Commands::Recursive { enabled }) => {
                assert_eq!(enabled, "true");
            }
            _ => panic!("Expected Recursive command"),
        }

        let cli = Cli::try_parse_from(&["chaser", "recursive", "false"]).unwrap();
        match cli.command {
            Some(Commands::Recursive { enabled }) => {
                assert_eq!(enabled, "false");
            }
            _ => panic!("Expected Recursive command"),
        }
    }

    #[test]
    fn test_ignore_command() {
        let cli = Cli::try_parse_from(&["chaser", "ignore", "*.tmp"]).unwrap();
        match cli.command {
            Some(Commands::Ignore { pattern }) => {
                assert_eq!(pattern, "*.tmp");
            }
            _ => panic!("Expected Ignore command"),
        }
    }

    #[test]
    fn test_reset_command() {
        let cli = Cli::try_parse_from(&["chaser", "reset"]).unwrap();
        match cli.command {
            Some(Commands::Reset) => {}
            _ => panic!("Expected Reset command"),
        }
    }

    #[test]
    fn test_lang_command() {
        let cli = Cli::try_parse_from(&["chaser", "lang", "zh-cn"]).unwrap();
        match cli.command {
            Some(Commands::Lang { language }) => {
                assert_eq!(language, "zh-cn");
            }
            _ => panic!("Expected Lang command"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let result = Cli::try_parse_from(&["chaser", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_args() {
        // Test Add command without path
        let result = Cli::try_parse_from(&["chaser", "add"]);
        assert!(result.is_err());

        // Test Remove command without path
        let result = Cli::try_parse_from(&["chaser", "remove"]);
        assert!(result.is_err());

        // Test Recursive command without enabled flag
        let result = Cli::try_parse_from(&["chaser", "recursive"]);
        assert!(result.is_err());

        // Test Ignore command without pattern
        let result = Cli::try_parse_from(&["chaser", "ignore"]);
        assert!(result.is_err());

        // Test Lang command without language
        let result = Cli::try_parse_from(&["chaser", "lang"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let result = Cli::try_parse_from(&["chaser", "--help"]);
        assert!(result.is_err()); // Help flag causes parse to "fail" but shows help
    }

    #[test]
    fn test_version_flag() {
        let result = Cli::try_parse_from(&["chaser", "--version"]);
        assert!(result.is_err()); // Version flag causes parse to "fail" but shows version
    }

    #[test]
    fn test_commands_with_special_characters() {
        // Test paths with spaces and special characters
        let cli = Cli::try_parse_from(&["chaser", "add", "/path with spaces/test"]).unwrap();
        match cli.command {
            Some(Commands::Add { path }) => {
                assert_eq!(path, "/path with spaces/test");
            }
            _ => panic!("Expected Add command"),
        }

        // Test ignore patterns with special characters
        let cli = Cli::try_parse_from(&["chaser", "ignore", "*.log*"]).unwrap();
        match cli.command {
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
            let result = Cli::try_parse_from(&["chaser", "recursive", value]);
            assert!(
                result.is_ok(),
                "Failed to parse recursive with value: {}",
                value
            );

            match result.unwrap().command {
                Some(Commands::Recursive { enabled }) => {
                    assert_eq!(enabled, value);
                }
                _ => panic!("Expected Recursive command for value: {}", value),
            }
        }
    }
}
