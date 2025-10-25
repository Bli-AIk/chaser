use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sys_locale::get_locale;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locale {
    strings: HashMap<String, String>,
}

pub struct I18n {
    current_locale: String,
    locales: HashMap<String, Locale>,
}

impl I18n {
    pub fn new() -> Result<Self> {
        let mut i18n = Self {
            current_locale: String::new(),
            locales: HashMap::new(),
        };

        // Load all available locales
        i18n.load_locales()?;

        // Set default locale
        i18n.set_locale(&Self::get_system_locale());

        Ok(i18n)
    }

    pub fn with_locale(locale: &str) -> Result<Self> {
        let mut i18n = Self::new()?;
        i18n.set_locale(locale);
        Ok(i18n)
    }

    fn load_locales(&mut self) -> Result<()> {
        let locale_dir = Path::new("locales");

        if !locale_dir.exists() {
            return Err(anyhow::anyhow!("Locales directory not found"));
        }

        for entry in fs::read_dir(locale_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(locale_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path).with_context(|| {
                        format!("Failed to read locale file: {}", path.display())
                    })?;

                    let strings: HashMap<String, String> = serde_yaml_ng::from_str(&content)
                        .with_context(|| {
                            format!("Failed to parse locale file: {}", path.display())
                        })?;

                    self.locales
                        .insert(locale_name.to_string(), Locale { strings });
                }
            }
        }

        Ok(())
    }

    pub fn set_locale(&mut self, locale: &str) {
        if self.locales.contains_key(locale) {
            self.current_locale = locale.to_string();
        } else {
            // Fallback to English if locale not found
            self.current_locale = "en".to_string();
        }
    }

    pub fn get_current_locale(&self) -> &str {
        &self.current_locale
    }

    pub fn available_locales(&self) -> Vec<&str> {
        self.locales.keys().map(|s| s.as_str()).collect()
    }

    pub fn t(&self, key: &str) -> String {
        if let Some(locale) = self.locales.get(&self.current_locale) {
            locale
                .strings
                .get(key)
                .map(|s| s.clone())
                .unwrap_or_else(|| key.to_string())
        } else {
            key.to_string()
        }
    }

    pub fn tf(&self, key: &str, args: &[&str]) -> String {
        let template = self.t(key);
        let mut result = template;

        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }

        result
    }

    fn get_system_locale() -> String {
        if let Some(locale) = get_locale() {
            // Convert system locale to our format
            let locale_lower = locale.to_lowercase();

            if locale_lower.starts_with("zh")
                && (locale_lower.contains("cn") || locale_lower.contains("hans"))
            {
                return "zh-cn".to_string();
            } else if locale_lower.starts_with("en") {
                return "en".to_string();
            }
        }

        // Default to English
        "en".to_string()
    }

    pub fn is_locale_supported(&self, locale: &str) -> bool {
        self.locales.contains_key(locale)
    }
}

// Global instance for easy access
use std::sync::{Mutex, OnceLock};

static I18N: OnceLock<Mutex<I18n>> = OnceLock::new();

pub fn init_i18n() -> Result<()> {
    let i18n = I18n::new()?;
    I18N.set(Mutex::new(i18n))
        .map_err(|_| anyhow::anyhow!("Failed to initialize i18n"))?;
    Ok(())
}

pub fn init_i18n_with_locale(locale: &str) -> Result<()> {
    let i18n = I18n::with_locale(locale)?;
    I18N.set(Mutex::new(i18n))
        .map_err(|_| anyhow::anyhow!("Failed to initialize i18n"))?;
    Ok(())
}

pub fn set_locale(locale: &str) {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(mut i18n) = i18n_mutex.lock() {
            i18n.set_locale(locale);
        }
    }
}

pub fn get_current_locale() -> String {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(i18n) = i18n_mutex.lock() {
            return i18n.get_current_locale().to_string();
        }
    }
    "en".to_string()
}

pub fn available_locales() -> Vec<String> {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(i18n) = i18n_mutex.lock() {
            return i18n
                .available_locales()
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
    }
    vec!["en".to_string()]
}

pub fn is_locale_supported(locale: &str) -> bool {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(i18n) = i18n_mutex.lock() {
            return i18n.is_locale_supported(locale);
        }
    }
    false
}

pub fn t(key: &str) -> String {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(i18n) = i18n_mutex.lock() {
            return i18n.t(key).to_string();
        }
    }
    key.to_string()
}

pub fn tf(key: &str, args: &[&str]) -> String {
    if let Some(i18n_mutex) = I18N.get() {
        if let Ok(i18n) = i18n_mutex.lock() {
            let template = i18n.t(key);
            let mut result = template;

            for (i, arg) in args.iter().enumerate() {
                result = result.replace(&format!("{{{}}}", i), arg);
            }

            return result;
        }
    }

    // Fallback: simple replacement for testing
    let mut result = key.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}
