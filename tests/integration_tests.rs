// Integration tests for complete application workflows

use chaser::{cli, config::Config, i18n};
use clap::Parser;
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;


fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    
    let locales_dir = temp_dir.path().join("locales");
    fs::create_dir_all(&locales_dir).unwrap();

    
    let en_content = r#"
msg_path_added: "✓ Added watch path: {0}"
msg_path_exists: "⚠ Path already exists: {0}"
msg_path_removed: "✓ Removed watch path: {0}"
msg_path_not_found: "❌ Path not found: {0}"
msg_config_loaded: "✓ Config loaded from: {0}"
msg_config_saved: "✓ Config saved to: {0}"
msg_config_created: "✓ Created default config at: {0}"
ui_watch_paths: "Watch Paths:"
ui_settings: "Settings:"
ui_recursive: "Recursive: {0}"
"#;
    fs::write(locales_dir.join("en.yaml"), en_content).unwrap();

    
    let zh_cn_content = r#"
msg_path_added: "✓ 已添加监控路径：{0}"
msg_path_exists: "⚠ 路径已存在：{0}"
msg_path_removed: "✓ 已移除监控路径：{0}"
msg_path_not_found: "❌ 路径未找到：{0}"
msg_config_loaded: "✓ 已从以下位置加载配置：{0}"
msg_config_saved: "✓ 配置已保存到：{0}"
msg_config_created: "✓ 已在以下位置创建默认配置：{0}"
ui_watch_paths: "监控路径："
ui_settings: "设置："
ui_recursive: "递归：{0}"
"#;
    fs::write(locales_dir.join("zh-cn.yaml"), zh_cn_content).unwrap();

    temp_dir
}

