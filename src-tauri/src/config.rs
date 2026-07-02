use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub region: Option<Value>,
    #[serde(default = "default_hotkey_select")]
    pub hotkey_select: String,
    #[serde(default = "default_hotkey_read")]
    pub hotkey_read: String,
    #[serde(default = "default_hotkey_stop")]
    pub hotkey_stop: String,
    #[serde(default = "default_hotkey_quit")]
    pub hotkey_quit: String,
    #[serde(default = "default_hotkey_cycle")]
    pub hotkey_cycle: String,
    #[serde(default = "default_tts_voice")]
    pub tts_voice: String,
    #[serde(default = "default_tts_speed")]
    pub tts_speed: f64,
    #[serde(default = "default_active_voice")]
    pub active_voice: String,
    #[serde(default = "default_prefetch")]
    pub prefetch_ocr: bool,
}

fn default_hotkey_select() -> String {
    "ctrl+shift+r".into()
}
fn default_hotkey_read() -> String {
    "ctrl+shift+t".into()
}
fn default_hotkey_stop() -> String {
    "ctrl+shift+s".into()
}
fn default_hotkey_quit() -> String {
    "ctrl+shift+q".into()
}
fn default_hotkey_cycle() -> String {
    "ctrl+shift+v".into()
}
fn default_tts_voice() -> String {
    "bf_emma".into()
}
fn default_tts_speed() -> f64 {
    1.0
}
fn default_active_voice() -> String {
    "narrator".into()
}
fn default_prefetch() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            region: None,
            hotkey_select: default_hotkey_select(),
            hotkey_read: default_hotkey_read(),
            hotkey_stop: default_hotkey_stop(),
            hotkey_quit: default_hotkey_quit(),
            hotkey_cycle: default_hotkey_cycle(),
            tts_voice: default_tts_voice(),
            tts_speed: default_tts_speed(),
            active_voice: default_active_voice(),
            prefetch_ocr: default_prefetch(),
        }
    }
}

pub fn config_path() -> PathBuf {
    paths::config_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    paths::ensure_dirs();
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    paths::ensure_dirs();
    let text = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path(), text).map_err(|e| e.to_string())
}

pub fn config_to_value(config: &AppConfig) -> Value {
    serde_json::to_value(config).unwrap_or(json!({}))
}

pub fn config_from_value(value: Value) -> Result<AppConfig, String> {
    serde_json::from_value(value).map_err(|e| e.to_string())
}
