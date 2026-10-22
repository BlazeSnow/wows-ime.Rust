//! 输入法扫描：通过 TSF/COM 的 `ITfInputProcessorProfiles` 枚举系统已启用的中文/日文输入法。
//!
//! 行为与原 WinUI 3 版本一致：
//! - 只处理中文（`0x0804` / `0x0404` / `0x0C04` / `0x1004` / `0x1404`）与日文（`0x0411`）语言 ID；
//! - 分类优先按语言 ID 推断，其次按名称关键词（速成/倉頡/注音/Quick/Cangjie → 繁体，
//!   拼音/五笔 → 简体，Japanese/日文 → 日文），都判断不出时归为简体；
//! - 过滤「输入体验 / Input Experience」噪音项；
//! - 失败时返回稳定的警告码（`Tsf/*`），前端据此映射为本地化文案。
//!
//! 仅 Windows 可用：依赖 COM 与 TSF。

use std::collections::BTreeMap;

use windows::core::GUID;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::TextServices::{ITfInputProcessorProfiles, TF_LANGUAGEPROFILE};

use crate::models::{ImeCategory, InputMethodScanResult, ScannedImeCandidate};

pub const CO_INITIALIZE_FAILED_WARNING: &str = "Tsf/CoInitializeFailed";
pub const CREATE_PROFILES_OBJECT_FAILED_WARNING: &str = "Tsf/CreateProfilesObjectFailed";
pub const GET_LANGUAGE_LIST_FAILED_WARNING: &str = "Tsf/GetLanguageListFailed";
pub const GET_LANGUAGE_LIST_EMPTY_WARNING: &str = "Tsf/GetLanguageListEmpty";
pub const COM_EXCEPTION_WARNING: &str = "Tsf/ComException";
pub const GENERIC_EXCEPTION_WARNING: &str = "Tsf/GenericException";

/// `RPC_E_CHANGED_MODE`：COM 已按其它套间模型初始化，可继续使用，但不得反初始化。
const RPC_E_CHANGED_MODE: i32 = -2147417850; // 0x80010106

const CLSID_INPUT_PROCESSOR_PROFILES: GUID = GUID {
    data1: 0x33C53A50,
    data2: 0xF456,
    data3: 0x4884,
    data4: [0xB0, 0x49, 0x85, 0xFD, 0x64, 0x3E, 0xCF, 0xED],
};

/// 扫描系统输入法。
pub fn scan() -> InputMethodScanResult {
    let mut candidates: BTreeMap<String, ScannedImeCandidate> = BTreeMap::new();
    let result = read_candidates_from_tsf();
    for candidate in result.candidates {
        upsert_candidate(&mut candidates, candidate);
    }

    let mut ordered: Vec<ScannedImeCandidate> = candidates.into_values().collect();
    ordered.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });

    InputMethodScanResult {
        candidates: ordered,
        warning_code: result.warning_code,
        warning_arguments: result.warning_arguments,
    }
}

fn read_candidates_from_tsf() -> InputMethodScanResult {
    let mut candidates: Vec<ScannedImeCandidate> = Vec::new();

    // COM 初始化：S_OK / S_FALSE 表示本次调用负责反初始化；
    // RPC_E_CHANGED_MODE 表示已按其它模型初始化，可继续但不反初始化。
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let should_uninitialize = hr.is_ok() || hr == windows::core::HRESULT(1);
    if hr.is_err() && hr.0 != RPC_E_CHANGED_MODE {
        return warning(
            candidates,
            CO_INITIALIZE_FAILED_WARNING,
            vec![format_hresult(hr.0)],
        );
    }

    let result = collect_candidates(&mut candidates);

    if should_uninitialize {
        unsafe { CoUninitialize() };
    }
    result
}

fn collect_candidates(candidates: &mut Vec<ScannedImeCandidate>) -> InputMethodScanResult {
    let profiles = match unsafe {
        CoCreateInstance::<Option<&windows::core::IUnknown>, ITfInputProcessorProfiles>(
            &CLSID_INPUT_PROCESSOR_PROFILES,
            None,
            CLSCTX_INPROC_SERVER,
        )
    } {
        Ok(profiles) => profiles,
        Err(error) => {
            return warning(
                std::mem::take(candidates),
                CREATE_PROFILES_OBJECT_FAILED_WARNING,
                vec![format_hresult(error.code().0)],
            );
        }
    };

    let mut language_ids: *mut u16 = std::ptr::null_mut();
    let mut language_count: u32 = 0;
    if let Err(error) = unsafe { profiles.GetLanguageList(&mut language_ids, &mut language_count) } {
        return warning(
            std::mem::take(candidates),
            GET_LANGUAGE_LIST_FAILED_WARNING,
            vec![format_hresult(error.code().0)],
        );
    }
    if language_ids.is_null() {
        return warning(
            std::mem::take(candidates),
            GET_LANGUAGE_LIST_EMPTY_WARNING,
            Vec::new(),
        );
    }

    // GetLanguageList 返回的数组由 COM 分配，必须用 CoTaskMemFree 释放
    // （原 WinUI 3 版在 finally 里释放）。
    let language_ids_ptr = language_ids;
    let com_error = {
        let ids = unsafe { std::slice::from_raw_parts(language_ids_ptr, language_count as usize) };
        collect_candidates_for_languages(&profiles, ids, candidates)
    };
    unsafe { CoTaskMemFree(Some(language_ids_ptr.cast())) };

    if let Some(hresult) = com_error {
        return warning(
            std::mem::take(candidates),
            COM_EXCEPTION_WARNING,
            vec![format_hresult(hresult)],
        );
    }

    InputMethodScanResult {
        candidates: std::mem::take(candidates),
        warning_code: None,
        warning_arguments: None,
    }
}

