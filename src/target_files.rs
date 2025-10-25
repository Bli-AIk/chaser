use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Clone, PartialEq)]
pub enum TargetFileFormat {
    Json,
    Yaml,
    Toml,
    Csv,
}

impl TargetFileFormat {
    pub fn from_path(path: &Path) -> Result<Self> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => Ok(Self::Json),
            Some("yaml") | Some("yml") => Ok(Self::Yaml),
            Some("toml") => Ok(Self::Toml),
            Some("csv") => Ok(Self::Csv),
            _ => anyhow::bail!("Unsupported file format for: {:?}", path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: String,
    pub exists: bool,
    pub last_known_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TargetFile {
    pub path: PathBuf,
    pub format: TargetFileFormat,
    pub paths: Vec<PathEntry>,
}

impl TargetFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        let format = TargetFileFormat::from_path(&path)?;
        let paths = Self::extract_paths(&path, &format)?;

        Ok(Self {
            path,
            format,
            paths,
        })
    }

    /// Extract all paths from the target file
    fn extract_paths(file_path: &Path, format: &TargetFileFormat) -> Result<Vec<PathEntry>> {
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        match format {
            TargetFileFormat::Json => Self::extract_paths_from_json(&content),
            TargetFileFormat::Yaml => Self::extract_paths_from_yaml(&content),
            TargetFileFormat::Toml => Self::extract_paths_from_toml(&content),
            TargetFileFormat::Csv => Self::extract_paths_from_csv(&content),
        }
    }

    fn extract_paths_from_json(content: &str) -> Result<Vec<PathEntry>> {
        let value: JsonValue = serde_json::from_str(content)?;
        let mut paths = Vec::new();
        Self::collect_paths_from_json_value(&value, &mut paths);
        Ok(paths
            .into_iter()
            .map(|p| PathEntry {
                path: p.clone(),
                exists: Path::new(&p).exists(),
                last_known_path: None,
            })
            .collect())
    }

    fn collect_paths_from_json_value(value: &JsonValue, paths: &mut Vec<String>) {
        match value {
            JsonValue::String(s) => {
                if Self::looks_like_path(s) {
                    paths.push(s.clone());
                }
            }
            JsonValue::Array(arr) => {
                for item in arr {
                    Self::collect_paths_from_json_value(item, paths);
                }
            }
            JsonValue::Object(obj) => {
                for (_, v) in obj {
                    Self::collect_paths_from_json_value(v, paths);
                }
            }
            _ => {}
        }
    }

    fn extract_paths_from_yaml(content: &str) -> Result<Vec<PathEntry>> {
        let value: YamlValue = serde_yaml_ng::from_str(content)?;
        let mut paths = Vec::new();
        Self::collect_paths_from_yaml_value(&value, &mut paths);
        Ok(paths
            .into_iter()
            .map(|p| PathEntry {
                path: p.clone(),
                exists: Path::new(&p).exists(),
                last_known_path: None,
            })
            .collect())
    }

    fn collect_paths_from_yaml_value(value: &YamlValue, paths: &mut Vec<String>) {
        match value {
            YamlValue::String(s) => {
                if Self::looks_like_path(s) {
                    paths.push(s.clone());
                }
            }
            YamlValue::Sequence(seq) => {
                for item in seq {
                    Self::collect_paths_from_yaml_value(item, paths);
                }
            }
            YamlValue::Mapping(map) => {
                for (_, v) in map {
                    Self::collect_paths_from_yaml_value(v, paths);
                }
            }
            _ => {}
        }
    }

    fn extract_paths_from_toml(content: &str) -> Result<Vec<PathEntry>> {
        let value: TomlValue = toml::from_str(content)?;
        let mut paths = Vec::new();
        Self::collect_paths_from_toml_value(&value, &mut paths);
        Ok(paths
            .into_iter()
            .map(|p| PathEntry {
                path: p.clone(),
                exists: Path::new(&p).exists(),
                last_known_path: None,
            })
            .collect())
    }

    fn collect_paths_from_toml_value(value: &TomlValue, paths: &mut Vec<String>) {
        match value {
            TomlValue::String(s) => {
                if Self::looks_like_path(s) {
                    paths.push(s.clone());
                }
            }
            TomlValue::Array(arr) => {
                for item in arr {
                    Self::collect_paths_from_toml_value(item, paths);
                }
            }
            TomlValue::Table(table) => {
                for (_, v) in table {
                    Self::collect_paths_from_toml_value(v, paths);
                }
            }
            _ => {}
        }
    }

