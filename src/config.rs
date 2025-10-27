use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub recursive: bool,
    pub ignore_patterns: Vec<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub target_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            recursive: true,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
            language: None,
            target_files: vec![],
        }
    }
}

impl Config {
    /// Get the config file path (cross-platform)
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        let app_config_dir = config_dir.join("chaser");

        Self::ensure_config_dir_exists(&app_config_dir)?;
        Ok(app_config_dir.join("config.yaml"))
    }

    fn ensure_config_dir_exists(dir: &Path) -> Result<()> {
        if !dir.exists() {
            fs::create_dir_all(dir).context("Failed to create config directory")?;
        }
        Ok(())
    }

    /// Load config from file, create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            let config: Config =
                serde_yaml_ng::from_str(&content).context("Failed to parse config file")?;

            eprintln!(
                "{} {}",
                "✓".green(),
                format!("Loaded config from: {}", config_path.display()).bright_white()
            );
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            eprintln!(
                "{} {}",
                "✓".green(),
                format!("Created default config at: {}", config_path.display()).bright_white()
            );
            Ok(default_config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        let content = serde_yaml_ng::to_string(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        eprintln!(
            "{} {}",
            "✓".green(),
            format!("Config saved to: {}", config_path.display()).bright_white()
        );
        Ok(())
    }

    /// Add a watch path
    pub fn add_path(&mut self, path: String) -> Result<()> {
        if !self.watch_paths.contains(&path) {
            self.watch_paths.push(path.clone());
            println!("{}", crate::i18n::tf("msg_path_added", &[&path]).green());
        } else {
            println!("{}", crate::i18n::tf("msg_path_exists", &[&path]).yellow());
        }
        Ok(())
    }

    /// Remove a watch path
    pub fn remove_path(&mut self, path: &str) -> Result<()> {
        if let Some(pos) = self.watch_paths.iter().position(|p| p == path) {
            self.watch_paths.remove(pos);
            println!("{}", crate::i18n::tf("msg_path_removed", &[path]).green());
        } else {
            println!("{}", crate::i18n::tf("msg_path_not_found", &[path]).red());
        }
        Ok(())
    }

    /// List all watch paths
    pub fn list_paths(&self) {
        println!("{}", crate::i18n::t("ui_watch_paths").bright_cyan().bold());
        for (i, path) in self.watch_paths.iter().enumerate() {
            println!("  {}. {}", format!("{}", i + 1).bright_white(), path.cyan());
        }

        println!("\n{}", crate::i18n::t("ui_settings").bright_cyan().bold());
        println!(
            "  {}",
            crate::i18n::tf("ui_recursive", &[&self.recursive.to_string()]).bright_white()
        );
        println!(
            "  {}: [{}]",
            "Ignore patterns".bright_white(),
            self.ignore_patterns
                .iter()
                .map(|p| p.yellow().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        if let Some(ref lang) = self.language {
            println!("  {}: {}", "Language".bright_white(), lang.green());
        } else {
            println!(
                "  {}: {} {}",
                "Language".bright_white(),
                self.get_effective_language().green(),
                "(auto)".dimmed()
            );
        }
    }

    /// Load config with i18n messages (use after i18n is initialized)
    pub fn load_with_i18n() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            let config: Config =
                serde_yaml_ng::from_str(&content).context("Failed to parse config file")?;

            println!(
                "{}",
                crate::i18n::tf(
                    "msg_config_loaded",
                    &[&config_path.display().to_string().cyan().to_string()]
                )
                .green()
            );
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save_with_i18n()?;
            println!(
                "{}",
                crate::i18n::tf(
                    "msg_config_created",
                    &[&config_path.display().to_string().cyan().to_string()]
                )
                .green()
            );
            Ok(default_config)
        }
    }

    /// Save config with i18n messages (use after i18n is initialized)
    pub fn save_with_i18n(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        let content = serde_yaml_ng::to_string(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        println!(
            "{}",
            crate::i18n::tf(
                "msg_config_saved",
                &[&config_path.display().to_string().cyan().to_string()]
            )
            .green()
        );
        Ok(())
    }
    pub fn set_language(&mut self, language: Option<String>) -> Result<()> {
        self.language = language;
        Ok(())
    }

    /// Get effective language (config or system default)
    pub fn get_effective_language(&self) -> String {
        if let Some(ref lang) = self.language {
            lang.clone()
        } else {
            // Get system locale - simplified version
            if let Some(locale) = std::env::var("LANG").ok() {
                let locale_lower = locale.to_lowercase();
                if locale_lower.starts_with("zh")
                    && (locale_lower.contains("cn") || locale_lower.contains("hans"))
                {
                    "zh-cn".to_string()
                } else {
                    "en".to_string()
                }
            } else {
                "en".to_string()
            }
        }
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

    /// Add a target file
    pub fn add_target_file(&mut self, target_file: String) -> Result<()> {
        if !self.target_files.contains(&target_file) {
            self.target_files.push(target_file);
        }
        Ok(())
    }

    /// Remove a target file
    pub fn remove_target_file(&mut self, target_file: &str) -> Result<()> {
        self.target_files.retain(|p| p != target_file);
        Ok(())
    }

    /// List all target files
    pub fn list_target_files(&self) -> &Vec<String> {
        &self.target_files
    }

    /// Validate target files have at least one entry
    pub fn validate_target_files(&self) -> Result<()> {
        if self.target_files.is_empty() {
            let error_msg = crate::i18n::t("msg_error_no_target_files");
            let hint_msg = crate::i18n::t("msg_error_no_target_files_hint");
            anyhow::bail!("{}. {}", error_msg, hint_msg);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn create_test_config_with_temp_dir() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("chaser");
        fs::create_dir_all(&config_dir).unwrap();

        let mut config = Config::default();
        config.watch_paths = vec![temp_dir.path().to_string_lossy().to_string()];

        (config, temp_dir)
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.watch_paths, Vec::<String>::new());
        assert_eq!(config.recursive, true);
        assert_eq!(
            config.ignore_patterns,
            vec!["*.tmp", "*.log", ".git/**", "target/**"]
        );
        assert_eq!(config.language, None);
        assert_eq!(config.target_files, Vec::<String>::new());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml_str = serde_yaml_ng::to_string(&config).unwrap();
        assert!(yaml_str.contains("watch_paths"));
        assert!(yaml_str.contains("recursive"));
        assert!(yaml_str.contains("ignore_patterns"));

        let deserialized: Config = serde_yaml_ng::from_str(&yaml_str).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    #[serial]
    fn test_config_file_path() {
        // Test config file path generation
        let result = Config::config_file_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("chaser"));
        assert!(path.file_name().unwrap() == "config.yaml");
    }

    #[test]
    fn test_add_path() {
        let mut config = Config::default();
        let initial_count = config.watch_paths.len();

        // Test adding new path
        let result = config.add_path("./new_path".to_string());
        assert!(result.is_ok());
        assert_eq!(config.watch_paths.len(), initial_count + 1);
        assert!(config.watch_paths.contains(&"./new_path".to_string()));

        // Test adding duplicate path (should not increase count)
        let result = config.add_path("./new_path".to_string());
        assert!(result.is_ok());
        assert_eq!(config.watch_paths.len(), initial_count + 1);
    }

    #[test]
    fn test_remove_path() {
        let mut config = Config::default();
        config.watch_paths.push("./removable_path".to_string());
        let initial_count = config.watch_paths.len();

        // Test removing existing path
        let result = config.remove_path("./removable_path");
        assert!(result.is_ok());
        assert_eq!(config.watch_paths.len(), initial_count - 1);
        assert!(!config.watch_paths.contains(&"./removable_path".to_string()));

        // Test removing non-existent path
        let result = config.remove_path("./non_existent_path");
        assert!(result.is_ok());
        assert_eq!(config.watch_paths.len(), initial_count - 1);
    }

    #[test]
    fn test_set_language() {
        let mut config = Config::default();

        // Test setting language
        let result = config.set_language(Some("en".to_string()));
        assert!(result.is_ok());
        assert_eq!(config.language, Some("en".to_string()));

        // Test setting None
        let result = config.set_language(None);
        assert!(result.is_ok());
        assert_eq!(config.language, None);
    }

    #[test]
    fn test_get_effective_language() {
        let mut config = Config::default();

        // Test with no language set (should return system locale or default)
        let effective = config.get_effective_language();
        assert!(effective == "en" || effective == "zh-cn");

        // Test with language set
        config.language = Some("zh-cn".to_string());
        assert_eq!(config.get_effective_language(), "zh-cn");

        config.language = Some("en".to_string());
        assert_eq!(config.get_effective_language(), "en");
    }

    #[test]
    fn test_get_effective_language_with_env() {
        let config = Config::default();

        // Test with LANG environment variable
        unsafe {
            env::set_var("LANG", "zh_CN.UTF-8");
        }
        assert_eq!(config.get_effective_language(), "zh-cn");

        unsafe {
            env::set_var("LANG", "en_US.UTF-8");
        }
        assert_eq!(config.get_effective_language(), "en");

        unsafe {
            env::set_var("LANG", "fr_FR.UTF-8");
        }
        assert_eq!(config.get_effective_language(), "en");

        // Clean up
        unsafe {
            env::remove_var("LANG");
        }
    }

    #[test]
    fn test_validate_paths() {
        let (mut config, temp_dir) = create_test_config_with_temp_dir();

        // Add some paths - one valid, one invalid
        let valid_path = temp_dir.path().to_string_lossy().to_string();
        let invalid_path = "/definitely/does/not/exist".to_string();

        config.watch_paths = vec![valid_path.clone(), invalid_path.clone()];

        let invalid_paths = config.validate_paths();
        assert_eq!(invalid_paths.len(), 1);
        assert_eq!(invalid_paths[0], invalid_path);

        // Test with all valid paths
        config.watch_paths = vec![valid_path];
        let invalid_paths = config.validate_paths();
        assert_eq!(invalid_paths.len(), 0);

        // Test with all invalid paths
        config.watch_paths = vec!["/invalid1".to_string(), "/invalid2".to_string()];
        let invalid_paths = config.validate_paths();
        assert_eq!(invalid_paths.len(), 2);
    }

    #[test]
    #[serial] // Serialize because we're dealing with config files
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create a test config
        let mut original_config = Config::default();
        original_config.watch_paths = vec!["./test1".to_string(), "./test2".to_string()];
        original_config.recursive = false;
        original_config.ignore_patterns = vec!["*.test".to_string()];
        original_config.language = Some("zh-cn".to_string());

        // Save config
        let yaml_content = serde_yaml_ng::to_string(&original_config).unwrap();
        fs::write(&config_path, yaml_content).unwrap();

        // Load config
        let content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = serde_yaml_ng::from_str(&content).unwrap();

        assert_eq!(original_config, loaded_config);
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::default();
        let config2 = config1.clone();
        assert_eq!(config1, config2);

        // Modify one and ensure they're different
        let mut config3 = config1.clone();
        config3.recursive = false;
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("watch_paths"));
        assert!(debug_str.contains("recursive"));
    }
}
