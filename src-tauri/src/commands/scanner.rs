//! 输入法扫描命令。

use crate::error::AppResult;
use crate::models::InputMethodScanResult;
use crate::scanner;

/// 扫描系统已启用的中文/日文输入法。
///
/// 扫描失败时仍返回结果对象，其中的 `warningCode` 为稳定的 `Tsf/*` 警告码，
/// 由前端映射为本地化文案。
///
/// 扫描过程中的未预期异常（panic）会被捕获并转为 `Tsf/GenericException` 警告码，
/// 不会中断界面——与原 WinUI 3 版「异常不阻断 UI」的行为一致。
#[tauri::command]
pub fn scan_input_methods() -> AppResult<InputMethodScanResult> {
    match std::panic::catch_unwind(scanner::scan) {
        Ok(result) => Ok(result),
        Err(_) => Ok(scanner::generic_failure()),
    }
}
