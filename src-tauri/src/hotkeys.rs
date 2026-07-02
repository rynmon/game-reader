use tauri::Manager;

pub async fn open_region_selector(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

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
