use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub selected_monitor: String, // Identify by name

    // General
    #[serde(default = "default_false")]
    pub use_12h_format: bool,
    #[serde(default = "default_true")]
    pub show_seconds: bool,
    #[serde(default = "default_false")]
    pub pixelated: bool,

    // Appearance
    #[serde(default = "default_scale")]
    pub scale: f32, // 0.2 - 1.0 (20% - 100%)
    #[serde(default = "default_spacing")]
    pub spacing: f32, // 0.0 - 0.1 (0% - 10%)
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32, // 0.0 - 20.0

    // Theme
    #[serde(default = "default_bg_color")]
    pub bg_color: [f32; 3],
    #[serde(default = "default_card_color")]
    pub card_color: [f32; 3],
    #[serde(default = "default_text_color")]
    pub text_color: [f32; 3],
    #[serde(default = "default_animation_speed")]
    pub animation_speed: u64, // ms
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_scale() -> f32 { 0.85 }
fn default_spacing() -> f32 { 0.04 }
fn default_corner_radius() -> f32 { 8.0 }
fn default_bg_color() -> [f32; 3] { [0.125, 0.125, 0.125] } // #202020
fn default_card_color() -> [f32; 3] { [0.165, 0.165, 0.165] } // #2a2a2a
fn default_text_color() -> [f32; 3] { [0.898, 0.898, 0.898] } // #e5e5e5
fn default_animation_speed() -> u64 { 600 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_monitor: String::new(),
            use_12h_format: default_false(),
            show_seconds: default_true(),
            pixelated: default_false(),
            scale: default_scale(),
            spacing: default_spacing(),
            corner_radius: default_corner_radius(),
            bg_color: default_bg_color(),
            card_color: default_card_color(),
            text_color: default_text_color(),
            animation_speed: default_animation_speed(),
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
            use_12h_format: true,
            show_seconds: false,
            pixelated: true,
            scale: 0.9,
            spacing: 0.05,
            corner_radius: 10.0,
            bg_color: [0.1, 0.2, 0.3],
            card_color: [0.4, 0.5, 0.6],
            text_color: [0.7, 0.8, 0.9],
            animation_speed: 500,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("TestMonitor"));
        assert!(json.contains("pixelated"));
        assert!(json.contains("bg_color"));

        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.selected_monitor, "TestMonitor");
        assert_eq!(loaded.pixelated, true);
        assert_eq!(loaded.bg_color, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.selected_monitor, "");
        assert_eq!(config.pixelated, false);
        assert_eq!(config.use_12h_format, false);
        assert_eq!(config.scale, 0.85);
    }
}
