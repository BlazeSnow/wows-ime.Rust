//! gameconfig 模块的单元测试。
//! 由 mod tests; 引入；use super::* 可访问父模块的私有项。

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

#[test]
fn has_game_executable_also_accepts_russian_client_exe() {
    let temp = std::env::temp_dir().join("wows-ime-korabli-test");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();
    fs::write(temp.join("Korabli.exe"), b"").unwrap();
    assert!(has_game_executable(temp.to_str().unwrap()));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn empty_or_missing_game_root_has_no_targets_and_no_exe() {
    assert!(!has_game_executable(""));
    assert!(!has_game_executable("   "));
    assert!(resolve_target_config_files("").is_empty());

    let missing = std::env::temp_dir().join("wows-ime-missing-root");
    let _ = fs::remove_dir_all(&missing);
    assert!(resolve_target_config_files(missing.to_str().unwrap()).is_empty());
}

#[test]
fn resolve_returns_nothing_when_bin_has_no_numeric_version_dir() {
    let temp = std::env::temp_dir().join("wows-ime-no-version-test");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(temp.join("bin").join("patch")).unwrap();
    // 形似版本号但其实是文件的条目也要排除
    fs::write(temp.join("bin").join("12345"), b"").unwrap();

    assert!(resolve_target_config_files(temp.to_str().unwrap()).is_empty());

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn each_input_method_lands_in_its_own_group() {
    let document = build_config_document(&[
        definition("微软拼音", ImeCategory::ChineseSimplified),
        definition("Microsoft 日本語", ImeCategory::Japanese),
        definition("微軟速成", ImeCategory::ChineseTraditional),
    ]);

    let simplified = document
        .split("<ChineseSimplified>")
        .nth(1)
        .unwrap()
        .split("</ChineseSimplified>")
        .next()
        .unwrap();
    let japanese = document
        .split("<Japanese>")
        .nth(1)
        .unwrap()
        .split("</Japanese>")
        .next()
        .unwrap();
    let traditional = document
        .split("<ChineseTraditional>")
        .nth(1)
        .unwrap()
        .split("</ChineseTraditional>")
        .next()
        .unwrap();

    assert!(simplified.contains("微软拼音"), "{document}");
    assert!(!simplified.contains("微軟速成"), "{document}");
    assert!(japanese.contains("Microsoft 日本語"), "{document}");
    assert!(!japanese.contains("微软拼音"), "{document}");
    assert!(traditional.contains("微軟速成"), "{document}");
    assert!(!traditional.contains("Microsoft 日本語"), "{document}");
}

#[test]
fn write_covers_every_target_file() {
    let temp = std::env::temp_dir().join("wows-ime-multi-write-test");
    let _ = fs::remove_dir_all(&temp);
    let targets = vec![
        temp.join("bin").join("1").join("res_mods").join("ime_config.xml"),
        temp.join("bin").join("2").join("res_mods").join("ime_config.xml"),
    ];

    write_config_files(
        &targets,
        &[definition("微软拼音", ImeCategory::ChineseSimplified)],
    )
    .unwrap();

    for target in &targets {
        assert!(target.is_file(), "{} 应已写入", target.display());
        let text = fs::read_to_string(target).unwrap();
        assert!(text.contains("微软拼音"), "{text}");
    }
    assert_eq!(count_existing_files(&targets), 2);

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn count_existing_files_counts_only_present_files() {
    let temp = std::env::temp_dir().join("wows-ime-count-test");
    let _ = fs::remove_dir_all(&temp);
    let present = temp.join("ime_config.xml");
    let absent = temp.join("other.xml");
    fs::create_dir_all(&temp).unwrap();
    fs::write(&present, b"<data />").unwrap();

    assert_eq!(count_existing_files(&[present.clone(), absent]), 1);
    assert_eq!(count_existing_files(&[]), 0);

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn write_failure_reports_the_offending_path() {
    let temp = std::env::temp_dir().join("wows-ime-write-fail-test");
    let _ = fs::remove_dir_all(&temp);
    // 把 res_mods 建成文件，导致创建目录失败
    let blocker = temp.join("bin").join("1").join("res_mods");
    fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    fs::write(&blocker, b"not a directory").unwrap();

    let target = blocker.join("ime_config.xml");
    let failure = write_config_files(
        std::slice::from_ref(&target),
        &[definition("微软拼音", ImeCategory::ChineseSimplified)],
    )
    .expect_err("应写入失败");

    assert_eq!(failure.path, target);
    assert!(failure.to_string().contains("res_mods"), "{failure}");

    let _ = fs::remove_dir_all(&temp);
}
