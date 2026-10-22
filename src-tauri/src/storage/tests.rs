//! storage 模块的单元测试。
//! 由 mod tests; 引入；use super::* 可访问父模块的私有项。

use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    initialize(&conn).unwrap();
    conn
}

#[test]
fn initialize_is_idempotent() {
    let conn = conn();
    initialize(&conn).unwrap();
}

#[test]
fn setting_defaults_when_absent() {
    let conn = conn();
    assert_eq!(
        get_setting(&conn, SETTING_LANGUAGE, crate::language::AUTOMATIC),
        "auto"
    );
}

#[test]
fn setting_upserts_overwrites_value() {
    let conn = conn();
    set_setting(&conn, SETTING_LANGUAGE, "ja").unwrap();
    assert_eq!(get_setting(&conn, SETTING_LANGUAGE, "auto"), "ja");
    set_setting(&conn, SETTING_LANGUAGE, "zh-Hant").unwrap();
    assert_eq!(get_setting(&conn, SETTING_LANGUAGE, "auto"), "zh-Hant");
}

#[test]
fn custom_game_paths_round_trip() {
    let conn = conn();
    save_custom_game_paths(
        &conn,
        &[
            PersistedGamePath {
                display_name: "Steam".to_string(),
                path: "C:\\Steam".to_string(),
            },
            PersistedGamePath {
                display_name: "360".to_string(),
                path: "C:\\360".to_string(),
            },
        ],
    )
    .unwrap();

    let loaded = load_custom_game_paths(&conn);
    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().any(|p| p.path == "C:\\Steam"));
}

#[test]
fn saving_removes_rows_that_are_no_longer_present() {
    let conn = conn();
    let paths = vec![PersistedGamePath {
        display_name: "A".to_string(),
        path: "C:\\A".to_string(),
    }];
    save_custom_game_paths(&conn, &paths).unwrap();
    save_custom_game_paths(&conn, &[]).unwrap();
    assert!(load_custom_game_paths(&conn).is_empty());
}

#[test]
fn custom_input_methods_round_trip() {
    let conn = conn();
    save_custom_input_methods(
        &conn,
        &[PersistedInputMethod {
            display_name: "自定义输入法".to_string(),
            category: "Japanese".to_string(),
        }],
    )
    .unwrap();

    let loaded = load_custom_input_methods(&conn);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].category, "Japanese");
}

#[test]
fn legacy_config_json_is_migrated_and_renamed() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{
            "SelectedGamePath": "C:\\Games\\WoWS",
            "GamePaths": [ { "Path": "C:\\Games\\WoWS2", "Name": "自定义" } ],
            "Ime": [ { "Name": "旧输入法", "Category": "ChineseTraditional" } ]
        }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));
    assert_eq!(
        get_setting(&conn, SETTING_SELECTED_GAME_PATH, ""),
        "C:\\Games\\WoWS"
    );
    assert_eq!(load_custom_game_paths(&conn).len(), 1);
    assert_eq!(load_custom_input_methods(&conn)[0].category, "ChineseTraditional");
    assert!(dir.join(MIGRATED_LEGACY_CONFIG_FILE_NAME).is_file());
    assert!(!dir.join(LEGACY_CONFIG_FILE_NAME).exists());

    // 迁移只发生一次
    assert!(!migrate_legacy_config_json(&conn, &dir));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn saving_same_path_updates_its_display_name() {
    let conn = conn();
    save_custom_game_paths(
        &conn,
        &[PersistedGamePath {
            display_name: "旧名称".to_string(),
            path: "C:\\Games".to_string(),
        }],
    )
    .unwrap();
    save_custom_game_paths(
        &conn,
        &[PersistedGamePath {
            display_name: "新名称".to_string(),
            path: "C:\\Games".to_string(),
        }],
    )
    .unwrap();

    let loaded = load_custom_game_paths(&conn);
    assert_eq!(loaded.len(), 1, "同一路径应被更新而不是新增");
    assert_eq!(loaded[0].display_name, "新名称");
}

#[test]
fn legacy_game_dir_is_used_when_selected_game_path_is_absent() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-gamedir-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{ "GameDir": "C:\\Games\\Legacy" }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));
    assert_eq!(
        get_setting(&conn, SETTING_SELECTED_GAME_PATH, ""),
        "C:\\Games\\Legacy"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_game_path_without_name_uses_directory_name() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-noname-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{ "GamePaths": [ { "Path": "C:\\Games\\WoWS" } ] }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));
    let loaded = load_custom_game_paths(&conn);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "WoWS");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_unknown_category_falls_back_to_simplified() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-category-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{ "Ime": [ { "Name": "未知分类", "Category": "Klingon" } ] }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));
    assert_eq!(load_custom_input_methods(&conn)[0].category, "ChineseSimplified");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_input_method_without_name_is_skipped() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-noname-ime-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{ "Ime": [ { "Category": "Japanese" }, { "Name": "有名字", "Category": "Japanese" } ] }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));
    let loaded = load_custom_input_methods(&conn);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "有名字");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn malformed_legacy_config_is_left_untouched() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-broken-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let broken = dir.join(LEGACY_CONFIG_FILE_NAME);
    std::fs::write(&broken, "{ 这不是合法 JSON").unwrap();

    assert!(!migrate_legacy_config_json(&conn, &dir));
    assert!(broken.is_file(), "解析失败时不应改名或删除旧文件");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_config_with_non_object_root_is_ignored() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-array-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(LEGACY_CONFIG_FILE_NAME), r#"[1, 2, 3]"#).unwrap();

    assert!(!migrate_legacy_config_json(&conn, &dir));
    assert!(load_custom_game_paths(&conn).is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_local_state_dir_points_at_msix_package_folder() {
    let Some(dir) = legacy_local_state_dir() else {
        // 无 LOCALAPPDATA 的环境（如 CI 的某些沙箱）跳过断言
        return;
    };
    let expected = Path::new("Packages")
        .join(LEGACY_PACKAGE_FAMILY_NAME)
        .join("LocalState");
    assert!(
        dir.ends_with(&expected),
        "{} 应以 {} 结尾",
        dir.display(),
        expected.display()
    );
}