    fn extract_paths_from_csv(content: &str) -> Result<Vec<PathEntry>> {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut paths = Vec::new();

        for result in reader.records() {
            let record = result?;
            for field in record.iter() {
                if Self::looks_like_path(field) {
                    paths.push(field.to_string());
                }
            }
        }

        Ok(paths
            .into_iter()
            .map(|p| PathEntry {
                path: p.clone(),
                exists: Path::new(&p).exists(),
                last_known_path: None,
            })
            .collect())
    }

    /// Check if a string looks like a file/directory path
    fn looks_like_path(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        // Check for common path patterns
        s.contains('/')
            || s.contains('\\')
            || s.starts_with("./")
            || s.starts_with("../")
            || s.starts_with("~/")
            || s.starts_with('/')
            || (cfg!(windows) && s.len() > 2 && s.chars().nth(1) == Some(':'))
    }

    /// Update a path in the target file
    pub fn update_path(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        // Update internal path tracking
        for entry in &mut self.paths {
            if entry.path == old_path {
                entry.last_known_path = Some(entry.path.clone());
                entry.path = new_path.to_string();
                entry.exists = Path::new(new_path).exists();
            }
        }

        // Update the actual file content
        self.update_file_content(old_path, new_path)
    }

    fn update_file_content(&self, old_path: &str, new_path: &str) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)?;
        let updated_content = content.replace(old_path, new_path);

        fs::write(&self.path, updated_content)?;
        Ok(())
    }

    /// Mark a path as deleted (but keep tracking it)
    pub fn mark_path_deleted(&mut self, path: &str) -> Result<()> {
        for entry in &mut self.paths {
            if entry.path == path {
                entry.exists = false;
            }
        }
        Ok(())
    }

    /// Mark a path as restored
    pub fn mark_path_restored(&mut self, path: &str) -> Result<()> {
        for entry in &mut self.paths {
            if entry.path == path {
                entry.exists = true;
            }
        }
        Ok(())
    }

    /// Get all valid (existing) paths
    pub fn get_valid_paths(&self) -> Vec<&String> {
        self.paths
            .iter()
            .filter(|entry| entry.exists)
            .map(|entry| &entry.path)
            .collect()
    }

    /// Get all tracked paths (including deleted ones)
    pub fn get_all_paths(&self) -> Vec<&String> {
        self.paths.iter().map(|entry| &entry.path).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_target_file_format_detection() {
        assert_eq!(
            TargetFileFormat::from_path(Path::new("test.json")).unwrap(),
            TargetFileFormat::Json
        );
        assert_eq!(
            TargetFileFormat::from_path(Path::new("test.yaml")).unwrap(),
            TargetFileFormat::Yaml
        );
        assert_eq!(
            TargetFileFormat::from_path(Path::new("test.toml")).unwrap(),
            TargetFileFormat::Toml
        );
        assert_eq!(
            TargetFileFormat::from_path(Path::new("test.csv")).unwrap(),
            TargetFileFormat::Csv
        );
    }

    #[test]
    fn test_looks_like_path() {
        assert!(TargetFile::looks_like_path("/absolute/path"));
        assert!(TargetFile::looks_like_path("./relative/path"));
        assert!(TargetFile::looks_like_path("../parent/path"));
        assert!(TargetFile::looks_like_path("~/home/path"));
        assert!(TargetFile::looks_like_path("folder/subfolder"));

        assert!(!TargetFile::looks_like_path(""));
        assert!(!TargetFile::looks_like_path("simple_string"));
        assert!(!TargetFile::looks_like_path("123"));
    }

    #[test]
    fn test_extract_paths_from_json() {
        let json_content = r#"{
            "paths": ["/home/user/docs", "./temp"],
            "config": {
                "data_dir": "/var/data",
                "name": "test"
            }
        }"#;

        let paths = TargetFile::extract_paths_from_json(json_content).unwrap();
        let path_strings: Vec<String> = paths.iter().map(|p| p.path.clone()).collect();

        assert!(path_strings.contains(&"/home/user/docs".to_string()));
        assert!(path_strings.contains(&"./temp".to_string()));
        assert!(path_strings.contains(&"/var/data".to_string()));
        assert!(!path_strings.contains(&"test".to_string()));
    }

    #[test]
    fn test_path_update() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let json_content = r#"{"paths": ["/old/path", "/other/path"]}"#;
        fs::write(&file_path, json_content).unwrap();

        let mut target_file = TargetFile::new(file_path.clone()).unwrap();
        target_file.update_path("/old/path", "/new/path").unwrap();

        let updated_content = fs::read_to_string(&file_path).unwrap();
        assert!(updated_content.contains("/new/path"));
        assert!(!updated_content.contains("/old/path"));
        assert!(updated_content.contains("/other/path"));
    }
}
