use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub selected_monitor: String, // Identify by name
    #[serde(default)]
    pub pixelated: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_monitor: String::new(),
            pixelated: false,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rust_flip_rs", "rust_flip_clock") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        config_dir.join("config.json")
    } else {
        PathBuf::from("config.json")
    }
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            selected_monitor: "TestMonitor".to_string(),
            pixelated: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("TestMonitor"));
        assert!(json.contains("pixelated"));

        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.selected_monitor, "TestMonitor");
        assert_eq!(loaded.pixelated, true);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.selected_monitor, "");
        assert_eq!(config.pixelated, false);
    }
}
