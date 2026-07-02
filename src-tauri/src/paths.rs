use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GameReader")
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("GameReader")
}

pub fn models_dir() -> PathBuf {
    data_dir().join("models")
}

pub fn voice_dir(voice_id: &str) -> PathBuf {
    models_dir().join(voice_id)
}

pub fn manifest_path() -> PathBuf {
    models_dir().join("manifest.json")
}

pub fn kokoro_dir() -> PathBuf {
    models_dir().join("kokoro")
}

pub fn ensure_dirs() {
    let _ = std::fs::create_dir_all(models_dir());
    let _ = std::fs::create_dir_all(config_dir());
}

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            total += dir_size(&entry.path());
        }
    }
    total
}

pub fn storage_usage() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    map.insert("kokoro".into(), dir_size(&kokoro_dir()));
    if let Ok(entries) = std::fs::read_dir(models_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name != "kokoro" {
                        map.insert(format!("voice:{name}"), dir_size(&path));
                    }
                }
            }
        }
    }
    map
}
