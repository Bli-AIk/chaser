mod cli;
mod config;
mod i18n;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use i18n::{available_locales, init_i18n_with_locale, is_locale_supported, set_locale, t, tf};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use owo_colors::OwoColorize;
use std::path::Path;
use std::sync::mpsc::channel;

fn main() -> Result<()> {
    // Load config first to get language preference
    let config = Config::load().unwrap_or_default();
    let locale = config.get_effective_language();

    // Initialize i18n with the preferred language
    init_i18n_with_locale(&locale)?;

    let cli = Cli::parse();

    match cli.command {
        Some(command) => handle_command(command),
        None => run_monitor(),
    }
}

fn handle_command(command: Commands) -> Result<()> {
    let mut config = Config::load_with_i18n()?;

    match command {
        Commands::Add { path } => {
            config.add_path(path)?;
            config.save_with_i18n()?;
        }
        Commands::Remove { path } => {
            config.remove_path(&path)?;
            config.save_with_i18n()?;
        }
        Commands::List => {
            config.list_paths();
        }
        Commands::Config => {
            let config_path = Config::config_file_path()?;
            println!(
                "{}",
                tf("msg_config_location", &[&config_path.display().to_string().cyan().to_string()])
            );
            println!("{}", t("msg_config_edit_hint").bright_white());
        }
        Commands::Recursive { enabled } => {
            let enabled_bool = match enabled.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    println!("{}", tf("msg_recursive_invalid", &[&enabled]).red());
                    return Ok(());
                }
            };
            config.recursive = enabled_bool;
            println!("{}", tf("msg_recursive_set", &[&enabled_bool.to_string()]).green());
            config.save_with_i18n()?;
        }
        Commands::Ignore { pattern } => {
            if !config.ignore_patterns.contains(&pattern) {
                config.ignore_patterns.push(pattern.clone());
                println!("{}", tf("msg_ignore_added", &[&pattern]).green());
                config.save_with_i18n()?;
            } else {
                println!("{}", tf("msg_ignore_exists", &[&pattern]).yellow());
            }
        }
        Commands::Reset => {
            config = Config::default();
            config.save_with_i18n()?;
            println!("{}", t("msg_config_reset").green());
        }
        Commands::Lang { language } => {
            if is_locale_supported(&language) {
                config.set_language(Some(language.clone()))?;
                config.save_with_i18n()?;
                set_locale(&language);
                println!("{}", tf("msg_language_set", &[&language]).green());
            } else {
                let available = available_locales().join(", ");
                println!("{}", tf("msg_language_invalid", &[&language, &available]).red());
            }
        }
    }

    Ok(())
}

fn run_monitor() -> Result<()> {
    let config = Config::load_with_i18n()?;

    // Validate paths
    let invalid_paths = config.validate_paths();
    if !invalid_paths.is_empty() {
        println!("{}", t("msg_invalid_paths_warning").yellow());
        for path in &invalid_paths {
            println!("  - {}", path.red());
        }
        println!("{}", t("msg_add_valid_paths_hint").bright_white());
    }

    let valid_paths: Vec<_> = config
        .watch_paths
        .iter()
        .filter(|p| Path::new(p).exists())
        .collect();

    if valid_paths.is_empty() {
        println!("{}", t("msg_no_valid_paths").red());
        return Ok(());
    }

    println!("{}", t("msg_monitoring_start").bright_green());
    println!(
        "{}",
        tf("msg_monitoring_paths", &[&valid_paths.len().to_string()]).bright_white()
    );
    for path in &valid_paths {
        println!("  - {}", path.cyan());
    }
    println!(
        "{}",
        tf("msg_monitoring_recursive", &[&config.recursive.to_string()]).bright_white()
    );

    watch(&config)
}

fn watch(config: &Config) -> Result<()> {
    let (tx, rx) = channel();

    // Create file watcher
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;

    // Watch all configured paths
    let recursive_mode = if config.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    for path in &config.watch_paths {
        if Path::new(path).exists() {
            watcher.watch(Path::new(path), recursive_mode)?;
            println!("{}", tf("msg_watching_path", &[path]).bright_green());
        }
    }

    println!("{}", t("msg_monitoring_started").bright_green().bold());

    for res in rx {
        match res {
            Ok(event) => {
                if should_ignore_event(&event, &config.ignore_patterns) {
                    continue;
                }
                handle_event(event);
            }
            Err(e) => println!("{}", tf("msg_monitoring_error", &[&format!("{:?}", e)]).red()),
        }
    }

    Ok(())
}

fn should_ignore_event(event: &Event, ignore_patterns: &[String]) -> bool {
    for path in &event.paths {
        let path_str = path.to_string_lossy();

        for pattern in ignore_patterns {
            // Simple pattern matching - you could use a more sophisticated glob library
            if pattern.contains("**") {
                // Handle directory patterns like ".git/**"
                let dir_pattern = pattern.replace("/**", "");
                if path_str.contains(&dir_pattern) {
                    return true;
                }
            } else if pattern.starts_with("*.") {
                // Handle file extension patterns like "*.tmp"
                let ext = pattern.strip_prefix("*.").unwrap();
                if path_str.ends_with(ext) {
                    return true;
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }
    }
    false
}

fn handle_event(event: Event) {
    match event.kind {
        EventKind::Create(_) => {
            for path in &event.paths {
                println!("{}", tf("msg_file_created", &[&path.display().to_string().cyan().to_string()]).green());
            }
        }
        EventKind::Modify(modify_kind) => {
            match modify_kind {
                notify::event::ModifyKind::Name(name_kind) => {
                    match name_kind {
                        notify::event::RenameMode::Both => {
                            // This is the actual rename event with both old and new paths
                            if event.paths.len() >= 2 {
                                println!("{}", t("msg_file_renamed").yellow());
                                println!(
                                    "{}",
                                    tf("msg_rename_from", &[&event.paths[0].display().to_string().cyan().to_string()])
                                );
                                println!(
                                    "{}",
                                    tf("msg_rename_to", &[&event.paths[1].display().to_string().cyan().to_string()])
                                );
                            }
                        }
                        notify::event::RenameMode::From => {
                            // First phase of rename, can be ignored for cleaner output
                            println!(
                                "{}",
                                tf(
                                    "msg_rename_started",
                                    &[&event.paths[0].display().to_string().cyan().to_string()]
                                ).yellow()
                            );
                        }
                        notify::event::RenameMode::To => {
                            // Second phase of rename, can be ignored for cleaner output
                            println!(
                                "{}",
                                tf(
                                    "msg_rename_completed",
                                    &[&event.paths[0].display().to_string().cyan().to_string()]
                                ).yellow()
                            );
                        }
                        _ => {
                            for path in &event.paths {
                                println!(
                                    "{}",
                                    tf("msg_name_modified", &[&path.display().to_string().cyan().to_string()]).yellow()
                                );
                            }
                        }
                    }
                }
                notify::event::ModifyKind::Data(_) => {
                    for path in &event.paths {
                        println!(
                            "{}",
                            tf("msg_file_content_modified", &[&path.display().to_string().cyan().to_string()]).blue()
                        );
                    }
                }
                notify::event::ModifyKind::Metadata(_) => {
                    // Metadata changes are usually not important, ignore them
                }
                _ => {
                    for path in &event.paths {
                        println!(
                            "{}",
                            tf("msg_file_modified", &[&path.display().to_string().cyan().to_string()]).blue()
                        );
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                println!("{}", tf("msg_file_deleted", &[&path.display().to_string().cyan().to_string()]).red());
            }
        }
        EventKind::Access(_) => {}
        EventKind::Any | EventKind::Other => {}
    }
}
