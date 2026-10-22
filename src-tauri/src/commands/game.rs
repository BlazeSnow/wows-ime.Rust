//! 游戏目录校验与配置文件写入命令。

use crate::error::{AppError, AppResult};
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
    write_config_files_inner(&game_root, &input_methods)
}

/// 写入配置文件的实际逻辑（与命令外壳解耦，便于单元测试）。
///
/// 校验顺序与原 WinUI 3 版一致：先确认至少选中一个输入法，再确认游戏目录下
/// 存在可写入的数字版本目录，最后才落盘。
pub fn write_config_files_inner(
    game_root: &str,
    input_methods: &[InputMethodDefinition],
) -> AppResult<usize> {
    if input_methods.is_empty() {
        return Err(AppError::new("ErrorNoInputMethodSelected"));
    }
    let targets = gameconfig::resolve_target_config_files(game_root);
    if targets.is_empty() {
        return Err(AppError::new("ErrorBinVersionDirectoryNotFound"));
    }
    gameconfig::write_config_files(&targets, input_methods).map_err(|failure| {
        // detail 带上出错的具体路径，便于用户判断是哪个版本目录写入失败
        AppError::with_detail("ErrorWriteConfigFailed", failure.to_string())
    })?;
    Ok(targets.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ImeCategory;
    use std::fs;
    use std::path::PathBuf;

    fn definition(name: &str) -> InputMethodDefinition {
        InputMethodDefinition {
            display_name: name.to_string(),
            category: ImeCategory::ChineseSimplified,
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        let temp = std::env::temp_dir().join(format!("wows-ime-command-{tag}"));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        temp
    }

    #[test]
    fn rejects_when_no_input_method_is_selected() {
        let root = temp_root("empty-ime");
        let error = write_config_files_inner(root.to_str().unwrap(), &[]).unwrap_err();
        assert_eq!(error.code, "ErrorNoInputMethodSelected");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_when_no_numeric_version_dir_exists() {
        let root = temp_root("no-version");
        fs::create_dir_all(root.join("bin").join("patch")).unwrap();
        let error =
            write_config_files_inner(root.to_str().unwrap(), &[definition("微软拼音")]).unwrap_err();
        assert_eq!(error.code, "ErrorBinVersionDirectoryNotFound");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn writes_every_target_and_returns_the_count() {
        let root = temp_root("write-ok");
        fs::create_dir_all(root.join("bin").join("1")).unwrap();
        fs::create_dir_all(root.join("bin").join("2")).unwrap();

        let written =
            write_config_files_inner(root.to_str().unwrap(), &[definition("微软拼音")]).unwrap();
        assert_eq!(written, 2);

        for version in ["1", "2"] {
            let target = root
                .join("bin")
                .join(version)
                .join("res_mods")
                .join("ime_config.xml");
            let text = fs::read_to_string(&target).unwrap();
            assert!(text.contains("微软拼音"), "{text}");
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn write_failure_carries_the_offending_path_in_detail() {
        let root = temp_root("write-fail");
        // 把 res_mods 建成文件，让创建目录失败
        let blocker = root.join("bin").join("1").join("res_mods");
        fs::create_dir_all(blocker.parent().unwrap()).unwrap();
        fs::write(&blocker, b"not a directory").unwrap();

        let error =
            write_config_files_inner(root.to_str().unwrap(), &[definition("微软拼音")]).unwrap_err();
        assert_eq!(error.code, "ErrorWriteConfigFailed");
        let detail = error.detail.unwrap_or_default();
        assert!(detail.contains("res_mods"), "{detail}");

        let _ = fs::remove_dir_all(&root);
    }
}
