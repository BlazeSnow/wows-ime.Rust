//! 游戏配置文件 `ime_config.xml` 的校验、生成与写入。
//!
//! 目标路径为 `<游戏根目录>\bin\<数字版本>\res_mods\ime_config.xml`：
//! `bin` 目录下所有数字版本目录都要写入，版本号（如 `8842736`）不固定，
//! 因此只按「目录名是纯数字」筛选，绝不硬编码版本号。
//!
//! 生成规则与原 WinUI 3 版本一致：`<data><language>` 下依次是
//! `ChineseSimplified`、`Japanese`、`ChineseTraditional` 三个分组，
//! 每个输入法写 `imeName` / `displayName` / `Tag` 三个元素，
//! 文件以 UTF-8 无 BOM 保存。

use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{ImeCategory, InputMethodDefinition};

const WOWS_EXE_NAME: &str = "WorldOfWarships.exe";
const KORABLI_EXE_NAME: &str = "Korabli.exe";
const TAG_SIMPLIFIED: &str = "GFxIME_Ch_Simp";
const TAG_TRADITIONAL: &str = "GFxIME_Ch_Trad_Array";
const TAG_JAPANESE: &str = "GFxIME_Jp";

/// 游戏根目录是否包含游戏可执行文件（国服 `WorldOfWarships.exe` 或俄服 `Korabli.exe`）。
pub fn has_game_executable(game_root: &str) -> bool {
    if game_root.trim().is_empty() {
        return false;
    }
    let root = Path::new(game_root);
    root.join(WOWS_EXE_NAME).is_file() || root.join(KORABLI_EXE_NAME).is_file()
}

/// 解析需要写入的目标配置文件列表：`bin` 下的数字版本目录各一份。
pub fn resolve_target_config_files(game_root: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if game_root.trim().is_empty() {
        return results;
    }

    let bin_path = Path::new(game_root).join("bin");
    let Ok(entries) = fs::read_dir(&bin_path) else {
        return results;
    };

    let mut version_dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();

    // 与原版一致：按目录名的字符串序（忽略大小写）排序。
    version_dirs.sort_by_key(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_lowercase()
    });

    for dir in version_dirs {
        results.push(dir.join("res_mods").join("ime_config.xml"));
    }
    results
}

/// 生成 `ime_config.xml` 文本。
pub fn build_config_document(selected: &[InputMethodDefinition]) -> String {
    let mut simplified: Vec<String> = Vec::new();
    let mut traditional: Vec<String> = Vec::new();
    let mut japanese: Vec<String> = Vec::new();

    for input_method in selected {
        let name = escape_xml(&input_method.display_name);
        let (bucket, tag) = match input_method.category {
            ImeCategory::ChineseSimplified => (&mut simplified, TAG_SIMPLIFIED),
            ImeCategory::ChineseTraditional => (&mut traditional, TAG_TRADITIONAL),
            ImeCategory::Japanese => (&mut japanese, TAG_JAPANESE),
        };
        bucket.push(format!(
            "      <imeName>{name}</imeName>\n      <displayName>{name}</displayName>\n      <Tag>{tag}</Tag>"
        ));
    }

    let mut document = String::from("<data>\n  <language>\n");
    document.push_str(&render_group("ChineseSimplified", &simplified));
    document.push_str(&render_group("Japanese", &japanese));
    document.push_str(&render_group("ChineseTraditional", &traditional));
    document.push_str("  </language>\n</data>\n");
    document
}

/// 写入失败：定位到具体的目标文件，便于前端提示出错的路径。
#[derive(Debug)]
pub struct WriteFailure {
    pub path: PathBuf,
    pub source: std::io::Error,
}

impl std::fmt::Display for WriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.source)
    }
}

/// 把配置文件写入全部目标路径（UTF-8 无 BOM，必要时创建目录）。
pub fn write_config_files(
    target_files: &[PathBuf],
    selected: &[InputMethodDefinition],
) -> Result<(), WriteFailure> {
    let document = build_config_document(selected);
    for target_file in target_files {
        let write = || -> std::io::Result<()> {
            if let Some(directory) = target_file.parent() {
                fs::create_dir_all(directory)?;
            }
            fs::write(target_file, document.as_bytes())
        };
        write().map_err(|source| WriteFailure {
            path: target_file.clone(),
            source,
        })?;
    }
    Ok(())
}

