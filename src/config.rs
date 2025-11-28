use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub auto_hide_delay: u64,
    pub refresh_interval: u64,
    pub show_battery_levels: bool,
    pub show_device_addresses: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_hide_delay: 100,
            refresh_interval: 5000,
            show_battery_levels: true,
            show_device_addresses: true,
            window_width: 300,
            window_height: 400,
            theme: "auto".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if let Some(path) = &config_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }

        let config = Self::default();
        config.save().ok(); // Try to save default config
        config
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::get_config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    fn get_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "bluewidget", "bluetooth-widget")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }
}