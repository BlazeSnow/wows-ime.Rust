//! 持久化：SQLite 设置库（简单设置 + 列表数据）与旧版 `config.json` 迁移。
//!
//! 表结构与原 WinUI 3 版本一致，便于迁移：
//! - `settings`：简单状态键值表（对应原 `ApplicationData.Current.LocalSettings`）
//! - `custom_game_paths`：自定义游戏路径
//! - `custom_input_methods`：自定义输入法

use std::path::Path;

use rusqlite::{params, Connection};

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
             path         TEXT PRIMARY KEY,
             display_name TEXT NOT NULL,
             created_at   TEXT NOT NULL,
             updated_at   TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS custom_input_methods (
             display_name TEXT PRIMARY KEY,
             category     TEXT NOT NULL,
             created_at   TEXT NOT NULL,
             updated_at   TEXT NOT NULL
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
        assert_eq!(get_setting(&conn, "Settings.Language", "auto"), "auto");
    }

    #[test]
    fn setting_upserts_overwrites_value() {
        let conn = conn();
        set_setting(&conn, "Settings.Language", "ja").unwrap();
        assert_eq!(get_setting(&conn, "Settings.Language", "auto"), "ja");
        set_setting(&conn, "Settings.Language", "zh-Hant").unwrap();
        assert_eq!(get_setting(&conn, "Settings.Language", "auto"), "zh-Hant");
    }
}
