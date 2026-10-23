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
/// 旧版数据迁移完成标记（详见 `migrate_legacy_config_json`）。
pub const SETTING_LEGACY_MIGRATED: &str = "Settings.LegacyMigrated";

/// 打开设置库；文件不存在时创建。
pub fn open(db_path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    initialize(&conn)?;
    Ok(conn)
}

/// 建表（幂等），并修复早期版本遗留的表结构。
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
    )?;
    ensure_timestamp_defaults(conn)
}

/// 早期版本建表时 `created_at` / `updated_at` 没有默认值，而写入语句只提供部分列，
/// 会触发 `NOT NULL constraint failed` 导致保存静默失败。
/// `CREATE TABLE IF NOT EXISTS` 不会改动已存在的表，因此这里检测并重建（保留数据）。
fn ensure_timestamp_defaults(conn: &Connection) -> rusqlite::Result<()> {
    for table in ["custom_game_paths", "custom_input_methods"] {
        if rebuild_table_without_defaults(conn, table)? {
            eprintln!("[storage] 已重建表 {table}：补上 created_at / updated_at 默认值");
        }
    }
    Ok(())
}

/// 表的建表语句里某列是否带 DEFAULT。
fn column_has_default(sql: &str, column: &str) -> bool {
    sql.lines().any(|line| {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(column) else {
            return false;
        };
        // 确认命中的是该列的定义行（其后应是类型名）
        let rest = rest.trim_start();
        rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && rest.to_uppercase().contains("DEFAULT")
    })
}

/// 重建缺少默认值的表（保留已有数据）。返回是否发生了重建。
fn rebuild_table_without_defaults(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    let ddl: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table],
            |row| row.get(0),
        )
        .ok();
    let Some(ddl) = ddl else {
        return Ok(false);
    };
    if column_has_default(&ddl, "created_at") && column_has_default(&ddl, "updated_at") {
        return Ok(false);
    }

    let (columns, column_list, create) = match table {
        "custom_game_paths" => (
            "(display_name, path, created_at, updated_at)",
            "display_name, path, created_at, updated_at",
            "CREATE TABLE {name}_new (
             display_name TEXT NOT NULL,
             path         TEXT PRIMARY KEY,
             created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );",
        ),
        "custom_input_methods" => (
            "(display_name, category, created_at, updated_at)",
            "display_name, category, created_at, updated_at",
            "CREATE TABLE {name}_new (
             display_name TEXT PRIMARY KEY,
             category     TEXT NOT NULL,
             created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );",
        ),
        _ => return Ok(false),
    };
    let sql = format!(
        "{create}
         INSERT INTO {{name}}_new {columns}
             SELECT {column_list} FROM {{name}};
         DROP TABLE {{name}};
         ALTER TABLE {{name}}_new RENAME TO {{name}};",
        columns = columns,
        column_list = column_list
    )
    .replace("{name}", table);
    conn.execute_batch(&sql)?;
    Ok(true)
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
            "INSERT INTO custom_game_paths (display_name, path, created_at, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
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
            "INSERT INTO custom_input_methods (display_name, category, created_at, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
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
    // 已迁移过就不再重复，不依赖文件改名是否成功
    if get_setting(conn, SETTING_LEGACY_MIGRATED, "") == "1" {
        return false;
    }
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
        if !migrated.is_empty() && upsert_custom_game_paths(conn, &migrated).is_err() {
            return false;
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
        if !migrated.is_empty() && upsert_custom_input_methods(conn, &migrated).is_err() {
            return false;
        }
    }

    // 数据落库成功后才记标记并改名；任一步失败都保留 config.json，下次启动会重试。
    if set_setting(conn, SETTING_LEGACY_MIGRATED, "1").is_err() {
        return false;
    }
    rename_migrated_legacy_config(&legacy_path, legacy_dir);
    true
}

/// 旧版数据迁移完成标记。
///
/// 原 WinUI 3 版靠「把 config.json 改名成 config.json.migrated」来保证只迁移一次；
/// 改名失败（文件被占用、目录只读）时旧文件仍在，下次启动会重复迁移，
/// 而迁移写入若沿用全量同步语义，就会把用户后来新增的自定义路径删掉。
/// 标记存进库，不依赖改名是否成功。

/// 只写入（upsert）不删除。迁移专用：绝不能动用户已有的数据。
fn upsert_custom_game_paths(
    conn: &Connection,
    paths: &[PersistedGamePath],
) -> rusqlite::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    for path in paths {
        transaction.execute(
            "INSERT INTO custom_game_paths (display_name, path, created_at, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(path) DO UPDATE SET
                 display_name = excluded.display_name,
                 updated_at = CURRENT_TIMESTAMP;",
            params![path.display_name, path.path],
        )?;
    }
    transaction.commit()
}

/// 只写入（upsert）不删除。迁移专用。
fn upsert_custom_input_methods(
    conn: &Connection,
    methods: &[PersistedInputMethod],
) -> rusqlite::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    for method in methods {
        transaction.execute(
            "INSERT INTO custom_input_methods (display_name, category, created_at, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(display_name) DO UPDATE SET
                 category = excluded.category,
                 updated_at = CURRENT_TIMESTAMP;",
            params![method.display_name, method.category],
        )?;
    }
    transaction.commit()
}

fn rename_migrated_legacy_config(legacy_path: &Path, legacy_dir: &Path) {
    let migrated_path = legacy_dir.join(MIGRATED_LEGACY_CONFIG_FILE_NAME);
    if migrated_path.exists() {
        let _ = std::fs::remove_file(&migrated_path);
    }
    let _ = std::fs::rename(legacy_path, migrated_path);
}

#[cfg(test)]
mod tests;
