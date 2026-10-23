//! 自定义游戏路径与自定义输入法的读写命令。

use tauri::State;

use crate::error::{internal_error, AppResult};
use crate::models::{PersistedGamePath, PersistedInputMethod};
use crate::state::AppState;
use crate::storage;

/// 读取全部自定义游戏路径。
#[tauri::command]
pub fn list_custom_game_paths(state: State<'_, AppState>) -> AppResult<Vec<PersistedGamePath>> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    Ok(storage::load_custom_game_paths(&conn))
}

/// 全量同步自定义游戏路径（前端提交完整列表）。
#[tauri::command]
pub fn save_custom_game_paths(
    paths: Vec<PersistedGamePath>,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    storage::save_custom_game_paths(&conn, &paths).map_err(|_| internal_error())
}

/// 读取全部自定义输入法。
#[tauri::command]
pub fn list_custom_input_methods(
    state: State<'_, AppState>,
) -> AppResult<Vec<PersistedInputMethod>> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    Ok(storage::load_custom_input_methods(&conn))
}

/// 全量同步自定义输入法（前端提交完整列表）。
#[tauri::command]
pub fn save_custom_input_methods(
    input_methods: Vec<PersistedInputMethod>,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    storage::save_custom_input_methods(&conn, &input_methods).map_err(|_| internal_error())
}

/// 当前选中的游戏根目录；未选择时返回空串。
#[tauri::command]
pub fn get_selected_game_path(state: State<'_, AppState>) -> AppResult<String> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    Ok(storage::get_setting(
        &conn,
        storage::SETTING_SELECTED_GAME_PATH,
        "",
    ))
}

/// 保存当前选中的游戏根目录。
#[tauri::command]
pub fn set_selected_game_path(path: String, state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| internal_error())?;
    storage::set_setting(&conn, storage::SETTING_SELECTED_GAME_PATH, &path)
        .map_err(|_| internal_error())
}
