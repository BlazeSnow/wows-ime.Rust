//! storage 模块的单元测试。
//! 由 mod tests; 引入；use super::* 可访问父模块的私有项。

use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    initialize(&conn).unwrap();
    conn
}

#[test]
fn legacy_migration_keeps_paths_added_by_this_version() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-keep-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // 本版本自己添加的自定义路径
    save_custom_game_paths(
        &conn,
        &[PersistedGamePath {
            display_name: "本版本路径".to_string(),
            path: "C:\\Current".to_string(),
        }],
    )
    .unwrap();

    std::fs::write(
        dir.join(LEGACY_CONFIG_FILE_NAME),
        r#"{ "GamePaths": [ { "Path": "C:\\Legacy", "Name": "旧路径" } ] }"#,
    )
    .unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));

    // 迁移只增不减：两条路径都应在
    let loaded = load_custom_game_paths(&conn);
    assert_eq!(loaded.len(), 2, "迁移不应删除本版本已保存的路径: {loaded:?}");
    assert!(loaded.iter().any(|item| item.path == "C:\\Current"));
    assert!(loaded.iter().any(|item| item.path == "C:\\Legacy"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn legacy_migration_runs_only_once_even_if_rename_fails() {
    let conn = conn();
    let dir = std::env::temp_dir().join("wows-ime-legacy-once-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let legacy = dir.join(LEGACY_CONFIG_FILE_NAME);
    let body = r#"{ "GamePaths": [ { "Path": "C:\\Legacy", "Name": "旧路径" } ] }"#;
    std::fs::write(&legacy, body).unwrap();

    assert!(migrate_legacy_config_json(&conn, &dir));

    // 模拟改名失败：旧文件仍在（如被占用、目录只读）
    std::fs::write(&legacy, body).unwrap();

    // 有迁移标记在，不再重复迁移（否则会覆盖用户后来新增的数据）
    assert!(!migrate_legacy_config_json(&conn, &dir));
    assert_eq!(load_custom_game_paths(&conn).len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn initialize_is_idempotent() {
    let conn = conn();
    initialize(&conn).unwrap();
}

#[test]
fn legacy_table_without_default_is_rebuilt_and_data_kept() {
    let conn = Connection::open_in_memory().unwrap();
    // 早期版本建表：created_at / updated_at 是 NOT NULL 但没有默认值
    conn.execute_batch(
        "CREATE TABLE custom_game_paths (
             path         TEXT PRIMARY KEY,
             display_name TEXT NOT NULL,
             created_at   TEXT NOT NULL,
             updated_at   TEXT NOT NULL
         );",
    )
    .unwrap();

    // 写入语句显式带上 created_at，旧结构下也能写进去（这正是丢数据的根因）
    save_custom_game_paths(
        &conn,
        &[PersistedGamePath {
            display_name: "新路径".to_string(),
            path: "C:\\New".to_string(),
        }],
    )
    .unwrap();
    assert_eq!(load_custom_game_paths(&conn).len(), 1);

    // 迁移保留数据：直接往旧表塞一行（绕开全量同步），再初始化
    conn.execute(
        "INSERT INTO custom_game_paths (path, display_name, created_at, updated_at)
         VALUES ('C:\\Games', '旧路径', '2026-01-01 00:00:00', '2026-01-01 00:00:00')",
        [],
    )
    .unwrap();

    initialize(&conn).unwrap();
    let loaded = load_custom_game_paths(&conn);
    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().any(|item| item.path == "C:\\Games"));

    let ddl: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'custom_game_paths'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        ddl.to_uppercase().contains("DEFAULT"),
        "重建后的表应带默认值: {ddl}"
    );
}

#[test]
fn legacy_input_method_table_without_default_is_rebuilt() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE custom_input_methods (
             display_name TEXT PRIMARY KEY,
             category     TEXT NOT NULL,
             created_at   TEXT NOT NULL,
             updated_at   TEXT NOT NULL
         );
         INSERT INTO custom_input_methods (display_name, category, created_at, updated_at)
             VALUES ('旧输入法', 'Japanese', '2026-01-01 00:00:00', '2026-01-01 00:00:00');",
    )
    .unwrap();

    save_custom_input_methods(
        &conn,
        &[PersistedInputMethod {
            display_name: "新输入法".to_string(),
            category: "Japanese".to_string(),
        }],
    )
    .unwrap();

    initialize(&conn).unwrap();
    let loaded = load_custom_input_methods(&conn);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "新输入法");

    let ddl: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'custom_input_methods'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        ddl.to_uppercase().contains("DEFAULT"),
        "重建后的表应带默认值: {ddl}"
    );
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
