use crate::target_files::TargetFile;
use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct PathMapping {
    pub original_path: String,
    pub current_path: String,
    pub exists: bool,
    pub target_files: Vec<usize>, // indices of target files containing this path
}

pub struct PathSyncManager {
    target_files: Vec<TargetFile>,
    path_mappings: HashMap<String, PathMapping>,
    watcher: Option<RecommendedWatcher>,
}

impl PathSyncManager {
    pub fn new(target_file_paths: Vec<String>) -> Result<Self> {
        let mut target_files = Vec::new();
        let mut path_mappings: HashMap<String, PathMapping> = HashMap::new();

        println!("{} Loading target files...", "◉".cyan());

        for (index, target_path) in target_file_paths.iter().enumerate() {
            let path = PathBuf::from(target_path);

            if !path.exists() {
                println!(
                    "  {} Creating target file: {}",
                    "◦".yellow(),
                    target_path.bright_white()
                );
                // Create empty file with basic structure based on format
                Self::create_empty_target_file(&path)?;
            }

            match TargetFile::new(path.clone()) {
                Ok(target_file) => {
                    println!(
                        "  {} Loaded: {} ({} paths found)",
                        "✓".green(),
                        target_path.bright_white(),
                        target_file.paths.len().to_string().cyan()
                    );

                    // Index all paths from this target file
                    for path_entry in &target_file.paths {
                        let path_key = path_entry.path.clone();

                        match path_mappings.get_mut(&path_key) {
                            Some(mapping) => {
                                mapping.target_files.push(index);
                            }
                            None => {
                                path_mappings.insert(
                                    path_key.clone(),
                                    PathMapping {
                                        original_path: path_key.clone(),
                                        current_path: path_key.clone(),
                                        exists: path_entry.exists,
                                        target_files: vec![index],
                                    },
                                );
                            }
                        }
                    }

                    target_files.push(target_file);
                }
                Err(e) => {
                    eprintln!(
                        "  {} Failed to load {}: {}",
                        "✗".red(),
                        target_path.bright_white(),
                        e
                    );
                    return Err(e);
                }
            }
        }

        Ok(Self {
            target_files,
            path_mappings,
            watcher: None,
        })
    }