/// 目标文件中已存在的数量（用于「是否覆盖」提示）。
pub fn count_existing_files(target_files: &[PathBuf]) -> usize {
    target_files.iter().filter(|path| path.is_file()).count()
}

fn render_group(name: &str, entries: &[String]) -> String {
    if entries.is_empty() {
        return format!("    <{name} />\n");
    }
    let mut group = format!("    <{name}>\n");
    for entry in entries {
        group.push_str(entry);
        group.push('\n');
    }
    group.push_str(&format!("    </{name}>\n"));
    group
}

/// XML 文本转义，避免输入法名称中的 `&`、`<` 等破坏文档结构。
fn escape_xml(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(name: &str, category: ImeCategory) -> InputMethodDefinition {
        InputMethodDefinition {
            display_name: name.to_string(),
            category,
        }
    }

    #[test]
    fn empty_selection_produces_three_empty_groups() {
        let document = build_config_document(&[]);
        assert!(document.contains("<ChineseSimplified />"), "{document}");
        assert!(document.contains("<Japanese />"), "{document}");
        assert!(document.contains("<ChineseTraditional />"), "{document}");
    }

    #[test]
    fn groups_keep_simplified_japanese_traditional_order() {
        let document = build_config_document(&[definition("微软拼音", ImeCategory::ChineseSimplified)]);
        let simplified = document.find("ChineseSimplified").unwrap();
        let japanese = document.find("Japanese").unwrap();
        let traditional = document.find("ChineseTraditional").unwrap();
        assert!(simplified < japanese && japanese < traditional, "{document}");
    }

    #[test]
    fn each_input_method_writes_name_display_name_and_tag() {
        let document = build_config_document(&[
            definition("微软拼音", ImeCategory::ChineseSimplified),
            definition("微軟速成", ImeCategory::ChineseTraditional),
            definition("Microsoft 日本語", ImeCategory::Japanese),
        ]);
        assert!(document.contains("<imeName>微软拼音</imeName>"), "{document}");
        assert!(document.contains("<displayName>微软拼音</displayName>"), "{document}");
        assert!(document.contains("<Tag>GFxIME_Ch_Simp</Tag>"), "{document}");
        assert!(document.contains("<Tag>GFxIME_Ch_Trad_Array</Tag>"), "{document}");
        assert!(document.contains("<Tag>GFxIME_Jp</Tag>"), "{document}");
    }

    #[test]
    fn names_are_xml_escaped() {
        let document = build_config_document(&[definition("A&B<C>", ImeCategory::ChineseSimplified)]);
        assert!(document.contains("A&amp;B&lt;C&gt;"), "{document}");
        assert!(!document.contains("A&B"), "{document}");
    }

    #[test]
    fn resolve_skips_non_numeric_version_dirs() {
        let temp = std::env::temp_dir().join("wows-ime-gameconfig-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("bin").join("8842736")).unwrap();
        fs::create_dir_all(temp.join("bin").join("12345")).unwrap();
        fs::create_dir_all(temp.join("bin").join("patch")).unwrap();

        let targets = resolve_target_config_files(temp.to_str().unwrap());
        let names: Vec<String> = targets
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["ime_config.xml", "ime_config.xml"]);
        assert!(targets.iter().all(|p| p.ends_with("res_mods/ime_config.xml")));
        // 字符串序：12345 在 8842736 之前
        assert!(targets[0].to_string_lossy().contains("12345"));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn write_creates_directories_and_uses_utf8_without_bom() {
        let temp = std::env::temp_dir().join("wows-ime-write-test");
        let _ = fs::remove_dir_all(&temp);
        let target = temp.join("bin").join("1").join("res_mods").join("ime_config.xml");

        write_config_files(
            std::slice::from_ref(&target),
            &[definition("微软拼音", ImeCategory::ChineseSimplified)],
        )
        .unwrap();

        let bytes = fs::read(&target).unwrap();
        assert_eq!(&bytes[..3], b"<da", "应为 XML 文本而非 BOM 开头");
        assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]), "不应写入 BOM");
        assert_eq!(count_existing_files(std::slice::from_ref(&target)), 1);

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn has_game_executable_requires_the_known_exe() {
        let temp = std::env::temp_dir().join("wows-ime-exe-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        assert!(!has_game_executable(temp.to_str().unwrap()));

        fs::write(temp.join("WorldOfWarships.exe"), b"").unwrap();
        assert!(has_game_executable(temp.to_str().unwrap()));

        let _ = fs::remove_dir_all(&temp);
    }
}
