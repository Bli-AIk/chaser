use crate::i18n::{t, tf};
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
    watch_paths: Vec<String>,
    watcher: Option<RecommendedWatcher>,
}

impl PathSyncManager {
    pub fn new(target_file_paths: Vec<String>, watch_paths: Vec<String>) -> Result<Self> {
        let mut target_files = Vec::new();
        let mut path_mappings: HashMap<String, PathMapping> = HashMap::new();

        println!("{}", t("msg_loading_target_files").cyan());

        for (index, target_path) in target_file_paths.iter().enumerate() {
            let path = PathBuf::from(target_path);

            if !path.exists() {
                println!(
                    "  {}",
                    tf("msg_target_file_created", &[target_path]).yellow()
                );
                Self::create_empty_target_file(&path)?;
            }

            match TargetFile::new(path.clone()) {
                Ok(target_file) => {
                    println!(
                        "  {}",
                        tf(
                            "msg_target_file_loaded",
                            &[target_path, &target_file.paths.len().to_string()]
                        )
                        .green()
                    );

                    // Validate that paths are within watch directories
                    let valid_paths =
                        Self::filter_paths_in_watch_dirs(&target_file.paths, &watch_paths);

                    if valid_paths.len() != target_file.paths.len() {
                        let filtered_count = target_file.paths.len() - valid_paths.len();
                        println!(
                            "    {} Filtered out {} paths not in watch directories",
                            "⚠".yellow(),
                            filtered_count.to_string().yellow()
                        );
                    }

                    // Index valid paths from this target file
                    for path_entry in &valid_paths {
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

        println!(
            "  {}",
            tf(
                "msg_tracking_summary",
                &[
                    &path_mappings.len().to_string(),
                    &target_files.len().to_string()
                ]
            )
            .bright_blue()
        );

        Ok(Self {
            target_files,
            path_mappings,
            watch_paths,
            watcher: None,
        })
    }

    /// Filter paths to only include those within watch directories
    fn filter_paths_in_watch_dirs(
        paths: &[crate::target_files::PathEntry],
        watch_paths: &[String],
    ) -> Vec<crate::target_files::PathEntry> {
        paths
            .iter()
            .filter(|path_entry| {
                watch_paths.iter().any(|watch_path| {
                    let watch_path_canonical = Path::new(watch_path)
                        .canonicalize()
                        .unwrap_or_else(|_| PathBuf::from(watch_path));
                    let target_path_canonical = Path::new(&path_entry.path)
                        .canonicalize()
                        .unwrap_or_else(|_| PathBuf::from(&path_entry.path));

                    target_path_canonical.starts_with(&watch_path_canonical)
                        || Path::new(&path_entry.path).starts_with(watch_path)
                })
            })
            .cloned()
            .collect()
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

        // Watch the configured watch paths
        for watch_path in &self.watch_paths {
            let path = Path::new(watch_path);
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                println!(
                    "  {}",
                    tf("msg_watching_path", &[&path.display().to_string()]).bright_blue()
                );
            } else {
                println!(
                    "  {}",
                    tf("msg_watch_path_not_exist", &[watch_path]).yellow()
                );
            }
        }

        self.watcher = Some(watcher);

        println!("{}", t("msg_path_sync_monitoring_started").bright_green());

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
                // For moves/renames, we need to detect the old->new path change
                // This is complex with notify; for now we'll handle create/delete pairs
                for path in &event.paths {
                    Self::handle_path_modified(path, target_files, path_mappings)?;
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

        // Check if this is a previously tracked path being restored
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

    fn handle_path_modified(
        _path: &Path,
        _target_files: &Arc<Mutex<Vec<TargetFile>>>,
        _path_mappings: &Arc<Mutex<HashMap<String, PathMapping>>>,
    ) -> Result<()> {
        // Path moves are complex to detect with basic file events
        // A comprehensive solution would require tracking inode changes
        // For now, we rely on create/delete event pairs
        Ok(())
    }

    /// Manually sync a path change (for testing or manual operations)
    pub fn sync_path_change(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        println!(
            "{}",
            tf("msg_syncing_path_change", &[old_path, new_path]).bright_blue()
        );

        // Normalize paths for consistent comparison
        let old_path_canonical = Path::new(old_path)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(old_path));
        let new_path_buf = PathBuf::from(new_path);

        // Find all paths that need to be updated:
        // 1. Exact match of the old path
        // 2. Any paths that are subdirectories/subfiles of the old path
        let mut paths_to_update: Vec<(String, String, PathMapping)> = Vec::new();

        // First, collect all mappings that need to be updated
        for (current_key, mapping) in &self.path_mappings {
            let should_update = if current_key == old_path {
                // Exact match
                true
            } else {
                // Check if current path is a subdirectory of the old path
                let current_canonical = Path::new(current_key)
                    .canonicalize()
                    .unwrap_or_else(|_| PathBuf::from(current_key));
                
                // Check if current path starts with old path (is a subpath)
                current_canonical.starts_with(&old_path_canonical) ||
                Path::new(current_key).starts_with(old_path)
            };

            if should_update {
                // Calculate the new path for this entry
                let new_key = if current_key == old_path {
                    // Exact match - replace with new path
                    new_path.to_string()
                } else {
                    // Subpath - replace the prefix
                    if let Ok(relative_part) = Path::new(current_key).strip_prefix(old_path) {
                        new_path_buf.join(relative_part).to_string_lossy().to_string()
                    } else {
                        // Try with canonical paths
                        let current_canonical = Path::new(current_key)
                            .canonicalize()
                            .unwrap_or_else(|_| PathBuf::from(current_key));
                        
                        if let Ok(relative_part) = current_canonical.strip_prefix(&old_path_canonical) {
                            new_path_buf.join(relative_part).to_string_lossy().to_string()
                        } else {
                            // Fallback: shouldn't happen, but keep original key
                            current_key.clone()
                        }
                    }
                };

                paths_to_update.push((current_key.clone(), new_key, mapping.clone()));
            }
        }

        if paths_to_update.is_empty() {
            println!(
                "  {}",
                tf("msg_path_not_found_in_tracking", &[old_path]).yellow()
            );
            return Ok(());
        }

        // Now update all the paths
        for (old_key, new_key, mut mapping) in paths_to_update {
            // Update all target files containing this path
            for &file_idx in &mapping.target_files {
                if let Some(target_file) = self.target_files.get_mut(file_idx) {
                    target_file.update_path(&old_key, &new_key)?;
                    println!(
                        "  {}",
                        tf(
                            "msg_target_file_updated",
                            &[&target_file.path.display().to_string()]
                        )
                        .green()
                    );
                }
            }

            // Update the mapping
            mapping.current_path = new_key.clone();
            mapping.exists = Path::new(&new_key).exists();

            // Remove old mapping and insert new one
            self.path_mappings.remove(&old_key);
            self.path_mappings.insert(new_key, mapping);
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

        if self.watch_paths.is_empty() {
            println!("  {} No watch paths configured", "ℹ".bright_yellow());
            return;
        }

        println!("Watch directories:");
        for watch_path in &self.watch_paths {
            let exists = Path::new(watch_path).exists();
            let status_icon = if exists {
                "✓".green().to_string()
            } else {
                "✗".red().to_string()
            };
            println!("  {} {}", status_icon, watch_path.bright_white());
        }

        println!();
        let status = self.get_path_status();
        if status.is_empty() {
            println!("  {} No target paths being tracked", "ℹ".bright_yellow());
            return;
        }

        println!("Tracked paths in target files:");
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

        // Rebuild path mappings with watch path filtering
        self.path_mappings.clear();
        for (index, target_file) in self.target_files.iter().enumerate() {
            let valid_paths =
                Self::filter_paths_in_watch_dirs(&target_file.paths, &self.watch_paths);

            for path_entry in &valid_paths {
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
    fn test_path_sync_manager_with_watch_paths() {
        let temp_dir = TempDir::new().unwrap();
        let watch_dir = temp_dir.path().join("watch");
        fs::create_dir_all(&watch_dir).unwrap();

        let target_path = watch_dir.join("target.txt");
        fs::write(&target_path, "test").unwrap();

        let json_file = temp_dir.path().join("test.json");
        fs::write(
            &json_file,
            format!(r#"["{}"]"#, target_path.to_string_lossy()),
        )
        .unwrap();

        let manager = PathSyncManager::new(
            vec![json_file.to_string_lossy().to_string()],
            vec![watch_dir.to_string_lossy().to_string()],
        )
        .unwrap();

        assert_eq!(manager.target_files.len(), 1);
        assert!(!manager.path_mappings.is_empty());
    }

    #[test]
    fn test_filter_paths_in_watch_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let watch_dir = temp_dir.path().join("watch");
        fs::create_dir_all(&watch_dir).unwrap();

        let inside_path = watch_dir.join("inside.txt");
        let outside_path = temp_dir.path().join("outside.txt");
        fs::write(&inside_path, "test").unwrap();
        fs::write(&outside_path, "test").unwrap();

        let paths = vec![
            crate::target_files::PathEntry {
                path: inside_path.to_string_lossy().to_string(),
                exists: true,
                last_known_path: None,
            },
            crate::target_files::PathEntry {
                path: outside_path.to_string_lossy().to_string(),
                exists: true,
                last_known_path: None,
            },
        ];

        let watch_paths = vec![watch_dir.to_string_lossy().to_string()];
        let filtered = PathSyncManager::filter_paths_in_watch_dirs(&paths, &watch_paths);

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].path.contains("inside.txt"));
    }

    #[test]
    fn test_sync_path_change() {
        let temp_dir = TempDir::new().unwrap();
        let watch_dir = temp_dir.path().join("watch");
        fs::create_dir_all(&watch_dir).unwrap();

        let old_path = watch_dir.join("old.txt");
        fs::write(&old_path, "test").unwrap();

        let json_file = temp_dir.path().join("test.json");
        fs::write(&json_file, format!(r#"["{}"]"#, old_path.to_string_lossy())).unwrap();

        let mut manager = PathSyncManager::new(
            vec![json_file.to_string_lossy().to_string()],
            vec![watch_dir.to_string_lossy().to_string()],
        )
        .unwrap();

        let new_path = watch_dir.join("new.txt");
        manager
            .sync_path_change(&old_path.to_string_lossy(), &new_path.to_string_lossy())
            .unwrap();

        let content = fs::read_to_string(&json_file).unwrap();
        assert!(content.contains("new.txt"));
        assert!(!content.contains("old.txt"));
    }

    #[test]
    fn test_sync_directory_rename_updates_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let watch_dir = temp_dir.path().join("watch");
        fs::create_dir_all(&watch_dir).unwrap();

        // 创建一个目录和子文件
        let old_dir = watch_dir.join("src");
        fs::create_dir_all(&old_dir).unwrap();
        let sub_file = old_dir.join("main.rs");
        fs::write(&sub_file, "fn main() {}").unwrap();

        // 创建目标文件，包含目录和子文件的路径
        let json_file = temp_dir.path().join("test.json");
        fs::write(
            &json_file,
            format!(
                r#"["{}","{}"]"#,
                old_dir.to_string_lossy(),
                sub_file.to_string_lossy()
            ),
        )
        .unwrap();

        let mut manager = PathSyncManager::new(
            vec![json_file.to_string_lossy().to_string()],
            vec![watch_dir.to_string_lossy().to_string()],
        )
        .unwrap();

        // 验证初始状态 - 应该有两个路径被跟踪
        assert_eq!(manager.path_mappings.len(), 2);
        assert!(manager.path_mappings.contains_key(&old_dir.to_string_lossy().to_string()));
        assert!(manager.path_mappings.contains_key(&sub_file.to_string_lossy().to_string()));

        // 模拟目录重命名
        let new_dir = watch_dir.join("source");
        let new_sub_file = new_dir.join("main.rs");
        
        // 重命名目录
        manager
            .sync_path_change(&old_dir.to_string_lossy(), &new_dir.to_string_lossy())
            .unwrap();

        // 读取更新后的文件内容
        let content = fs::read_to_string(&json_file).unwrap();
        
        // 验证目录路径已更新
        assert!(content.contains("source"));
        assert!(!content.contains("\"src\""));
        
        // 关键测试：验证子文件路径也已更新
        assert!(content.contains(&new_sub_file.to_string_lossy().to_string()), 
               "子文件路径应该从 {} 更新为 {}", 
               sub_file.to_string_lossy(), 
               new_sub_file.to_string_lossy());
        assert!(!content.contains(&sub_file.to_string_lossy().to_string()), 
               "旧的子文件路径 {} 应该被移除", 
               sub_file.to_string_lossy());
    }

    #[test]
    fn test_sync_nested_directory_rename() {
        let temp_dir = TempDir::new().unwrap();
        let watch_dir = temp_dir.path().join("watch");
        fs::create_dir_all(&watch_dir).unwrap();

        // 创建嵌套目录结构
        let old_dir = watch_dir.join("test_files").join("src");
        fs::create_dir_all(&old_dir).unwrap();
        
        let sub_dir = old_dir.join("components");
        fs::create_dir_all(&sub_dir).unwrap();
        
        let main_file = old_dir.join("main.rs");
        let comp_file = sub_dir.join("button.rs");
        fs::write(&main_file, "fn main() {}").unwrap();
        fs::write(&comp_file, "pub struct Button;").unwrap();

        // 创建目标文件，包含多个嵌套路径
        let json_file = temp_dir.path().join("test.json");
        fs::write(
            &json_file,
            format!(
                r#"["{}","{}","{}","{}"]"#,
                watch_dir.join("test_files").to_string_lossy(),
                old_dir.to_string_lossy(),
                main_file.to_string_lossy(),
                comp_file.to_string_lossy()
            ),
        )
        .unwrap();

        let mut manager = PathSyncManager::new(
            vec![json_file.to_string_lossy().to_string()],
            vec![watch_dir.to_string_lossy().to_string()],
        )
        .unwrap();

        // 验证初始状态
        assert_eq!(manager.path_mappings.len(), 4);

        // 重命名 src 目录为 source
        let new_dir = watch_dir.join("test_files").join("source");
        manager
            .sync_path_change(&old_dir.to_string_lossy(), &new_dir.to_string_lossy())
            .unwrap();

        let content = fs::read_to_string(&json_file).unwrap();
        
        // 验证所有相关路径都已更新
        assert!(content.contains("source"));
        assert!(!content.contains("/src/"));
        
        // 验证嵌套文件路径正确更新
        let new_main_file = new_dir.join("main.rs");
        let new_comp_file = new_dir.join("components").join("button.rs");
        
        assert!(content.contains(&new_main_file.to_string_lossy().to_string()));
        assert!(content.contains(&new_comp_file.to_string_lossy().to_string()));
        assert!(!content.contains(&main_file.to_string_lossy().to_string()));
        assert!(!content.contains(&comp_file.to_string_lossy().to_string()));
    }
}
