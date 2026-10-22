//! 全局应用状态。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

/// 应用数据目录（SQLite 设置库与旧版迁移来源都在其下）。
pub struct AppState {
    pub data_dir: PathBuf,
    /// 设置库连接；rusqlite 的连接非线程安全，用互斥量串行化访问。
    pub db: Mutex<Connection>,
}

impl AppState {
    /// 设置库路径：`<数据目录>\settings.db`。
    pub fn db_path(data_dir: &Path) -> PathBuf {
        data_dir.join("settings.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_is_settings_db_under_data_dir() {
        assert_eq!(
            AppState::db_path(Path::new("app-data")),
            PathBuf::from("app-data").join("settings.db")
        );
    }
}
