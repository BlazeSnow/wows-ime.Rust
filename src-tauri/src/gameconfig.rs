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
mod tests;
