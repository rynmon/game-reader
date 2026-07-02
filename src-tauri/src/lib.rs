use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::ShortcutState;

mod config;
mod downloads;
mod engine;
mod hotkeys;
mod paths;

use config::{config_from_value, config_to_value, load_config, save_config, AppConfig};
use engine::{EngineClient, SharedEngine};
use hotkeys::{register_hotkeys, HotkeyBindings};
use std::sync::Mutex;

pub struct AppState {
    pub engine: SharedEngine,
    pub hotkeys: Mutex<HotkeyBindings>,
}

#[tauri::command]
async fn init_engine(state: State<'_, AppState>) -> Result<Value, String> {
    state.engine.call("init", None).await
}

#[tauri::command]
async fn get_engine_status(state: State<'_, AppState>) -> Result<Value, String> {
    state.engine.call("get_status", None).await
}

#[tauri::command]
async fn read_region(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.call("read_region", None).await?;
    Ok(())
}

#[tauri::command]
async fn stop_speech(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.call("stop", None).await?;
    Ok(())
}

#[tauri::command]
async fn cycle_voice(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.call("cycle_voice", None).await?;
    Ok(())
}

#[tauri::command]
async fn set_active_voice(voice_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .engine
        .call(
            "set_voice",
            Some(serde_json::json!({ "voice_id": voice_id })),
        )
        .await?;
    Ok(())
}

#[tauri::command]
async fn get_gpu_info(state: State<'_, AppState>) -> Result<Value, String> {
    state.engine.call("get_gpu_info", None).await
}

#[tauri::command]
fn get_settings() -> Result<Value, String> {
    Ok(config_to_value(&load_config()))
}

#[tauri::command]
fn format_hotkey(hotkey: String) -> String {
    hotkeys::format_hotkey(&hotkey)
}

#[tauri::command]
async fn save_settings(
    settings: Value,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let config: AppConfig = config_from_value(settings)?;
    save_config(&config)?;

    let bindings = register_hotkeys(&app, &config)?;
    *state.hotkeys.lock().map_err(|e| e.to_string())? = bindings;

    let value = config_to_value(&config);
    let _ = state.engine.call("save_config", Some(value.clone())).await;
    Ok(value)
}

#[tauri::command]
async fn complete_region_selection(
    region: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(r) = region {
        state.engine.call("save_region", Some(r)).await?;
    }
    Ok(())
}

#[tauri::command]
async fn open_region_selector(app: AppHandle) -> Result<(), String> {
    hotkeys::open_region_selector(app).await
}

#[tauri::command]
fn get_voice_catalog(app: AppHandle) -> Result<Vec<downloads::VoiceCatalogEntry>, String> {
    downloads::load_catalog(&app)
}

#[tauri::command]
async fn download_voice(
    voice_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    downloads::download_voice(app.clone(), voice_id).await?;
    state.engine.call("reload_voices", None).await?;
    Ok(())
}

#[tauri::command]
async fn delete_voice(voice_id: String, state: State<'_, AppState>) -> Result<(), String> {
    downloads::delete_voice(&voice_id).await?;
    state.engine.call("reload_voices", None).await?;
    Ok(())
}

#[tauri::command]
async fn download_kokoro(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    downloads::download_kokoro(app, state.engine.clone()).await
}

#[tauri::command]
fn get_storage_usage() -> Result<std::collections::HashMap<String, u64>, String> {
    Ok(paths::storage_usage())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    let bindings = state.hotkeys.lock().unwrap();
                    hotkeys::handle_hotkey(app, shortcut, &bindings);
                })
                .build(),
        )
        .setup(|app| {
            paths::ensure_dirs();

            let engine = Arc::new(EngineClient::new(app.handle().clone()));
            let config = load_config();
            let bindings = register_hotkeys(app.handle(), &config).unwrap_or_else(|e| {
                eprintln!("Failed to register hotkeys: {e}");
                HotkeyBindings::empty()
            });

            app.manage(AppState {
                engine: engine.clone(),
                hotkeys: Mutex::new(bindings),
            });

            use tauri::menu::{Menu, MenuItem};

            let show_i = MenuItem::with_id(app, "show", "Show Game Reader", true, None::<&str>)?;
            let select_i = MenuItem::with_id(app, "select", "Select Region", true, None::<&str>)?;
            let read_i = MenuItem::with_id(app, "read", "Read Region", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_i, &select_i, &read_i, &quit_i])?;

            let tray_engine = engine.clone();
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .tooltip("Game Reader")
                .on_menu_event(move |app, event| {
                    let eng = tray_engine.clone();
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "select" => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = hotkeys::open_region_selector(app).await;
                            });
                        }
                        "read" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = eng.call("read_region", None).await;
                            });
                        }
                        "quit" => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                eng.shutdown().await;
                                app.exit(0);
                            });
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            let init_engine = engine.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match init_engine.call("init", None).await {
                    Ok(_) => {
                        let _ = handle.emit("engine-event", "initialized");
                    }
                    Err(e) => {
                        let _ = handle.emit("engine-event", format!("init-error:{e}"));
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            init_engine,
            get_engine_status,
            read_region,
            stop_speech,
            cycle_voice,
            set_active_voice,
            get_gpu_info,
            get_settings,
            save_settings,
            format_hotkey,
            complete_region_selection,
            open_region_selector,
            get_voice_catalog,
            download_voice,
            delete_voice,
            download_kokoro,
            get_storage_usage,
            hotkeys::get_dpi_scale,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
