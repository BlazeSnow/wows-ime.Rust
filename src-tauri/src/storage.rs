//! 持久化：SQLite 设置库（简单设置 + 列表数据）与旧版 `config.json` 迁移。
//!
//! 表结构与原 WinUI 3 版本一致，便于迁移：
//! - `settings`：简单状态键值表（对应原 `ApplicationData.Current.LocalSettings`）
//! - `custom_game_paths`：自定义游戏路径
//! - `custom_input_methods`：自定义输入法
//!
//! 持久化异常一律静默吞掉，不影响主流程（与原版行为一致）。

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use serde::Deserialize;

use crate::models::{ImeCategory, PersistedGamePath, PersistedInputMethod};

/// 旧 WinUI 3 版本的 MSIX 包身份（决定其 LocalState 目录）。
pub const LEGACY_PACKAGE_FAMILY_NAME: &str = "BlazeSnow.WOWS-IME";
/// 旧配置文件名。
pub const LEGACY_CONFIG_FILE_NAME: &str = "config.json";
/// 迁移成功后旧配置改用的文件名。
pub const MIGRATED_LEGACY_CONFIG_FILE_NAME: &str = "config.json.migrated";

/// 界面语言模式键（对应原版 `Settings.Language`）。
pub const SETTING_LANGUAGE: &str = "Settings.Language";
/// 当前选中的游戏根目录键（对应原版 `Game.SelectedPath`）。
pub const SETTING_SELECTED_GAME_PATH: &str = "Game.SelectedPath";
/// 配置结构版本键（对应原版 `Settings.SchemaVersion`）。
pub const SETTING_SCHEMA_VERSION: &str = "Settings.SchemaVersion";
/// 当前配置结构版本。
pub const CURRENT_SCHEMA_VERSION: &str = "1";
/// 语言模式默认值：跟随系统。
pub const LANGUAGE_AUTO: &str = "auto";

/// 打开设置库；文件不存在时创建。
pub fn open(db_path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    initialize(&conn)?;
    Ok(conn)
}

/// 建表（幂等）。
pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS custom_game_paths (
             display_name TEXT NOT NULL,
             path         TEXT PRIMARY KEY,
             created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );
         CREATE TABLE IF NOT EXISTS custom_input_methods (
             display_name TEXT PRIMARY KEY,
             category     TEXT NOT NULL,
             created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );",
    )
}

