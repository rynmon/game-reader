use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod downloads;
mod engine;
mod hotkeys;
mod paths;

use engine::{EngineClient, SharedEngine};

pub struct AppState {
    pub engine: SharedEngine,
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
async fn get_settings(state: State<'_, AppState>) -> Result<Value, String> {
    state.engine.call("load_config", None).await
}

#[tauri::command]
async fn save_settings(settings: Value, state: State<'_, AppState>) -> Result<Value, String> {
    state.engine.call("save_config", Some(settings)).await
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

fn register_shortcuts(app: &AppHandle) -> Result<(), String> {
    let gs = app.global_shortcut();
    let shortcuts = [
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyR),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyQ),
    ];
    for s in shortcuts {
        gs.register(s).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut) {
    let state = app.state::<AppState>();
    let eng = state.engine.clone();
    let app_handle = app.clone();

    let mods = Modifiers::CONTROL | Modifiers::SHIFT;

    if shortcut.mods == mods && shortcut.key == Code::KeyR {
        tauri::async_runtime::spawn(async move {
            let _ = hotkeys::open_region_selector(app_handle).await;
        });
    } else if shortcut.mods == mods && shortcut.key == Code::KeyT {
        tauri::async_runtime::spawn(async move {
            let _ = eng.call("read_region", None).await;
        });
    } else if shortcut.mods == mods && shortcut.key == Code::KeyS {
        tauri::async_runtime::spawn(async move {
            let _ = eng.call("stop", None).await;
        });
    } else if shortcut.mods == mods && shortcut.key == Code::KeyV {
        tauri::async_runtime::spawn(async move {
            let _ = eng.call("cycle_voice", None).await;
        });
    } else if shortcut.mods == mods && shortcut.key == Code::KeyQ {
        tauri::async_runtime::spawn(async move {
            eng.shutdown().await;
            app_handle.exit(0);
        });
    }
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
                    handle_shortcut(app, shortcut);
                })
                .build(),
        )
        .setup(|app| {
            paths::ensure_dirs();

            let engine = Arc::new(EngineClient::new(app.handle().clone()));
            app.manage(AppState {
                engine: engine.clone(),
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
                            tauri::async_runtime::spawn(async move {
                                eng.shutdown().await;
                                app.exit(0);
                            });
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            if let Err(e) = register_shortcuts(app.handle()) {
                eprintln!("Failed to register hotkeys: {e}");
            }

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
