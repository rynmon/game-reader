use std::collections::HashMap;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    SelectRegion,
    ReadRegion,
    StopSpeech,
    CycleVoice,
    Quit,
}

pub struct HotkeyBindings {
    pub by_shortcut: HashMap<Shortcut, HotkeyAction>,
}

impl HotkeyBindings {
    pub fn empty() -> Self {
        Self {
            by_shortcut: HashMap::new(),
        }
    }
}

pub fn parse_hotkey(raw: &str) -> Option<Shortcut> {
    let lowered = raw.to_lowercase();
    let parts: Vec<&str> = lowered.split('+').collect();
    let mut mods = Modifiers::empty();
    let mut code = None;

    for part in parts.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        match part.as_ref() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" | "option" => mods |= Modifiers::ALT,
            "cmd" | "command" | "meta" | "super" | "win" => mods |= Modifiers::SUPER,
            key => code = code_from_key(key),
        }
    }

    code.map(|c| Shortcut::new(Some(mods), c))
}

fn code_from_key(key: &str) -> Option<Code> {
    Some(match key {
        "a" => Code::KeyA,
        "b" => Code::KeyB,
        "c" => Code::KeyC,
        "d" => Code::KeyD,
        "e" => Code::KeyE,
        "f" => Code::KeyF,
        "g" => Code::KeyG,
        "h" => Code::KeyH,
        "i" => Code::KeyI,
        "j" => Code::KeyJ,
        "k" => Code::KeyK,
        "l" => Code::KeyL,
        "m" => Code::KeyM,
        "n" => Code::KeyN,
        "o" => Code::KeyO,
        "p" => Code::KeyP,
        "q" => Code::KeyQ,
        "r" => Code::KeyR,
        "s" => Code::KeyS,
        "t" => Code::KeyT,
        "u" => Code::KeyU,
        "v" => Code::KeyV,
        "w" => Code::KeyW,
        "x" => Code::KeyX,
        "y" => Code::KeyY,
        "z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "space" => Code::Space,
        "escape" | "esc" => Code::Escape,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        _ => return None,
    })
}

pub fn format_hotkey(raw: &str) -> String {
    raw.split('+')
        .filter(|p| !p.is_empty())
        .map(|part| match part.to_lowercase().as_str() {
            "ctrl" | "control" => "Ctrl".to_string(),
            "shift" => "Shift".to_string(),
            "alt" | "option" => "Alt".to_string(),
            "cmd" | "command" | "meta" | "super" | "win" => "Cmd".to_string(),
            key if key.len() == 1 => key.to_uppercase(),
            key => key.to_uppercase(),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

pub fn register_hotkeys(app: &AppHandle, config: &AppConfig) -> Result<HotkeyBindings, String> {
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;

    let pairs = [
        (&config.hotkey_select, HotkeyAction::SelectRegion),
        (&config.hotkey_read, HotkeyAction::ReadRegion),
        (&config.hotkey_stop, HotkeyAction::StopSpeech),
        (&config.hotkey_cycle, HotkeyAction::CycleVoice),
        (&config.hotkey_quit, HotkeyAction::Quit),
    ];

    let mut by_shortcut = HashMap::new();
    for (raw, action) in pairs {
        let shortcut = parse_hotkey(raw).ok_or_else(|| format!("Invalid shortcut: {raw}"))?;
        if by_shortcut.contains_key(&shortcut) {
            return Err(format!("Duplicate shortcut binding: {raw}"));
        }
        gs.register(shortcut).map_err(|e| e.to_string())?;
        by_shortcut.insert(shortcut, action);
    }

    Ok(HotkeyBindings { by_shortcut })
}

pub fn handle_hotkey(app: &AppHandle, shortcut: &Shortcut, bindings: &HotkeyBindings) {
    let Some(action) = bindings.by_shortcut.get(shortcut) else {
        return;
    };

    let state = app.state::<crate::AppState>();
    let eng = state.engine.clone();
    let app_handle = app.clone();

    match action {
        HotkeyAction::SelectRegion => {
            tauri::async_runtime::spawn(async move {
                let _ = open_region_selector(app_handle).await;
            });
        }
        HotkeyAction::ReadRegion => {
            tauri::async_runtime::spawn(async move {
                let _ = eng.call("read_region", None).await;
            });
        }
        HotkeyAction::StopSpeech => {
            tauri::async_runtime::spawn(async move {
                let _ = eng.call("stop", None).await;
            });
        }
        HotkeyAction::CycleVoice => {
            tauri::async_runtime::spawn(async move {
                let _ = eng.call("cycle_voice", None).await;
            });
        }
        HotkeyAction::Quit => {
            tauri::async_runtime::spawn(async move {
                eng.shutdown().await;
                app_handle.exit(0);
            });
        }
    }
}

pub async fn open_region_selector(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("index.html".into()))
        .title("Select Region")
        .fullscreen(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(windows)]
#[tauri::command]
pub fn get_dpi_scale() -> f64 {
    use windows::Win32::Graphics::Gdi::{GetDeviceCaps, GetDC, LOGPIXELSX, ReleaseDC};
    use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

    unsafe {
        let hwnd = GetDesktopWindow();
        let hdc = GetDC(hwnd);
        if hdc.0.is_null() {
            return 1.0;
        }
        let dpi = GetDeviceCaps(hdc, LOGPIXELSX);
        let _ = ReleaseDC(hwnd, hdc);
        if dpi > 0 {
            dpi as f64 / 96.0
        } else {
            1.0
        }
    }
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_dpi_scale() -> f64 {
    1.0
}

#[allow(dead_code)]
pub fn shortcut_state_pressed(event: ShortcutState) -> bool {
    event == ShortcutState::Pressed
}