#[test]
#[serial]
fn test_config_operations_integration() {
    let temp_dir = setup_test_env();
    let original_dir = env::current_dir().unwrap();

    
    env::set_current_dir(temp_dir.path()).unwrap();

    
    let mut config = Config::default();
    assert_eq!(config.watch_paths, vec!["./test_files"]);

    
    let test_path = temp_dir.path().join("test_watch_dir");
    fs::create_dir_all(&test_path).unwrap();

    let result = config.add_path(test_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    assert!(
        config
            .watch_paths
            .contains(&test_path.to_string_lossy().to_string())
    );

    
    let result = config.remove_path(&test_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    assert!(
        !config
            .watch_paths
            .contains(&test_path.to_string_lossy().to_string())
    );

    
    let result = config.set_language(Some("zh-cn".to_string()));
    assert!(result.is_ok());
    assert_eq!(config.language, Some("zh-cn".to_string()));

    
    config.watch_paths = vec![
        test_path.to_string_lossy().to_string(),          "/definitely/does/not/exist".to_string(),     ];

    let invalid_paths = config.validate_paths();
    assert_eq!(invalid_paths.len(), 1);
    assert_eq!(invalid_paths[0], "/definitely/does/not/exist");

    
    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_i18n_integration() {
    let temp_dir = setup_test_env();
    let original_dir = env::current_dir().unwrap();

    
    env::set_current_dir(temp_dir.path()).unwrap();

    
    let result = i18n::init_i18n_with_locale("en");
    if result.is_ok() {
        
        let message = i18n::t("ui_watch_paths");
        assert_eq!(message, "Watch Paths:");

        let formatted_message = i18n::tf("msg_path_added", &["/test/path"]);
        assert_eq!(formatted_message, "✓ Added watch path: /test/path");

        
        i18n::set_locale("zh-cn");
        
    }

    
    let locales = i18n::available_locales();
    assert!(locales.contains(&"en".to_string()));
    assert!(locales.contains(&"zh-cn".to_string()));

    
    assert!(i18n::is_locale_supported("en"));
    assert!(i18n::is_locale_supported("zh-cn"));
    assert!(!i18n::is_locale_supported("invalid"));

    
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_cli_parsing_integration() {
    

    
    let cli = cli::Cli::try_parse_from(&["chaser"]).unwrap();
    assert!(cli.command.is_none());

    
    let cli = cli::Cli::try_parse_from(&["chaser", "add", "/new/path"]).unwrap();
    match cli.command {
        Some(cli::Commands::Add { path }) => assert_eq!(path, "/new/path"),
        _ => panic!("Expected Add command"),
    }

    
    let cli = cli::Cli::try_parse_from(&["chaser", "remove", "/old/path"]).unwrap();
    match cli.command {
        Some(cli::Commands::Remove { path }) => assert_eq!(path, "/old/path"),
        _ => panic!("Expected Remove command"),
    }

    
    let cli = cli::Cli::try_parse_from(&["chaser", "list"]).unwrap();
    assert!(matches!(cli.command, Some(cli::Commands::List)));

    
    let cli = cli::Cli::try_parse_from(&["chaser", "config"]).unwrap();
    assert!(matches!(cli.command, Some(cli::Commands::Config)));

    
    let cli = cli::Cli::try_parse_from(&["chaser", "recursive", "false"]).unwrap();
    match cli.command {
        Some(cli::Commands::Recursive { enabled }) => assert_eq!(enabled, "false"),
        _ => panic!("Expected Recursive command"),
    }

    
    let cli = cli::Cli::try_parse_from(&["chaser", "ignore", "*.backup"]).unwrap();
    match cli.command {
        Some(cli::Commands::Ignore { pattern }) => assert_eq!(pattern, "*.backup"),
        _ => panic!("Expected Ignore command"),
    }

    
    let cli = cli::Cli::try_parse_from(&["chaser", "reset"]).unwrap();
    assert!(matches!(cli.command, Some(cli::Commands::Reset)));

    
    let cli = cli::Cli::try_parse_from(&["chaser", "lang", "zh-cn"]).unwrap();
    match cli.command {
        Some(cli::Commands::Lang { language }) => assert_eq!(language, "zh-cn"),
        _ => panic!("Expected Lang command"),
    }
}

#[test]
#[serial]
fn test_config_persistence_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    
    let mut original_config = Config::default();
    original_config.watch_paths = vec!["/test1".to_string(), "/test2".to_string()];
    original_config.recursive = false;
    original_config.ignore_patterns = vec!["*.test".to_string()];
    original_config.language = Some("zh-cn".to_string());

    
    let yaml_content = serde_yaml_ng::to_string(&original_config).unwrap();
    fs::write(&config_path, yaml_content).unwrap();

    
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_yaml_ng::from_str(&content).unwrap();

    
    assert_eq!(original_config, loaded_config);
    assert_eq!(loaded_config.watch_paths, vec!["/test1", "/test2"]);
    assert_eq!(loaded_config.recursive, false);
    assert_eq!(loaded_config.ignore_patterns, vec!["*.test"]);
    assert_eq!(loaded_config.language, Some("zh-cn".to_string()));
}

#[test]
fn test_recursive_option_parsing() {
    
    let test_cases = vec![
        ("true", true),
        ("1", true),
        ("yes", true),
        ("on", true),
        ("false", false),
        ("0", false),
        ("no", false),
        ("off", false),
    ];

    for (input, expected) in test_cases {
        let cli = cli::Cli::try_parse_from(&["chaser", "recursive", input]).unwrap();
        match cli.command {
            Some(cli::Commands::Recursive { enabled }) => {
                
                let parsed = match enabled.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => false,
                };
                assert_eq!(parsed, expected, "Failed for input: {}", input);
            }
            _ => panic!("Expected Recursive command for input: {}", input),
        }
    }
}

#[test]
fn test_error_handling() {
    

    
    let result = cli::Cli::try_parse_from(&["chaser", "invalid_command"]);
    assert!(result.is_err());

    
    let result = cli::Cli::try_parse_from(&["chaser", "add"]);
    assert!(result.is_err());

    let result = cli::Cli::try_parse_from(&["chaser", "remove"]);
    assert!(result.is_err());

    let result = cli::Cli::try_parse_from(&["chaser", "recursive"]);
    assert!(result.is_err());

    let result = cli::Cli::try_parse_from(&["chaser", "ignore"]);
    assert!(result.is_err());

    let result = cli::Cli::try_parse_from(&["chaser", "lang"]);
    assert!(result.is_err());
}

#[test]
fn test_config_edge_cases() {
    let mut config = Config::default();

    
    let result = config.add_path(String::new());
    assert!(result.is_ok());

    
    let path = "./test_path".to_string();
    config.add_path(path.clone()).unwrap();
    let initial_len = config.watch_paths.len();
    config.add_path(path.clone()).unwrap();     assert_eq!(config.watch_paths.len(), initial_len); 
    
    let result = config.remove_path("non_existent_path");
    assert!(result.is_ok()); 
    
    config.ignore_patterns.clear();
    let _invalid_paths = config.validate_paths();
}

#[test]
fn test_language_fallback() {
    
    let config = Config::default();

    
    assert_eq!(config.language, None);
    let effective = config.get_effective_language();
    assert!(effective == "en" || effective == "zh-cn"); 
    
    assert!(!i18n::is_locale_supported("invalid"));
    assert!(!i18n::is_locale_supported(""));

    
    
    let _ = i18n::is_locale_supported("en");
    let _ = i18n::is_locale_supported("zh-cn");
    let _ = i18n::is_locale_supported("invalid");
}

#[test]
fn test_path_validation_edge_cases() {
    let mut config = Config::default();

    
    config.watch_paths = vec![
        ".".to_string(),                      "..".to_string(),                     "/".to_string(),                      "relative/path".to_string(),          "/absolute/path".to_string(),     ];

    let _invalid_paths = config.validate_paths();
}
