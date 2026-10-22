//! 前后端共享的数据模型。
//!
//! 字段命名与原 WinUI 3 版本保持一致（`camelCase` 序列化），
//! 分类枚举的字符串值沿用原版 `ImeCategory` 名称，便于旧数据迁移。

use serde::{Deserialize, Serialize};

/// 输入法分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImeCategory {
    ChineseSimplified,
    ChineseTraditional,
    Japanese,
}

impl ImeCategory {
    /// 持久化与旧数据迁移使用的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChineseSimplified => "ChineseSimplified",
            Self::ChineseTraditional => "ChineseTraditional",
            Self::Japanese => "Japanese",
        }
    }

    /// 解析持久化字符串；未知值回退简体（与原版一致）。
    pub fn from_stored(value: &str) -> Self {
        match value {
            "ChineseTraditional" => Self::ChineseTraditional,
            "Japanese" => Self::Japanese,
            _ => Self::ChineseSimplified,
        }
    }
}

/// 扫描到的输入法候选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedImeCandidate {
    pub display_name: String,
    pub category: ImeCategory,
    pub confidence: i32,
}

/// 扫描结果；`warning_code` 非空表示扫描未完全成功。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMethodScanResult {
    pub candidates: Vec<ScannedImeCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_arguments: Option<Vec<String>>,
}

/// 写入配置文件所需的输入法定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMethodDefinition {
    pub display_name: String,
    pub category: ImeCategory,
}

/// 自定义游戏路径。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedGamePath {
    pub display_name: String,
    pub path: String,
}

/// 自定义输入法。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedInputMethod {
    pub display_name: String,
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_round_trips_through_stored_string() {
        for category in [
            ImeCategory::ChineseSimplified,
            ImeCategory::ChineseTraditional,
            ImeCategory::Japanese,
        ] {
            assert_eq!(ImeCategory::from_stored(category.as_str()), category);
        }
    }

    #[test]
    fn unknown_stored_category_falls_back_to_simplified() {
        assert_eq!(
            ImeCategory::from_stored("SomethingElse"),
            ImeCategory::ChineseSimplified
        );
    }

    #[test]
    fn models_serialize_with_camel_case_keys() {
        let definition = InputMethodDefinition {
            display_name: "微软拼音".to_string(),
            category: ImeCategory::ChineseSimplified,
        };
        let json = serde_json::to_string(&definition).unwrap();
        assert!(json.contains("\"displayName\""), "{json}");
    }

    #[test]
    fn category_serializes_with_original_names() {
        assert_eq!(
            serde_json::to_string(&ImeCategory::ChineseSimplified).unwrap(),
            "\"ChineseSimplified\""
        );
        assert_eq!(
            serde_json::to_string(&ImeCategory::ChineseTraditional).unwrap(),
            "\"ChineseTraditional\""
        );
        assert_eq!(
            serde_json::to_string(&ImeCategory::Japanese).unwrap(),
            "\"Japanese\""
        );
    }

    #[test]
    fn scan_result_round_trips_through_json() {
        let result = InputMethodScanResult {
            candidates: vec![ScannedImeCandidate {
                display_name: "微软拼音".to_string(),
                category: ImeCategory::ChineseSimplified,
                confidence: 10,
            }],
            warning_code: Some("Tsf/GenericException".to_string()),
            warning_arguments: Some(vec!["boom".to_string()]),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"warningCode\""), "{json}");
        assert!(json.contains("\"warningArguments\""), "{json}");
        assert!(json.contains("\"displayName\""), "{json}");

        let parsed: InputMethodScanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.candidates.len(), 1);
        assert_eq!(parsed.candidates[0].category, ImeCategory::ChineseSimplified);
        assert_eq!(parsed.warning_code.as_deref(), Some("Tsf/GenericException"));
    }

    #[test]
    fn persisted_models_round_trip_through_json() {
        let path = PersistedGamePath {
            display_name: "Steam".to_string(),
            path: "C:\\Steam".to_string(),
        };
        let json = serde_json::to_string(&path).unwrap();
        assert!(json.contains("\"displayName\""), "{json}");
        let parsed: PersistedGamePath = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, "C:\\Steam");

        let method = PersistedInputMethod {
            display_name: "自定义".to_string(),
            category: "Japanese".to_string(),
        };
        let json = serde_json::to_string(&method).unwrap();
        let parsed: PersistedInputMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.category, "Japanese");
    }

    #[test]
    fn scan_result_without_warning_omits_the_fields() {
        let result = InputMethodScanResult {
            candidates: Vec::new(),
            warning_code: None,
            warning_arguments: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("warningCode"), "{json}");
    }
}