/// 读取字符串设置项；不存在时返回 `default`。
pub fn get_setting(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

/// 写入字符串设置项（upsert）。
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// 读取全部自定义游戏路径（按更新时间、路径排序）。
pub fn load_custom_game_paths(conn: &Connection) -> Vec<PersistedGamePath> {
    let mut statement = match conn.prepare(
        "SELECT display_name, path FROM custom_game_paths ORDER BY updated_at, path;",
    ) {
        Ok(statement) => statement,
        Err(_) => return Vec::new(),
    };

    let rows = statement.query_map([], |row| {
        Ok(PersistedGamePath {
            display_name: row.get(0)?,
            path: row.get(1)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// 全量同步自定义游戏路径（upsert + 删除缺失行）。
pub fn save_custom_game_paths(
    conn: &Connection,
    paths: &[PersistedGamePath],
) -> rusqlite::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    for path in paths {
        transaction.execute(
            "INSERT INTO custom_game_paths (display_name, path, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(path) DO UPDATE SET
                 display_name = excluded.display_name,
                 updated_at = CURRENT_TIMESTAMP;",
            params![path.display_name, path.path],
        )?;
    }
    delete_missing_rows(&transaction, "custom_game_paths", "path", &paths_of(paths))?;
    transaction.commit()
}

/// 全量同步自定义输入法（upsert + 删除缺失行）。
pub fn save_custom_input_methods(
    conn: &Connection,
    methods: &[PersistedInputMethod],
) -> rusqlite::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    for method in methods {
        transaction.execute(
            "INSERT INTO custom_input_methods (display_name, category, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(display_name) DO UPDATE SET
                 category = excluded.category,
                 updated_at = CURRENT_TIMESTAMP;",
            params![method.display_name, method.category],
        )?;
    }
    let keys: Vec<String> = methods
        .iter()
        .map(|method| method.display_name.clone())
        .collect();
    delete_missing_rows(&transaction, "custom_input_methods", "display_name", &keys)?;
    transaction.commit()
}

/// 读取全部自定义输入法（按更新时间、名称排序）。
pub fn load_custom_input_methods(conn: &Connection) -> Vec<PersistedInputMethod> {
    let mut statement = match conn.prepare(
        "SELECT display_name, category FROM custom_input_methods ORDER BY updated_at, display_name;",
    ) {
        Ok(statement) => statement,
        Err(_) => return Vec::new(),
    };

    let rows = statement.query_map([], |row| {
        Ok(PersistedInputMethod {
            display_name: row.get(0)?,
            category: row.get(1)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

fn paths_of(paths: &[PersistedGamePath]) -> Vec<String> {
    paths.iter().map(|path| path.path.clone()).collect()
}

/// 删除不在保留集合中的行（表名与列名由调用方固定传入，不来自外部输入）。
fn delete_missing_rows(
    transaction: &rusqlite::Transaction<'_>,
    table_name: &str,
    key_column: &str,
    values_to_keep: &[String],
) -> rusqlite::Result<()> {
    if values_to_keep.is_empty() {
        transaction.execute(&format!("DELETE FROM {table_name};"), [])?;
        return Ok(());
    }
    let placeholders = (1..=values_to_keep.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<String>>()
        .join(", ");
    transaction.execute(
        &format!("DELETE FROM {table_name} WHERE {key_column} NOT IN ({placeholders});"),
        rusqlite::params_from_iter(values_to_keep.iter()),
    )?;
    Ok(())
}

/// 旧 WinUI 3 版本的 LocalState 目录（MSIX 包私有目录）。
pub fn legacy_local_state_dir() -> Option<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    Some(
        PathBuf::from(local_app_data)
            .join("Packages")
            .join(LEGACY_PACKAGE_FAMILY_NAME)
            .join("LocalState"),
    )
}

/// 旧版 `config.json` 结构（仅取迁移需要的字段）。
#[derive(Debug, Deserialize, Default)]
struct LegacyConfig {
    #[serde(rename = "SelectedGamePath")]
    selected_game_path: Option<String>,
    #[serde(rename = "GameDir")]
    game_dir: Option<String>,
    #[serde(rename = "GamePaths")]
    game_paths: Option<Vec<LegacyGamePath>>,
    #[serde(rename = "Ime")]
    input_methods: Option<Vec<LegacyInputMethod>>,
}

#[derive(Debug, Deserialize)]
struct LegacyGamePath {
    #[serde(rename = "Path")]
    path: Option<String>,
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyInputMethod {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Category")]
    category: Option<String>,
}

/// 迁移旧版 `config.json`：成功后把文件改名为 `config.json.migrated`。
///
/// 迁移内容：
/// - `SelectedGamePath` / `GameDir` → `Game.SelectedPath`
/// - `GamePaths` → `custom_game_paths`
/// - `Ime` → `custom_input_methods`
///
/// 返回是否发生了迁移。
pub fn migrate_legacy_config_json(conn: &Connection, legacy_dir: &Path) -> bool {
    let legacy_path = legacy_dir.join(LEGACY_CONFIG_FILE_NAME);
    if !legacy_path.is_file() {
        return false;
    }

    let Ok(text) = std::fs::read_to_string(&legacy_path) else {
        return false;
    };
    let Ok(config) = serde_json::from_str::<LegacyConfig>(&text) else {
        return false;
    };

    let selected = config
        .selected_game_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            config
                .game_dir
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        });
    if let Some(selected) = selected {
        let _ = set_setting(conn, SETTING_SELECTED_GAME_PATH, selected);
    }

    if let Some(game_paths) = &config.game_paths {
        let migrated: Vec<PersistedGamePath> = game_paths
            .iter()
            .filter_map(|entry| {
                let path = entry.path.as_deref().map(str::trim).filter(|v| !v.is_empty())?;
                let display_name = entry
                    .name
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .or_else(|| {
                        Path::new(path)
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| path.to_string());
                Some(PersistedGamePath { display_name, path: path.to_string() })
            })
            .collect();
        if !migrated.is_empty() {
            let _ = save_custom_game_paths(conn, &migrated);
        }
    }

    if let Some(input_methods) = &config.input_methods {
        let migrated: Vec<PersistedInputMethod> = input_methods
            .iter()
            .filter_map(|entry| {
                let name = entry.name.as_deref().map(str::trim).filter(|v| !v.is_empty())?;
                let category = ImeCategory::from_stored(entry.category.as_deref().unwrap_or_default());
                Some(PersistedInputMethod {
                    display_name: name.to_string(),
                    category: category.as_str().to_string(),
                })
            })
            .collect();
        if !migrated.is_empty() {
            let _ = save_custom_input_methods(conn, &migrated);
        }
    }

    rename_migrated_legacy_config(&legacy_path, legacy_dir);
    true
}

fn rename_migrated_legacy_config(legacy_path: &Path, legacy_dir: &Path) {
    let migrated_path = legacy_dir.join(MIGRATED_LEGACY_CONFIG_FILE_NAME);
    if migrated_path.exists() {
        let _ = std::fs::remove_file(&migrated_path);
    }
    let _ = std::fs::rename(legacy_path, migrated_path);
}

#[cfg(test)]
mod tests {
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
        assert_eq!(get_setting(&conn, SETTING_LANGUAGE, LANGUAGE_AUTO), "auto");
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
}