/// 遍历语言 ID 收集输入法；COM 调用失败时返回 HRESULT。
fn collect_candidates_for_languages(
    profiles: &ITfInputProcessorProfiles,
    language_ids: &[u16],
    candidates: &mut Vec<ScannedImeCandidate>,
) -> Option<i32> {
    let mut com_error: Option<i32> = None;
    for &language_id in language_ids {
        if !is_target_language_profile(language_id) {
            continue;
        }
        if let Some(hresult) = collect_language_profiles(profiles, language_id, candidates) {
            com_error.get_or_insert(hresult);
        }
    }
    com_error
}

/// 枚举某个语言下的输入法配置文件；COM 调用失败时返回 HRESULT。
fn collect_language_profiles(
    profiles: &ITfInputProcessorProfiles,
    language_id: u16,
    candidates: &mut Vec<ScannedImeCandidate>,
) -> Option<i32> {
    let enumerator = match unsafe { profiles.EnumLanguageProfiles(language_id) } {
        Ok(enumerator) => enumerator,
        Err(error) => return Some(error.code().0),
    };

    loop {
        let mut items = [TF_LANGUAGEPROFILE::default()];
        let mut fetched: u32 = 0;
        if unsafe { enumerator.Next(&mut items, &mut fetched) }.is_err() || fetched == 0 {
            break;
        }

        let item = items[0];
        let enabled = unsafe {
            profiles
                .IsEnabledLanguageProfile(&item.clsid, item.langid, &item.guidProfile)
                .map(|value| value.as_bool())
                .unwrap_or(false)
        };
        if !enabled {
            continue;
        }

        let Some(name) = profile_description(profiles, &item) else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() || is_noise_ime_name(name) {
            continue;
        }

        let category = infer_category_from_lang_id(item.langid)
            .or_else(|| infer_category_from_name(name))
            .unwrap_or(ImeCategory::ChineseSimplified);
        candidates.push(ScannedImeCandidate {
            display_name: name.to_string(),
            category,
            confidence: 10,
        });
    }

    None
}

fn profile_description(
    profiles: &ITfInputProcessorProfiles,
    item: &TF_LANGUAGEPROFILE,
) -> Option<String> {
    let description =
        unsafe { profiles.GetLanguageProfileDescription(&item.clsid, item.langid, &item.guidProfile) }
            .ok()?;
    Some(description.to_string())
}

fn upsert_candidate(
    candidates: &mut BTreeMap<String, ScannedImeCandidate>,
    candidate: ScannedImeCandidate,
) {
    let key = candidate.display_name.to_lowercase();
    match candidates.get(&key) {
        Some(existing) if existing.confidence >= candidate.confidence => {}
        _ => {
            candidates.insert(key, candidate);
        }
    }
}

fn is_target_language_profile(language_id: u16) -> bool {
    matches!(
        language_id,
        0x0804 | 0x0404 | 0x0C04 | 0x1004 | 0x1404 | 0x0411
    )
}

fn is_noise_ime_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("输入体验") || lower.contains("input experience")
}

fn infer_category_from_lang_id(language_id: u16) -> Option<ImeCategory> {
    match language_id {
        0x0804 | 0x1004 => Some(ImeCategory::ChineseSimplified),
        0x0404 | 0x0C04 | 0x1404 => Some(ImeCategory::ChineseTraditional),
        0x0411 => Some(ImeCategory::Japanese),
        _ => None,
    }
}

fn infer_category_from_name(name: &str) -> Option<ImeCategory> {
    if name.trim().is_empty() {
        return None;
    }
    let lower = name.to_lowercase();

    if ["速成", "倉頡", "仓颉", "注音", "quick", "cangjie"]
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        return Some(ImeCategory::ChineseTraditional);
    }
    if ["japanese", "日文"]
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        return Some(ImeCategory::Japanese);
    }
    if ["拼音", "五笔"]
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        return Some(ImeCategory::ChineseSimplified);
    }
    None
}

fn warning(
    candidates: Vec<ScannedImeCandidate>,
    code: &str,
    arguments: Vec<String>,
) -> InputMethodScanResult {
    InputMethodScanResult {
        candidates,
        warning_code: Some(code.to_string()),
        warning_arguments: if arguments.is_empty() {
            None
        } else {
            Some(arguments)
        },
    }
}

/// 扫描出现未预期异常（panic）时的结果：返回稳定的警告码，保证界面不中断。
pub fn generic_failure() -> InputMethodScanResult {
    warning(Vec::new(), GENERIC_EXCEPTION_WARNING, Vec::new())
}

fn format_hresult(value: i32) -> String {
    format!("0x{:08X}", value as u32)
}

#[cfg(test)]
mod tests;