    fn create_empty_target_file(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => "[]",
            Some("yaml") | Some("yml") => "paths: []",
            Some("toml") => "paths = []",
            Some("csv") => "path,type\n",
            _ => "",
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn start_monitoring(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |result| {
                if let Ok(event) = result {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )?;

        // Watch all directories containing our tracked paths
        let watched_dirs = self.get_directories_to_watch();
        for dir in watched_dirs {
            if dir.exists() {
                watcher.watch(&dir, RecursiveMode::Recursive)?;
                println!(
                    "  {} Watching: {}",
                    "👁".bright_blue(),
                    dir.display().to_string().bright_white()
                );
            }
        }

        self.watcher = Some(watcher);

        println!("{} Path synchronization started", "🚀".bright_green());

        // Handle events in a separate thread
        let target_files = Arc::new(Mutex::new(self.target_files.clone()));
        let path_mappings = Arc::new(Mutex::new(self.path_mappings.clone()));

        thread::spawn(move || {
            for event in rx {
                if let Err(e) = Self::handle_event(&event, &target_files, &path_mappings) {
                    eprintln!("Error handling event: {}", e);
                }
            }
        });

        Ok(())
    }

    fn get_directories_to_watch(&self) -> Vec<PathBuf> {
        let mut dirs = std::collections::HashSet::new();

        // Add directories of all tracked paths
        for mapping in self.path_mappings.values() {
            let path = Path::new(&mapping.current_path);
            if let Some(parent) = path.parent() {
                dirs.insert(parent.to_path_buf());
            }
            if path.is_dir() {
                dirs.insert(path.to_path_buf());
            }
        }

        // Add current directory as fallback
        if dirs.is_empty() {
            dirs.insert(PathBuf::from("."));
        }

        dirs.into_iter().collect()
    }

    fn handle_event(
        event: &Event,
        target_files: &Arc<Mutex<Vec<TargetFile>>>,
        path_mappings: &Arc<Mutex<HashMap<String, PathMapping>>>,
    ) -> Result<()> {
        match event.kind {
            EventKind::Create(_) => {
                for path in &event.paths {
                    Self::handle_path_created(path, target_files, path_mappings)?;
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    Self::handle_path_removed(path, target_files, path_mappings)?;
                }
            }
            EventKind::Modify(_) => {
                for path in &event.paths {
                    Self::handle_path_moved(path, target_files, path_mappings)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_path_created(
        path: &Path,
        target_files: &Arc<Mutex<Vec<TargetFile>>>,
        path_mappings: &Arc<Mutex<HashMap<String, PathMapping>>>,
    ) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();

        let mut mappings = path_mappings.lock().unwrap();

        // Check if this is a previously deleted path being restored
        for (_, mapping) in mappings.iter_mut() {
            if mapping.current_path == path_str && !mapping.exists {
                mapping.exists = true;

                println!(
                    "{} Path restored: {}",
                    "🔄".bright_green(),
                    path_str.bright_white()
                );

                // Update target files
                let mut files = target_files.lock().unwrap();
                for &file_idx in &mapping.target_files {
                    if let Some(target_file) = files.get_mut(file_idx) {
                        target_file.mark_path_restored(&path_str)?;
                    }
                }
                break;
            }
        }

        Ok(())
    }

    fn handle_path_removed(
        path: &Path,
        target_files: &Arc<Mutex<Vec<TargetFile>>>,
        path_mappings: &Arc<Mutex<HashMap<String, PathMapping>>>,
    ) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();

        let mut mappings = path_mappings.lock().unwrap();

        if let Some(mapping) = mappings.get_mut(&path_str) {
            mapping.exists = false;

            println!(
                "{} Path deleted (tracking continues): {}",
                "🗑".yellow(),
                path_str.bright_white()
            );

            // Update target files
            let mut files = target_files.lock().unwrap();
            for &file_idx in &mapping.target_files {
                if let Some(target_file) = files.get_mut(file_idx) {
                    target_file.mark_path_deleted(&path_str)?;
                }
            }
        }

        Ok(())
    }

    fn handle_path_moved(
        _path: &Path,
        _target_files: &Arc<Mutex<Vec<TargetFile>>>,
        _path_mappings: &Arc<Mutex<HashMap<String, PathMapping>>>,
    ) -> Result<()> {
        // Path moves are complex to detect with basic file events
        // For now, we'll rely on create/delete events
        // A future enhancement could implement move detection
        Ok(())
    }

    pub fn sync_path(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        println!(
            "{} Syncing path change: {} → {}",
            "🔄".bright_blue(),
            old_path.bright_white(),
            new_path.bright_green()
        );

        if let Some(mapping) = self.path_mappings.get_mut(old_path) {
            let file_indices = mapping.target_files.clone();

            // Update all target files containing this path
            for &file_idx in &file_indices {
                if let Some(target_file) = self.target_files.get_mut(file_idx) {
                    target_file.update_path(old_path, new_path)?;
                    println!(
                        "  {} Updated: {}",
                        "✓".green(),
                        target_file.path.display().to_string().bright_white()
                    );
                }
            }

            // Update the mapping
            mapping.current_path = new_path.to_string();
            mapping.exists = Path::new(new_path).exists();

            // Update the key in the HashMap
            let updated_mapping = mapping.clone();
            self.path_mappings.remove(old_path);
            self.path_mappings
                .insert(new_path.to_string(), updated_mapping);
        }

        Ok(())
    }

    pub fn get_path_status(&self) -> Vec<(String, bool, Vec<String>)> {
        self.path_mappings
            .iter()
            .map(|(path, mapping)| {
                let target_file_names: Vec<String> = mapping
                    .target_files
                    .iter()
                    .filter_map(|&idx| self.target_files.get(idx))
                    .map(|tf| {
                        tf.path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    })
                    .collect();

                (path.clone(), mapping.exists, target_file_names)
            })
            .collect()
    }

    pub fn print_status(&self) {
        println!("\n{} Path Synchronization Status", "📊".bright_blue());
        println!("{}", "─".repeat(50).bright_black());

        let status = self.get_path_status();
        if status.is_empty() {
            println!("  {} No paths being tracked", "ℹ".bright_yellow());
            return;
        }

        for (path, exists, target_files) in status {
            let status_icon = if exists {
                "✓".green().to_string()
            } else {
                "✗".red().to_string()
            };
            let status_text = if exists {
                "exists".green().to_string()
            } else {
                "missing".red().to_string()
            };

            println!(
                "  {} {} [{}]",
                status_icon,
                path.bright_white(),
                status_text
            );
            for target_file in target_files {
                println!("    └─ {}", target_file.bright_black());
            }
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        println!("{} Refreshing target files...", "🔄".bright_blue());

        for target_file in &mut self.target_files {
            *target_file = TargetFile::new(target_file.path.clone())?;
        }

        // Rebuild path mappings
        self.path_mappings.clear();
        for (index, target_file) in self.target_files.iter().enumerate() {
            for path_entry in &target_file.paths {
                let path_key = path_entry.path.clone();

                match self.path_mappings.get_mut(&path_key) {
                    Some(mapping) => {
                        mapping.target_files.push(index);
                    }
                    None => {
                        self.path_mappings.insert(
                            path_key.clone(),
                            PathMapping {
                                original_path: path_key.clone(),
                                current_path: path_key.clone(),
                                exists: path_entry.exists,
                                target_files: vec![index],
                            },
                        );
                    }
                }
            }
        }

        println!("  {} Refresh completed", "✓".green());
        Ok(())
    }
}

impl Drop for PathSyncManager {
    fn drop(&mut self) {
        if self.watcher.is_some() {
            println!("{} Path synchronization stopped", "🛑".bright_red());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_sync_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let json_file = temp_dir.path().join("test.json");

        fs::write(&json_file, r#"["./test/path"]"#).unwrap();

        let manager = PathSyncManager::new(vec![json_file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(manager.target_files.len(), 1);
    }

    #[test]
    fn test_sync_path() {
        let temp_dir = TempDir::new().unwrap();
        let json_file = temp_dir.path().join("test.json");

        fs::write(&json_file, r#"["./old/path"]"#).unwrap();

        let mut manager =
            PathSyncManager::new(vec![json_file.to_string_lossy().to_string()]).unwrap();
        manager.sync_path("./old/path", "./new/path").unwrap();

        let content = fs::read_to_string(&json_file).unwrap();
        assert!(content.contains("./new/path"));
        assert!(!content.contains("./old/path"));
    }

    #[test]
    fn test_create_empty_target_files() {
        let temp_dir = TempDir::new().unwrap();

        let json_file = temp_dir.path().join("test.json");
        PathSyncManager::create_empty_target_file(&json_file).unwrap();
        assert!(json_file.exists());

        let yaml_file = temp_dir.path().join("test.yaml");
        PathSyncManager::create_empty_target_file(&yaml_file).unwrap();
        assert!(yaml_file.exists());
    }
}
