use std::path::PathBuf;

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use crate::paths;

const GITHUB_REPO: &str = "baylic/game-reader";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCatalogEntry {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub release_tag: String,
    pub asset: String,
    pub size_bytes: u64,
    pub f0up_key: i32,
    pub speed: f64,
    pub recommended: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub id: String,
    pub kind: String,
    pub downloaded: u64,
    pub total: u64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

pub fn load_catalog(app: &AppHandle) -> Result<Vec<VoiceCatalogEntry>, String> {
    let resource = app.path().resource_dir().map_err(|e| e.to_string())?;
    let candidates = [
        resource.join("voices.json"),
        resource.join("assets").join("voices.json"),
        PathBuf::from("assets").join("voices.json"),
    ];
    for path in candidates {
        if path.exists() {
            let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            return serde_json::from_str(&data).map_err(|e| e.to_string());
        }
    }
    Ok(default_catalog())
}

fn default_catalog() -> Vec<VoiceCatalogEntry> {
    serde_json::from_value(json!([
      {
        "id": "dagoth",
        "label": "Dagoth Ur",
        "description": "Trained RVC voice model from Morrowind",
        "release_tag": "v1.0",
        "asset": "dagoth_ur_v2.pth",
        "size_bytes": 55220472,
        "f0up_key": -12,
        "speed": 1.0
      },
      {
        "id": "narrator",
        "label": "Narrator",
        "description": "Trained RVC voice model of the Baldur's Gate 3 narrator",
        "release_tag": "v1.1",
        "asset": "bg3_narrator_v2.pth",
        "size_bytes": 55232492,
        "f0up_key": -6,
        "speed": 0.8,
        "recommended": true
      }
    ])).unwrap_or_default()
}

async fn release_asset_url(repo: &str, tag: &str, asset_name: &str) -> Result<(String, u64), String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let client = Client::builder()
        .user_agent("game-reader/2.0")
        .build()
        .map_err(|e| e.to_string())?;
    let release: Release = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    release
        .assets
        .into_iter()
        .find(|a| a.name == asset_name)
        .map(|a| (a.browser_download_url, a.size))
        .ok_or_else(|| format!("Asset {asset_name} not found in release {tag}"))
}

fn emit_progress(app: &AppHandle, progress: DownloadProgress) {
    let _ = app.emit("download-progress", progress);
}

async fn download_file(
    app: &AppHandle,
    id: &str,
    kind: &str,
    url: &str,
    dest: PathBuf,
    total: u64,
) -> Result<(), String> {
    paths::ensure_dirs();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let client = Client::builder()
        .user_agent("game-reader/2.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| e.to_string())?;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        emit_progress(
            app,
            DownloadProgress {
                id: id.into(),
                kind: kind.into(),
                downloaded,
                total,
                status: "downloading".into(),
                error: None,
            },
        );
    }

    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn download_voice(app: AppHandle, voice_id: String) -> Result<(), String> {
    let catalog = load_catalog(&app)?;
    let voice = catalog
        .into_iter()
        .find(|v| v.id == voice_id)
        .ok_or_else(|| format!("Unknown voice: {voice_id}"))?;

    let (url, size) = release_asset_url(GITHUB_REPO, &voice.release_tag, &voice.asset).await?;
    let dest = paths::voice_dir(&voice.id).join(&voice.asset);

    if let Err(e) = download_file(&app, &voice.id, "voice", &url, dest.clone(), size).await {
        emit_progress(
            &app,
            DownloadProgress {
                id: voice.id.clone(),
                kind: "voice".into(),
                downloaded: 0,
                total: size,
                status: "error".into(),
                error: Some(e.clone()),
            },
        );
        return Err(e);
    }

    update_manifest(&voice)?;

    emit_progress(
        &app,
        DownloadProgress {
            id: voice.id,
            kind: "voice".into(),
            downloaded: size,
            total: size,
            status: "complete".into(),
            error: None,
        },
    );

    Ok(())
}

fn update_manifest(voice: &VoiceCatalogEntry) -> Result<(), String> {
    paths::ensure_dirs();
    let manifest_path = paths::manifest_path();
    let mut manifest: serde_json::Value = if manifest_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?)
            .unwrap_or(json!({"voices": {}}))
    } else {
        json!({"voices": {}})
    };

    let model_path = paths::voice_dir(&voice.id).join(&voice.asset);
    manifest["voices"][&voice.id] = json!({
        "model": model_path.to_string_lossy(),
        "f0up_key": voice.f0up_key,
    });

    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

