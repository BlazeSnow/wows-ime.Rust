//! 游戏目录校验与配置文件写入命令。

use crate::error::{internal_error, AppResult};
use crate::gameconfig;
use crate::models::InputMethodDefinition;

/// 游戏根目录是否包含游戏可执行文件。
#[tauri::command]
pub fn has_game_executable(game_root: String) -> bool {
    gameconfig::has_game_executable(&game_root)
}

/// 解析需要写入的配置文件路径列表（`bin` 下所有数字版本目录）。
#[tauri::command]
pub fn resolve_target_config_files(game_root: String) -> Vec<String> {
    gameconfig::resolve_target_config_files(&game_root)
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

/// 目标路径中已存在配置文件的数量（用于「是否覆盖」提示）。
#[tauri::command]
pub fn existing_config_count(game_root: String) -> usize {
    gameconfig::count_existing_files(&gameconfig::resolve_target_config_files(&game_root))
}

/// 写入输入法配置文件；返回实际写入的文件数量。
#[tauri::command]
pub fn write_config_files(
    game_root: String,
    input_methods: Vec<InputMethodDefinition>,
) -> AppResult<usize> {
    if input_methods.is_empty() {
        return Err(crate::error::AppError::new("ErrorNoInputMethodSelected"));
    }
    let targets = gameconfig::resolve_target_config_files(&game_root);
    if targets.is_empty() {
        return Err(crate::error::AppError::new("ErrorBinVersionDirectoryNotFound"));
    }
    gameconfig::write_config_files(&targets, &input_methods).map_err(|_| internal_error())?;
    Ok(targets.len())
}
