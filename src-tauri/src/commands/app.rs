//! 应用级命令：版本号、数据目录、语言偏好与简单设置读写。

use tauri::State;

use crate::error::{internal_error, AppResult};
use crate::state::AppState;
use crate::storage;

/// 界面语言模式键（对应原 WinUI 3 版本的 `Settings.Language`）。
pub const SETTING_LANGUAGE: &str = "Settings.Language";
/// 当前选中的游戏根目录键（对应原版的 `Game.SelectedPath`）。
pub const SETTING_SELECTED_GAME_PATH: &str = "Game.SelectedPath";
/// 语言模式默认值：跟随系统。
pub const LANGUAGE_AUTO: &str = "auto";

/// 应用版本号（取自 Cargo.toml 的 package.version）。
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 应用数据目录路径。
#[tauri::command]
pub fn get_data_dir(state: State<'_, AppState>) -> String {
    state.data_dir.to_string_lossy().to_string()
}

/// 界面语言模式（`auto` / `zh-Hans` / `zh-Hant` / `ja`）。
#[tauri::command]
pub fn get_language_preference(state: State<'_, AppState>) -> AppResult<String> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    Ok(storage::get_setting(&conn, SETTING_LANGUAGE, LANGUAGE_AUTO))
}

/// 写入简单设置项（键值表）。
#[tauri::command]
pub fn set_setting(key: String, value: String, state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    storage::set_setting(&conn, &key, &value).map_err(|_| internal_error())
}

/// 读取简单设置项；不存在时返回空串。
#[tauri::command]
pub fn get_setting(key: String, state: State<'_, AppState>) -> AppResult<String> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    Ok(storage::get_setting(&conn, &key, ""))
}
