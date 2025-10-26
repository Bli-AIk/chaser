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

        let updated_content = match self.format {
            TargetFileFormat::Json => self.update_json_content(&content, old_path, new_path)?,
            TargetFileFormat::Yaml => self.update_yaml_content(&content, old_path, new_path)?,
            TargetFileFormat::Toml => self.update_toml_content(&content, old_path, new_path)?,
            TargetFileFormat::Csv => self.update_csv_content(&content, old_path, new_path)?,
        };

        fs::write(&self.path, updated_content)?;
        Ok(())
    }

    fn update_json_content(&self, content: &str, old_path: &str, new_path: &str) -> Result<String> {
        let mut value: JsonValue = serde_json::from_str(content)?;
        Self::update_json_value(&mut value, old_path, new_path);
        Ok(serde_json::to_string_pretty(&value)?)
    }

    fn update_json_value(value: &mut JsonValue, old_path: &str, new_path: &str) {
        match value {
            JsonValue::String(s) => {
                if s == old_path {
                    *s = new_path.to_string();
                }
            }
            JsonValue::Array(arr) => {
                for item in arr {
                    Self::update_json_value(item, old_path, new_path);
                }
            }
            JsonValue::Object(obj) => {
                for (_, v) in obj {
                    Self::update_json_value(v, old_path, new_path);
                }
            }
            _ => {}
        }
    }

    fn update_yaml_content(&self, content: &str, old_path: &str, new_path: &str) -> Result<String> {
        let mut value: YamlValue = serde_yaml_ng::from_str(content)?;
        Self::update_yaml_value(&mut value, old_path, new_path);
        Ok(serde_yaml_ng::to_string(&value)?)
    }

    fn update_yaml_value(value: &mut YamlValue, old_path: &str, new_path: &str) {
        match value {
            YamlValue::String(s) => {
                if s == old_path {
                    *s = new_path.to_string();
                }
            }
            YamlValue::Sequence(seq) => {
                for item in seq {
                    Self::update_yaml_value(item, old_path, new_path);
                }
            }
            YamlValue::Mapping(map) => {
                for (_, v) in map {
                    Self::update_yaml_value(v, old_path, new_path);
                }
            }
            _ => {}
        }
    }

    fn update_toml_content(&self, content: &str, old_path: &str, new_path: &str) -> Result<String> {
        let mut value: TomlValue = toml::from_str(content)?;
        Self::update_toml_value(&mut value, old_path, new_path);
        Ok(toml::to_string_pretty(&value)?)
    }

    fn update_toml_value(value: &mut TomlValue, old_path: &str, new_path: &str) {
        match value {
            TomlValue::String(s) => {
                if s == old_path {
                    *s = new_path.to_string();
                }
            }
            TomlValue::Array(arr) => {
                for item in arr {
                    Self::update_toml_value(item, old_path, new_path);
                }
            }
            TomlValue::Table(table) => {
                for (_, v) in table {
                    Self::update_toml_value(v, old_path, new_path);
                }
            }
            _ => {}
        }
    }

    fn update_csv_content(&self, content: &str, old_path: &str, new_path: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(content.to_string());
        }

        let mut updated_lines = Vec::new();
        updated_lines.push(lines[0].to_string()); // Keep header

        for line in &lines[1..] {
            if line.starts_with(old_path) {
                // Replace the path at the beginning of the line
                let remaining = &line[old_path.len()..];
                updated_lines.push(format!("{}{}", new_path, remaining));
            } else {
                updated_lines.push(line.to_string());
            }
        }

        Ok(updated_lines.join("\n") + "\n")
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
            TargetFileFormat::from_path(Path::new("test.yml")).unwrap(),
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
        assert!(TargetFileFormat::from_path(Path::new("test.txt")).is_err()); // Unsupported format
    }

    #[test]
    fn test_looks_like_path() {
        assert!(TargetFile::looks_like_path("./test_files/test.txt"));
        assert!(TargetFile::looks_like_path("/home/user/test.txt"));
        assert!(TargetFile::looks_like_path("../relative/path"));
        assert!(TargetFile::looks_like_path("simple/path"));
        assert!(TargetFile::looks_like_path("C:\\Windows\\System32"));
        assert!(!TargetFile::looks_like_path("not a path"));
        assert!(!TargetFile::looks_like_path("123456"));
        assert!(!TargetFile::looks_like_path("config_option"));
    }

    #[test]
    fn test_extract_paths_from_json() {
        let json_content = r#"[
            "./test_files/file1.txt",
            "./test_files/dir",
            "not a path",
            "/absolute/path"
        ]"#;

        let paths = TargetFile::extract_paths_from_json(json_content).unwrap();
        assert_eq!(paths.len(), 3);
        assert!(paths.iter().any(|p| p.path == "./test_files/file1.txt"));
        assert!(paths.iter().any(|p| p.path == "./test_files/dir"));
        assert!(paths.iter().any(|p| p.path == "/absolute/path"));
    }

    #[test]
    fn test_extract_paths_from_yaml() {
        let yaml_content = r#"
paths:
  - "./test_files/file1.txt"
  - "./test_files/dir"
  - "/absolute/path"
other_field: "value"
"#;

        let paths = TargetFile::extract_paths_from_yaml(yaml_content).unwrap();
        assert_eq!(paths.len(), 3);
        assert!(paths.iter().any(|p| p.path == "./test_files/file1.txt"));
        assert!(paths.iter().any(|p| p.path == "./test_files/dir"));
        assert!(paths.iter().any(|p| p.path == "/absolute/path"));
    }

    #[test]
    fn test_extract_paths_from_toml() {
        let toml_content = r#"
paths = ["./test_files/file1.txt", "./test_files/dir", "/absolute/path"]
other_field = "value"
"#;

        let paths = TargetFile::extract_paths_from_toml(toml_content).unwrap();
        assert_eq!(paths.len(), 3);
        assert!(paths.iter().any(|p| p.path == "./test_files/file1.txt"));
        assert!(paths.iter().any(|p| p.path == "./test_files/dir"));
        assert!(paths.iter().any(|p| p.path == "/absolute/path"));
    }

    #[test]
    fn test_extract_paths_from_csv() {
        let csv_content = r#"path,type,description
./test_files/file1.txt,file,Test file
./test_files/dir,directory,Test directory
/absolute/path,file,Absolute path
"#;

        let paths = TargetFile::extract_paths_from_csv(csv_content).unwrap();
        assert_eq!(paths.len(), 3);
        assert!(paths.iter().any(|p| p.path == "./test_files/file1.txt"));
        assert!(paths.iter().any(|p| p.path == "./test_files/dir"));
        assert!(paths.iter().any(|p| p.path == "/absolute/path"));
    }

    #[test]
    fn test_json_file_path_update() {
        let temp_dir = TempDir::new().unwrap();
        let json_file = temp_dir.path().join("test.json");

        let initial_content = r#"["./test_files/old_path", "./test_files/keep_path"]"#;
        fs::write(&json_file, initial_content).unwrap();

        let mut target_file = TargetFile::new(json_file.clone()).unwrap();
        target_file
            .update_path("./test_files/old_path", "./test_files/new_path")
            .unwrap();

        let updated_content = fs::read_to_string(&json_file).unwrap();
        assert!(updated_content.contains("./test_files/new_path"));
        assert!(updated_content.contains("./test_files/keep_path"));
        assert!(!updated_content.contains("./test_files/old_path"));
    }

    #[test]
    fn test_yaml_file_path_update() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_file = temp_dir.path().join("test.yaml");

        let initial_content = r#"paths:
  - "./test_files/old_path"
  - "./test_files/keep_path"
