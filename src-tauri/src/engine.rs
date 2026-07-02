use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

type PendingMap = HashMap<u64, oneshot::Sender<Result<Value, String>>>;

pub struct EngineClient {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<ChildStdin>>,
    pending: Mutex<PendingMap>,
    app: AppHandle,
}

impl EngineClient {
    pub fn new(app: AppHandle) -> Self {
        Self {
            child: Mutex::new(None),
            stdin: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            app,
        }
    }

    fn sidecar_path(app: &AppHandle) -> Result<PathBuf, String> {
        if cfg!(debug_assertions) {
            if std::env::var("GAME_READER_USE_SIDECAR").is_ok() {
                // fall through to bundled sidecar lookup
            } else {
                let python = std::env::var("GAME_READER_PYTHON").unwrap_or_else(|_| {
                    if cfg!(windows) {
                        "python".into()
                    } else {
                        "python3".into()
                    }
                });
                let cwd = std::env::current_dir().unwrap_or_default();
                let service = cwd.join("engine").join("service.py");
                if service.exists() {
                    return Ok(PathBuf::from(python));
                }
            }
        }

        let candidates = [
            app.path()
                .resource_dir()
                .ok()
                .map(|d| d.join("binaries").join(sidecar_name())),
            app.path()
                .resource_dir()
                .ok()
                .map(|d| d.join(sidecar_name())),
            std::env::current_dir()
                .ok()
                .map(|d| d.join("src-tauri").join("binaries").join(sidecar_name())),
        ];

        for c in candidates.into_iter().flatten() {
            if c.exists() {
                return Ok(c);
            }
        }

        Err("Engine sidecar not found. Run scripts/build-engine.ps1 or use GAME_READER_PYTHON for dev.".into())
    }

    fn env_vars(app: &AppHandle) -> Vec<(String, String)> {
        crate::paths::ensure_dirs();
        let data = crate::paths::data_dir();
        let config = crate::paths::config_dir();
        let resource = app
            .path()
            .resource_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let repo_root = resource
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let mut vars = vec![
            ("GAME_READER_DATA_DIR".into(), data.to_string_lossy().into()),
            ("GAME_READER_CONFIG_DIR".into(), config.to_string_lossy().into()),
            ("GAME_READER_APP_DIR".into(), resource.to_string_lossy().into()),
            ("PYTHONPATH".into(), repo_root.to_string_lossy().into()),
        ];

        let tess = resource.join("tesseract").join("tesseract.exe");
        if tess.exists() {
            vars.push(("GAME_READER_TESSERACT".into(), tess.to_string_lossy().into()));
        }

        if std::env::var("GAME_READER_ALLOW_HF_DOWNLOAD").is_ok() {
            vars.push(("GAME_READER_ALLOW_HF_DOWNLOAD".into(), "1".into()));
        }

        vars
    }

    pub async fn ensure_running(&self) -> Result<(), String> {
        let mut guard = self.child.lock().await;
        if guard
            .as_mut()
            .map(|c| c.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
        {
            return Ok(());
        }

        let sidecar = Self::sidecar_path(&self.app)?;
        let env_vars = Self::env_vars(&self.app);

        let mut cmd = if is_python_interpreter(&sidecar) {
            let cwd = std::env::current_dir().unwrap_or_default();
            let service = cwd.join("engine").join("service.py");
            let mut c = Command::new(&sidecar);
            c.arg("-u").arg(service);
            c
        } else {
            Command::new(&sidecar)
        };

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start engine: {e}"))?;
        let stdin = child.stdin.take().ok_or("No engine stdin")?;
        let stdout = child.stdout.take().ok_or("No engine stdout")?;

        *self.stdin.lock().await = Some(stdin);
        *guard = Some(child);

        let pending = self.pending.clone();
        let app = self.app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(val) = serde_json::from_str::<Value>(&line) else {
                    let _ = app.emit("engine-log", line);
                    continue;
                };
                if let Some(id) = val.get("id").and_then(|v| v.as_u64()) {
                    let mut map = pending.lock().await;
                    if let Some(tx) = map.remove(&id) {
                        let result = if let Some(err) = val.get("error") {
                            Err(err
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Engine error")
                                .into())
                        } else {
                            Ok(val.get("result").cloned().unwrap_or(json!({})))
                        };
                        let _ = tx.send(result);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value, String> {
        self.ensure_running().await?;

        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(json!({})),
            "id": id
        });

        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard.as_mut().ok_or("Engine stdin unavailable")?;
            let line = format!("{req}\n");
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            stdin.flush().await.map_err(|e| e.to_string())?;
        }

        rx.await
            .map_err(|_| "Engine closed".into())?
    }

    pub async fn shutdown(&self) {
        let _ = self.call("shutdown", None).await;
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        *self.stdin.lock().await = None;
    }
}

fn sidecar_name() -> String {
    if cfg!(windows) {
        "game-reader-engine.exe".into()
    } else {
        "game-reader-engine".into()
    }
}

fn is_python_interpreter(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| {
            n == "python"
                || n == "python3"
                || n.ends_with("python.exe")
                || n.ends_with("python3.exe")
        })
        .unwrap_or(false)
}

pub type SharedEngine = std::sync::Arc<EngineClient>;
