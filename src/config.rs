use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub recursive: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_paths: vec!["./test_files".to_string()],
            recursive: true,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
        }
    }
}

impl Config {
    /// Get the config file path (cross-platform)
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        
        let app_config_dir = config_dir.join("chaser");
        
        // Create config directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .context("Failed to create config directory")?;
        }
        
        Ok(app_config_dir.join("config.yaml"))
    }

    /// Load config from file, create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            
            let config: Config = serde_yaml::from_str(&content)
                .context("Failed to parse config file")?;
            
            println!("✓ Loaded config from: {}", config_path.display());
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            println!("✓ Created default config at: {}", config_path.display());
            Ok(default_config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        println!("✓ Config saved to: {}", config_path.display());
        Ok(())
    }

    /// Add a watch path
    pub fn add_path(&mut self, path: String) -> Result<()> {
        if !self.watch_paths.contains(&path) {
            self.watch_paths.push(path.clone());
            println!("✓ Added watch path: {}", path);
        } else {
            println!("⚠ Path already exists: {}", path);
        }
        Ok(())
    }

    /// Remove a watch path
    pub fn remove_path(&mut self, path: &str) -> Result<()> {
        if let Some(pos) = self.watch_paths.iter().position(|p| p == path) {
            self.watch_paths.remove(pos);
            println!("✓ Removed watch path: {}", path);
        } else {
            println!("⚠ Path not found: {}", path);
        }
        Ok(())
    }

    /// List all watch paths
    pub fn list_paths(&self) {
        println!("Watch paths:");
        for (i, path) in self.watch_paths.iter().enumerate() {
            println!("  {}. {}", i + 1, path);
        }
        
        println!("\nSettings:");
        println!("  Recursive: {}", self.recursive);
        println!("  Ignore patterns: {:?}", self.ignore_patterns);
    }

    /// Validate paths exist
    pub fn validate_paths(&self) -> Vec<String> {
        let mut invalid_paths = Vec::new();
        
        for path in &self.watch_paths {
            if !Path::new(path).exists() {
                invalid_paths.push(path.clone());
            }
        }
        
        invalid_paths
    }
}