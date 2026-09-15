pub mod migrations;
pub mod service;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

/// 数据库目录名。
/// - Windows: `%APPDATA%\自媒体AI工作台\workbench.db`
/// - macOS:   `~/Library/Application Support/自媒体AI工作台/workbench.db`
pub const DB_DIR_NAME: &str = "自媒体AI工作台";
pub const DB_FILE_NAME: &str = "workbench.db";

/// 解析数据库文件路径，并确保目录存在。
pub fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = app.path().data_dir()?;
    let dir = base.join(DB_DIR_NAME);
    fs::create_dir_all(&dir)?;
    Ok(dir.join(DB_FILE_NAME))
}

/// 打开数据库：先设 PRAGMA，再跑迁移。首次启动会自动建库建表。
pub fn connect(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    migrations::run(&conn)?;
    Ok(conn)
}

/// PRAGMA 用 execute_batch 设置：journal_mode 会返回一行结果，
/// 用 execute 会报 "Execute returned results"。
pub fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
}
