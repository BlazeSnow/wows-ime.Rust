mod commands;
mod error;
mod state;
mod storage;
mod window;

use state::AppState;
use tauri::Manager;
use tauri_plugin_window_state::StateFlags;

/// 参与保存/恢复的窗口状态：几何 + 最大化 + 全屏；不含 VISIBLE（显隐由启动流程控制）。
const WINDOW_STATE_FLAGS: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED)
    .union(StateFlags::FULLSCREEN);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 第二实例：唤醒已有主窗口。set_focus 对最小化的窗口不会还原，
            // 需先还原再聚焦，否则用户感知为「二次启动没反应」。
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        // 保存并恢复用户调整的窗口几何；窗口默认隐藏（tauri.conf visible=false），
        // 由 setup 完成恢复与钳制后再显示，避免按默认几何闪窗。
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db = storage::open(&data_dir.join("settings.db")).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("打开设置库失败：{e}"))
            })?;

            app.manage(AppState {
                data_dir,
                db: std::sync::Mutex::new(db),
            });

            // 深色模式：按系统主题预设 WebView 底色，避免启动时白底闪烁
            //（页面加载后配色由 CSS 的 prefers-color-scheme 接管）。
            if let Some(window) = app.get_webview_window("main") {
                let dark = window.theme().is_ok_and(|t| t == tauri::Theme::Dark);
                let (r, g, b) = if dark { (0x0a, 0x0a, 0x0a) } else { (0xff, 0xff, 0xff) };
                let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 0xff)));
            }

            // 几何已由插件恢复（无状态则用默认值）：钳制到工作区后显示，
            // 期间窗口一直不可见，用户看到的即最终位置与尺寸。
            window::fit_main_window(app.handle());
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关闭前把用户调整的几何写盘（插件自动保存依赖应用退出事件，
            // 显式保存保证任何退出路径都有最新值）。
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    use tauri_plugin_window_state::AppHandleExt as _;
                    let _ = window.app_handle().save_window_state(WINDOW_STATE_FLAGS);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_version,
            commands::app::get_data_dir,
            commands::app::get_language_preference,
            commands::app::get_setting,
            commands::app::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
