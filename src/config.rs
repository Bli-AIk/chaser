use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub recursive: bool,
    pub ignore_patterns: Vec<String>,
    pub language: Option<String>,
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
            language: None, // Use system locale by default
        }
    }
}

impl Config {
    /// Get the config file path (cross-platform)
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;

        let app_config_dir = config_dir.join("chaser");

        // Create config directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir).context("Failed to create config directory")?;
        }

        Ok(app_config_dir.join("config.yaml"))
    }

    /// Load config from file, create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

            let config: Config =
                serde_yaml_ng::from_str(&content).context("Failed to parse config file")?;

            // Don't use i18n here as it might not be initialized yet
            eprintln!("{} {}", "✓".green(), format!("Loaded config from: {}", config_path.display()).bright_white());
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            eprintln!("{} {}", "✓".green(), format!("Created default config at: {}", config_path.display()).bright_white());
            Ok(default_config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        let content = serde_yaml_ng::to_string(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        // Use eprintln to avoid i18n dependency during early initialization
        eprintln!("{} {}", "✓".green(), format!("Config saved to: {}", config_path.display()).bright_white());
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
        println!("  {}: [{}]", "Ignore patterns".bright_white(), 
                 self.ignore_patterns.iter()
                     .map(|p| p.yellow().to_string())
                     .collect::<Vec<_>>()
                     .join(", "));

        if let Some(ref lang) = self.language {
            println!("  {}: {}", "Language".bright_white(), lang.green());
        } else {
            println!("  {}: {} {}", "Language".bright_white(), self.get_effective_language().green(), "(auto)".dimmed());
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
                crate::i18n::tf("msg_config_loaded", &[&config_path.display().to_string().cyan().to_string()]).green()
            );
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save_with_i18n()?;
            println!(
                "{}",
                crate::i18n::tf("msg_config_created", &[&config_path.display().to_string().cyan().to_string()]).green()
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
            crate::i18n::tf("msg_config_saved", &[&config_path.display().to_string().cyan().to_string()]).green()
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
}