other_field: "value"
"#;
        fs::write(&yaml_file, initial_content).unwrap();

        let mut target_file = TargetFile::new(yaml_file.clone()).unwrap();
        target_file
            .update_path("./test_files/old_path", "./test_files/new_path")
            .unwrap();

        let updated_content = fs::read_to_string(&yaml_file).unwrap();

        assert!(updated_content.contains("./test_files/new_path"));
        assert!(updated_content.contains("./test_files/keep_path"));
        assert!(!updated_content.contains("./test_files/old_path"));
        assert!(updated_content.contains("other_field")); // YAML formatting might change quotes
    }

    #[test]
    fn test_toml_file_path_update() {
        let temp_dir = TempDir::new().unwrap();
        let toml_file = temp_dir.path().join("test.toml");

        let initial_content = r#"paths = ["./test_files/old_path", "./test_files/keep_path"]
other_field = "value"
"#;
        fs::write(&toml_file, initial_content).unwrap();

        let mut target_file = TargetFile::new(toml_file.clone()).unwrap();
        target_file
            .update_path("./test_files/old_path", "./test_files/new_path")
            .unwrap();

        let updated_content = fs::read_to_string(&toml_file).unwrap();
        assert!(updated_content.contains("./test_files/new_path"));
        assert!(updated_content.contains("./test_files/keep_path"));
        assert!(!updated_content.contains("./test_files/old_path"));
        assert!(updated_content.contains("other_field = \"value\""));
    }

    #[test]
    fn test_csv_file_path_update() {
        let temp_dir = TempDir::new().unwrap();
        let csv_file = temp_dir.path().join("test.csv");

        let initial_content = r#"path,type,description
./test_files/old_path,file,Old file
./test_files/keep_path,directory,Keep this
"#;
        fs::write(&csv_file, initial_content).unwrap();

        let mut target_file = TargetFile::new(csv_file.clone()).unwrap();
        target_file
            .update_path("./test_files/old_path", "./test_files/new_path")
            .unwrap();

        let updated_content = fs::read_to_string(&csv_file).unwrap();
        assert!(updated_content.contains("./test_files/new_path"));
        assert!(updated_content.contains("./test_files/keep_path"));
        assert!(!updated_content.contains("./test_files/old_path"));
        assert!(updated_content.contains("path,type,description"));
    }

    #[test]
    fn test_complex_path_scenarios() {
        let temp_dir = TempDir::new().unwrap();

        // Test with paths that are substrings of each other
        let json_file = temp_dir.path().join("test.json");
        let initial_content =
            r#"["./test_files/path", "./test_files/path_extended", "./test_files/other"]"#;
        fs::write(&json_file, initial_content).unwrap();

        let mut target_file = TargetFile::new(json_file.clone()).unwrap();
        target_file
            .update_path("./test_files/path", "./test_files/renamed")
            .unwrap();

        let updated_content = fs::read_to_string(&json_file).unwrap();
        assert!(updated_content.contains("./test_files/renamed"));
        assert!(updated_content.contains("./test_files/path_extended")); // Should not be changed
        assert!(updated_content.contains("./test_files/other"));
        assert!(!updated_content.contains("\"./test_files/path\"")); // Exact match should be gone
    }

    #[test]
    fn test_mixed_file_formats() {
        let temp_dir = TempDir::new().unwrap();

        // Create files in different formats with same paths
        let json_file = temp_dir.path().join("test.json");
        let yaml_file = temp_dir.path().join("test.yaml");
        let toml_file = temp_dir.path().join("test.toml");
        let csv_file = temp_dir.path().join("test.csv");

        fs::write(&json_file, r#"["./test_files/shared_path"]"#).unwrap();
        fs::write(&yaml_file, "paths:\n  - \"./test_files/shared_path\"").unwrap();
        fs::write(&toml_file, "paths = [\"./test_files/shared_path\"]").unwrap();
        fs::write(&csv_file, "path,type\n./test_files/shared_path,file").unwrap();

        let mut json_target = TargetFile::new(json_file.clone()).unwrap();
        let mut yaml_target = TargetFile::new(yaml_file.clone()).unwrap();
        let mut toml_target = TargetFile::new(toml_file.clone()).unwrap();
        let mut csv_target = TargetFile::new(csv_file.clone()).unwrap();

        // Update all files
        json_target
            .update_path("./test_files/shared_path", "./test_files/updated_path")
            .unwrap();
        yaml_target
            .update_path("./test_files/shared_path", "./test_files/updated_path")
            .unwrap();
        toml_target
            .update_path("./test_files/shared_path", "./test_files/updated_path")
            .unwrap();
        csv_target
            .update_path("./test_files/shared_path", "./test_files/updated_path")
            .unwrap();

        // Verify all formats were updated
        let json_content = fs::read_to_string(&json_file).unwrap();
        let yaml_content = fs::read_to_string(&yaml_file).unwrap();
        let toml_content = fs::read_to_string(&toml_file).unwrap();
        let csv_content = fs::read_to_string(&csv_file).unwrap();

        assert!(json_content.contains("./test_files/updated_path"));
        assert!(yaml_content.contains("./test_files/updated_path"));
        assert!(toml_content.contains("./test_files/updated_path"));
        assert!(csv_content.contains("./test_files/updated_path"));

        assert!(!json_content.contains("./test_files/shared_path"));
        assert!(!yaml_content.contains("./test_files/shared_path"));
        assert!(!toml_content.contains("./test_files/shared_path"));
        assert!(!csv_content.contains("./test_files/shared_path"));
    }
}
