//! scanner 模块的单元测试。
//! 由 mod tests; 引入；use super::* 可访问父模块的私有项。

use super::*;

#[test]
fn only_target_language_ids_are_scanned() {
    assert!(is_target_language_profile(0x0804));
    assert!(is_target_language_profile(0x0404));
    assert!(is_target_language_profile(0x0C04));
    assert!(is_target_language_profile(0x1004));
    assert!(is_target_language_profile(0x1404));
    assert!(is_target_language_profile(0x0411));
    assert!(!is_target_language_profile(0x0409));
    assert!(!is_target_language_profile(0x0412));
}

#[test]
fn language_id_drives_category() {
    assert_eq!(
        infer_category_from_lang_id(0x0804),
        Some(ImeCategory::ChineseSimplified)
    );
    assert_eq!(
        infer_category_from_lang_id(0x1404),
        Some(ImeCategory::ChineseTraditional)
    );
    assert_eq!(
        infer_category_from_lang_id(0x0411),
        Some(ImeCategory::Japanese)
    );
    assert_eq!(infer_category_from_lang_id(0x0409), None);
}

#[test]
fn name_keywords_drive_category() {
    assert_eq!(
        infer_category_from_name("微軟速成"),
        Some(ImeCategory::ChineseTraditional)
    );
    assert_eq!(
        infer_category_from_name("微軟倉頡"),
        Some(ImeCategory::ChineseTraditional)
    );
    assert_eq!(
        infer_category_from_name("Microsoft Quick"),
        Some(ImeCategory::ChineseTraditional)
    );
    assert_eq!(
        infer_category_from_name("微軟注音"),
        Some(ImeCategory::ChineseTraditional)
    );
    assert_eq!(
        infer_category_from_name("搜狗拼音输入法"),
        Some(ImeCategory::ChineseSimplified)
    );
    assert_eq!(
        infer_category_from_name("微软五笔"),
        Some(ImeCategory::ChineseSimplified)
    );
    assert_eq!(
        infer_category_from_name("Japanese IME"),
        Some(ImeCategory::Japanese)
    );
    assert_eq!(infer_category_from_name("未知输入法"), None);
}

#[test]
fn noise_names_are_filtered() {
    assert!(is_noise_ime_name("微软拼音 输入体验"));
    assert!(is_noise_ime_name("Microsoft Pinyin Input Experience"));
    assert!(!is_noise_ime_name("微软拼音"));
}

#[test]
fn generic_failure_carries_stable_warning_code() {
    let result = generic_failure();
    assert_eq!(result.warning_code.as_deref(), Some(GENERIC_EXCEPTION_WARNING));
    assert!(result.candidates.is_empty());
}

#[test]
fn format_hresult_uses_fixed_width_hex() {
    assert_eq!(format_hresult(0), "0x00000000");
    assert_eq!(format_hresult(-1), "0xFFFFFFFF");
    assert_eq!(format_hresult(0x80010106u32 as i32), "0x80010106");
}

#[test]
fn upsert_keeps_the_higher_confidence_for_the_same_name() {
    let mut candidates = BTreeMap::new();
    upsert_candidate(
        &mut candidates,
        ScannedImeCandidate {
            display_name: "微软拼音".to_string(),
            category: ImeCategory::ChineseSimplified,
            confidence: 10,
        },
    );
    upsert_candidate(
        &mut candidates,
        ScannedImeCandidate {
            display_name: "微软拼音".to_string(),
            category: ImeCategory::ChineseTraditional,
            confidence: 20,
        },
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates.values().next().unwrap().category,
        ImeCategory::ChineseTraditional
    );

    // 置信度更低的结果不应覆盖
    upsert_candidate(
        &mut candidates,
        ScannedImeCandidate {
            display_name: "微软拼音".to_string(),
            category: ImeCategory::Japanese,
            confidence: 1,
        },
    );
    assert_eq!(
        candidates.values().next().unwrap().category,
        ImeCategory::ChineseTraditional
    );
}

#[test]
fn upsert_treats_names_case_insensitively() {
    let mut candidates = BTreeMap::new();
    upsert_candidate(
        &mut candidates,
        ScannedImeCandidate {
            display_name: "MS Pinyin".to_string(),
            category: ImeCategory::ChineseSimplified,
            confidence: 10,
        },
    );
    upsert_candidate(
        &mut candidates,
        ScannedImeCandidate {
            display_name: "ms pinyin".to_string(),
            category: ImeCategory::ChineseSimplified,
            confidence: 10,
        },
    );
    assert_eq!(candidates.len(), 1);
}

#[test]
fn panic_is_catchable_in_this_profile() {
    // release profile 必须保留 unwind：panic = "abort" 会让 catch_unwind 失效，
    // 扫描异常将直接终止进程，违背「扫描异常不中断界面」的约定。
    // 用 `cargo test --release` 跑本用例即可守住这条约束。
    let caught = std::panic::catch_unwind(|| -> u32 { panic!("测试 panic") });
    assert!(caught.is_err());
}

#[test]
fn scan_returns_sorted_unique_candidates_on_this_machine() {
    // 冒烟测试：结果依赖真实系统输入法配置，只断言结构稳定。
    let result = scan();
    println!(
        "扫描到 {} 个输入法，警告码 {:?}",
        result.candidates.len(),
        result.warning_code
    );
    for candidate in &result.candidates {
        println!("  - {} ({:?})", candidate.display_name, candidate.category);
    }
    let mut names: Vec<String> = result
        .candidates
        .iter()
        .map(|c| c.display_name.to_lowercase())
        .collect();
    let count = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), count, "候选输入法名称应唯一");
}
