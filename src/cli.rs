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
}
