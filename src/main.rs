use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;

fn main() {
    println!("Starting file monitoring...");
    println!("Monitoring directory: ./test_files");
    println!("Try moving or renaming files in the test_files directory");
    
    if let Err(e) = watch() {
        println!("Monitoring error: {:?}", e);
    }
}

fn watch() -> notify::Result<()> {
    let (tx, rx) = channel();

    // Create file watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Monitor test_files directory
    watcher.watch(Path::new("./test_files"), RecursiveMode::Recursive)?;

    println!("File monitoring started, press Ctrl+C to exit...\n");

    for res in rx {
        match res {
            Ok(event) => handle_event(event),
            Err(e) => println!("Monitoring error: {:?}", e),
        }
    }

    Ok(())
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
