//! 语言模式规则（对应原 WinUI 3 版的 `Rules/LanguageRules.cs`）。
//!
//! 语言模式只有四种取值：`auto`（跟随系统）、`zh-Hans`、`zh-Hant`、`ja`。
//! 存库前必须归一化，避免非法值被持久化后无法再被识别。

/// 跟随系统。
pub const AUTOMATIC: &str = "auto";
/// 简体中文。
pub const CHINESE_SIMPLIFIED: &str = "zh-Hans";
/// 繁体中文。
pub const CHINESE_TRADITIONAL: &str = "zh-Hant";
/// 日语。
pub const JAPANESE: &str = "ja";

/// 是否为受支持的模式（含 `auto`）。
pub fn is_supported_mode(mode: &str) -> bool {
    matches!(
        mode,
        AUTOMATIC | CHINESE_SIMPLIFIED | CHINESE_TRADITIONAL | JAPANESE
    )
}

/// 归一化语言模式：显式的三种语言原样保留，其余（含 `auto` 与非法值）归为 `auto`。
pub fn normalize_mode(mode: &str) -> &'static str {
    match mode {
        CHINESE_SIMPLIFIED => CHINESE_SIMPLIFIED,
        CHINESE_TRADITIONAL => CHINESE_TRADITIONAL,
        JAPANESE => JAPANESE,
        _ => AUTOMATIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_modes_are_kept_as_is() {
        assert_eq!(normalize_mode(CHINESE_SIMPLIFIED), CHINESE_SIMPLIFIED);
        assert_eq!(normalize_mode(CHINESE_TRADITIONAL), CHINESE_TRADITIONAL);
        assert_eq!(normalize_mode(JAPANESE), JAPANESE);
    }

    #[test]
    fn automatic_and_unknown_values_fall_back_to_auto() {
        assert_eq!(normalize_mode(AUTOMATIC), AUTOMATIC);
        assert_eq!(normalize_mode(""), AUTOMATIC);
        assert_eq!(normalize_mode("en-US"), AUTOMATIC);
        assert_eq!(normalize_mode("zh-Hans-CN"), AUTOMATIC);
        assert_eq!(normalize_mode("ja-JP"), AUTOMATIC);
    }

    #[test]
    fn supported_modes_cover_the_four_known_values() {
        for mode in [AUTOMATIC, CHINESE_SIMPLIFIED, CHINESE_TRADITIONAL, JAPANESE] {
            assert!(is_supported_mode(mode), "{mode} 应受支持");
        }
        assert!(!is_supported_mode("en-US"));
        assert!(!is_supported_mode(""));
    }
}
