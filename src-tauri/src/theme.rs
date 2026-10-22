//! 系统主题（深色 / 浅色）适配。
//!
//! 主题变化由 Tauri 的 `WindowEvent::ThemeChanged` 感知：它源自系统广播
//! `WM_SETTINGCHANGE`，是**推送式通知**，应用不需要轮询。
//!
//! 需要适配的是原生标题栏：WebView2 应用的标题栏按钮跟随「应用配色」而非系统配色，
//! 所以要显式设置 `DWMWA_USE_IMMERSIVE_DARK_MODE`；
//! 页面内容由 CSS 的 `prefers-color-scheme` 自动跟随，无需干预。

use tauri::{AppHandle, Manager, Runtime};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};

/// 把深浅主题应用到原生窗口（标题栏按钮配色 + 窗口底色）。
pub fn apply_theme<R: Runtime>(app: &AppHandle<R>, dark: bool) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    // 窗口底色：避免深色下加载瞬间的白底闪烁
    let (r, g, b) = if dark { (0x0a, 0x0a, 0x0a) } else { (0xff, 0xff, 0xff) };
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 0xff)));

    // 原生标题栏按钮配色
    if let Ok(hwnd) = window.hwnd() {
        apply_dark_title_bar(hwnd, dark);
    }
}

/// 设置原生标题栏的深色模式。
fn apply_dark_title_bar(hwnd: HWND, dark: bool) {
    let value: i32 = if dark { 1 } else { 0 };
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&value as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }
}
