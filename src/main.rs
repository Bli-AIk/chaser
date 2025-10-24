mod config;
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use notify::{Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => handle_command(command),
        None => run_monitor(),
    }
}

fn handle_command(command: Commands) -> Result<()> {
    let mut config = Config::load()?;

    match command {
        Commands::Add { path } => {
            config.add_path(path)?;
            config.save()?;
        }
        Commands::Remove { path } => {
            config.remove_path(&path)?;
            config.save()?;
        }
        Commands::List => {
            config.list_paths();
        }
        Commands::Config => {
            let config_path = Config::config_file_path()?;
            println!("Config file location: {}", config_path.display());
            println!("You can edit this file directly if needed.");
        }
        Commands::Recursive { enabled } => {
            let enabled_bool = match enabled.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    println!("❌ Invalid value: '{}'. Use true/false, yes/no, 1/0, or on/off", enabled);
                    return Ok(());
                }
            };
            config.recursive = enabled_bool;
            println!("✓ Recursive watching set to: {}", enabled_bool);
            config.save()?;
        }
        Commands::Ignore { pattern } => {
            if !config.ignore_patterns.contains(&pattern) {
                config.ignore_patterns.push(pattern.clone());
                println!("✓ Added ignore pattern: {}", pattern);
                config.save()?;
            } else {
                println!("⚠ Pattern already exists: {}", pattern);
            }
        }
        Commands::Reset => {
            config = Config::default();
            config.save()?;
            println!("✓ Config reset to default values");
        }
    }

    Ok(())
}

fn run_monitor() -> Result<()> {
    let config = Config::load()?;
    
    // Validate paths
    let invalid_paths = config.validate_paths();
    if !invalid_paths.is_empty() {
        println!("⚠ Warning: Some paths don't exist:");
        for path in &invalid_paths {
            println!("  - {}", path);
        }
        println!("You can add valid paths using: chaser add <path>");
    }

    let valid_paths: Vec<_> = config.watch_paths.iter()
        .filter(|p| Path::new(p).exists())
        .collect();

    if valid_paths.is_empty() {
        println!("❌ No valid paths to monitor. Add some paths using: chaser add <path>");
        return Ok(());
    }

    println!("Starting file monitoring...");
    println!("Monitoring {} path(s):", valid_paths.len());
    for path in &valid_paths {
        println!("  - {}", path);
    }
    println!("Recursive: {}", config.recursive);

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
            println!("✓ Watching: {}", path);
        }
    }

    println!("File monitoring started, press Ctrl+C to exit...\n");

    for res in rx {
        match res {
            Ok(event) => {
                if should_ignore_event(&event, &config.ignore_patterns) {
                    continue;
                }
                handle_event(event);
            }
            Err(e) => println!("Monitoring error: {:?}", e),
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
                println!("✓ File created: {}", path.display());
            }
        }
        EventKind::Modify(modify_kind) => {
            match modify_kind {
                notify::event::ModifyKind::Name(name_kind) => {
                    match name_kind {
                        notify::event::RenameMode::Both => {
                            // This is the actual rename event with both old and new paths
                            if event.paths.len() >= 2 {
                                println!("🔄 File renamed:");
                                println!("   From: {}", event.paths[0].display());
                                println!("   To: {}", event.paths[1].display());
                            }
                        }
                        notify::event::RenameMode::From => {
                            // First phase of rename, can be ignored for cleaner output
                            println!("🔄 Rename started: {}", event.paths[0].display());
                        }
                        notify::event::RenameMode::To => {
                            // Second phase of rename, can be ignored for cleaner output
                            println!("🔄 Rename completed: {}", event.paths[0].display());
                        }
                        _ => {
                            for path in &event.paths {
                                println!("📝 Name modified: {}", path.display());
                            }
                        }
                    }
                }
                notify::event::ModifyKind::Data(_) => {
                    for path in &event.paths {
                        println!("📝 File content modified: {}", path.display());
                    }
                }
                notify::event::ModifyKind::Metadata(_) => {
                    // Metadata changes are usually not important, ignore them
                }
                _ => {
                    for path in &event.paths {
                        println!("📝 File modified: {}", path.display());
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                println!("🗑️ File deleted: {}", path.display());
            }
        }
        EventKind::Access(_) => {
            // Access events are usually too frequent, ignore them
        }
        EventKind::Any | EventKind::Other => {
            // These are catch-all events that are already handled by more specific events above
        }
    }
}