pub async fn delete_voice(voice_id: &str) -> Result<(), String> {
    let dir = paths::voice_dir(voice_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    let manifest_path = paths::manifest_path();
    if manifest_path.exists() {
        let mut manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?)
                .unwrap_or(json!({"voices": {}}));
        if let Some(voices) = manifest.get_mut("voices").and_then(|v| v.as_object_mut()) {
            voices.remove(voice_id);
        }
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Trigger Kokoro model download by calling HuggingFace hub via Python engine.
pub async fn download_kokoro(app: AppHandle, engine: crate::engine::SharedEngine) -> Result<(), String> {
    let total: u64 = 330_000_000;
    emit_progress(
        &app,
        DownloadProgress {
            id: "kokoro".into(),
            kind: "kokoro".into(),
            downloaded: 0,
            total,
            status: "downloading".into(),
            error: None,
        },
    );

    std::env::set_var("GAME_READER_ALLOW_HF_DOWNLOAD", "1");
    std::env::set_var("HF_HOME", paths::kokoro_dir().to_string_lossy().as_ref());
    std::env::set_var(
        "HUGGINGFACE_HUB_CACHE",
        paths::kokoro_dir().to_string_lossy().as_ref(),
    );

    let result = engine.call("download_kokoro", None).await;

    match result {
        Ok(_) => {
            emit_progress(
                &app,
                DownloadProgress {
                    id: "kokoro".into(),
                    kind: "kokoro".into(),
                    downloaded: total,
                    total,
                    status: "complete".into(),
                    error: None,
                },
            );
            Ok(())
        }
        Err(e) => {
            emit_progress(
                &app,
                DownloadProgress {
                    id: "kokoro".into(),
                    kind: "kokoro".into(),
                    downloaded: 0,
                    total,
                    status: "error".into(),
                    error: Some(e.clone()),
                },
            );
            Err(e)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRuntimeComponent {
    pub id: String,
    pub asset: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRuntimeManifest {
    pub github_repo: String,
    pub release_tag: String,
    pub label: String,
    pub description: String,
    pub size_bytes: u64,
    pub components: Vec<EngineRuntimeComponent>,
}

pub fn load_engine_runtime_manifest(app: &AppHandle) -> Result<EngineRuntimeManifest, String> {
    let resource = app.path().resource_dir().map_err(|e| e.to_string())?;
    let candidates = [
        resource.join("engine-runtime.json"),
        resource.join("assets").join("engine-runtime.json"),
        PathBuf::from("assets").join("engine-runtime.json"),
    ];
    for path in candidates {
        if path.exists() {
            let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            return serde_json::from_str(&data).map_err(|e| e.to_string());
        }
    }
    Err("engine-runtime.json not found".into())
}

pub fn get_engine_runtime_manifest(app: &AppHandle) -> Result<EngineRuntimeManifest, String> {
    load_engine_runtime_manifest(app)
}

pub async fn download_engine_runtime(app: AppHandle) -> Result<(), String> {
    let manifest = load_engine_runtime_manifest(&app)?;
    let repo = manifest.github_repo.clone();
    let tag = manifest.release_tag.clone();

    let mut component_sizes: Vec<(EngineRuntimeComponent, String, u64)> = Vec::new();
    let mut total: u64 = 0;
    for component in &manifest.components {
        let (url, size) = release_asset_url(&repo, &tag, &component.asset).await?;
        total += size;
        component_sizes.push((component.clone(), url, size));
    }

    let mut downloaded_total: u64 = 0;
    emit_progress(
        &app,
        DownloadProgress {
            id: "engine-runtime".into(),
            kind: "engine-runtime".into(),
            downloaded: 0,
            total,
            status: "downloading".into(),
            error: None,
        },
    );

    for (component, url, size) in component_sizes {
        let dest = paths::engine_bin_dir().join(&component.filename);
        let base_downloaded = downloaded_total;

        if let Err(e) = download_file_with_offset(
            &app,
            "engine-runtime",
            "engine-runtime",
            &url,
            dest,
            size,
            total,
            base_downloaded,
        )
        .await
        {
            emit_progress(
                &app,
                DownloadProgress {
                    id: "engine-runtime".into(),
                    kind: "engine-runtime".into(),
                    downloaded: base_downloaded,
                    total,
                    status: "error".into(),
                    error: Some(e.clone()),
                },
            );
            return Err(e);
        }

        downloaded_total += size;
    }

    emit_progress(
        &app,
        DownloadProgress {
            id: "engine-runtime".into(),
            kind: "engine-runtime".into(),
            downloaded: total,
            total,
            status: "complete".into(),
            error: None,
        },
    );

    Ok(())
}

async fn download_file_with_offset(
    app: &AppHandle,
    id: &str,
    kind: &str,
    url: &str,
    dest: PathBuf,
    file_total: u64,
    overall_total: u64,
    overall_base: u64,
) -> Result<(), String> {
    paths::ensure_dirs();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let client = Client::builder()
        .user_agent("game-reader/2.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let mut stream = response.bytes_stream();
    let mut file_downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| e.to_string())?;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file_downloaded += chunk.len() as u64;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        emit_progress(
            app,
            DownloadProgress {
                id: id.into(),
                kind: kind.into(),
                downloaded: overall_base + file_downloaded,
                total: overall_total,
                status: "downloading".into(),
                error: None,
            },
        );
    }

    file.flush().await.map_err(|e| e.to_string())?;
    let _ = file_total;
    Ok(())
}

pub async fn delete_engine_runtime() -> Result<(), String> {
    let dir = paths::engine_bin_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    paths::ensure_dirs();
    Ok(())
}
