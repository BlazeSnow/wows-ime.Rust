//! 输入法扫描命令。

use crate::error::AppResult;
use crate::models::InputMethodScanResult;
use crate::scanner;

/// 扫描系统已启用的中文/日文输入法。
///
/// 扫描失败时仍返回结果对象，其中的 `warningCode` 为稳定的 `Tsf/*` 警告码，
/// 由前端映射为本地化文案。
#[tauri::command]
pub fn scan_input_methods() -> AppResult<InputMethodScanResult> {
    Ok(scanner::scan())
}
