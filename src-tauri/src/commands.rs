//! Tauri command 层。
//!
//! 这一层只做三件事：取状态、调 service、把错误转成字符串。
//! 通用 CRUD 只有一套命令，前端传表名和条件即可，不需要每张表写一遍。

use serde::Serialize;
use serde_json::{json, Map, Value};
use tauri::State;

use crate::db::{migrations, service};
use crate::db::service::{Filter, ListQuery, TableInfo};
use crate::AppState;

type CommandResult<T> = Result<T, String>;

macro_rules! conn {
    ($state:expr) => {
        $state.db.lock().map_err(|error| error.to_string())?
    };
}

/* --------------------------------- 状态查询 --------------------------------- */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    ready: bool,
    /// 数据库文件绝对路径
    path: String,
    /// 所在目录
    directory: String,
    /// SQLite 库版本
    version: String,
    /// schema 版本（PRAGMA user_version）
    schema_version: i32,
    /// 已注册的业务表数量
    table_count: usize,
}

#[tauri::command]
pub fn db_status(state: State<'_, AppState>) -> CommandResult<DbStatus> {
    let conn = conn!(state);
    let version: String = conn
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;

    Ok(DbStatus {
        ready: true,
        path: state.path.display().to_string(),
        directory: state
            .path
            .parent()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default(),
        version,
        schema_version: migrations::schema_version(&conn).map_err(|error| error.to_string())?,
        table_count: service::TABLES.len(),
    })
}

/// 表结构 + 行数，用于设置页自检
#[tauri::command]
pub fn db_describe(state: State<'_, AppState>) -> CommandResult<Vec<TableInfo>> {
    let conn = conn!(state);
    service::describe(&conn)
}

/* --------------------------------- 通用 CRUD -------------------------------- */

#[tauri::command]
pub fn db_list(
    state: State<'_, AppState>,
    table: String,
    query: Option<ListQuery>,
) -> CommandResult<Vec<Value>> {
    let conn = conn!(state);
    service::list(&conn, &table, &query.unwrap_or_default())
}

#[tauri::command]
pub fn db_get(state: State<'_, AppState>, table: String, id: i64) -> CommandResult<Option<Value>> {
    let conn = conn!(state);
    service::get(&conn, &table, id)
}

#[tauri::command]
pub fn db_insert(
    state: State<'_, AppState>,
    table: String,
    data: Map<String, Value>,
) -> CommandResult<Value> {
    let conn = conn!(state);
    let row = service::insert(&conn, &table, &data)?;
    // 数据变更后自动同步记忆库索引
    crate::services::memory::commands::sync_for_table(&conn, &table, &row);
    Ok(row)
}

#[tauri::command]
pub fn db_update(
    state: State<'_, AppState>,
    table: String,
    id: i64,
    data: Map<String, Value>,
) -> CommandResult<Value> {
    let conn = conn!(state);
    let row = service::update(&conn, &table, id, &data)?;
    crate::services::memory::commands::sync_for_table(&conn, &table, &row);
    Ok(row)
}

#[tauri::command]
pub fn db_delete(state: State<'_, AppState>, table: String, id: i64) -> CommandResult<bool> {
    let conn = conn!(state);
    let removed = service::delete(&conn, &table, id)?;
    if removed {
        crate::services::memory::commands::remove_for_table(&conn, &table, id);
    }
    Ok(removed)
}

#[tauri::command]
pub fn db_count(
    state: State<'_, AppState>,
    table: String,
    filters: Option<Vec<Filter>>,
) -> CommandResult<i64> {
    let conn = conn!(state);
    service::count(&conn, &table, &filters.unwrap_or_default())
}

/* ---------------------------------- 配置项 ---------------------------------- */

#[tauri::command]
pub fn setting_get(state: State<'_, AppState>, key: String) -> CommandResult<Option<String>> {
    let conn = conn!(state);
    service::setting_get(&conn, &key)
}

#[tauri::command]
pub fn setting_set(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> CommandResult<String> {
    let conn = conn!(state);
    service::setting_set(&conn, &key, &value)
}

#[tauri::command]
pub fn setting_all(state: State<'_, AppState>) -> CommandResult<Vec<Value>> {
    let conn = conn!(state);
    service::setting_all(&conn)
}

/* --------------------------------- AI 用量 --------------------------------- */

/// 记一次 AI 调用，用于免费额度控制
#[tauri::command]
pub fn ai_usage_record(
    state: State<'_, AppState>,
    model: String,
    kind: String,
    tokens: Option<i64>,
    is_free_trial: Option<bool>,
) -> CommandResult<Value> {
    let conn = conn!(state);
    service::insert(
        &conn,
        "ai_usage",
        &json!({
            "model": model,
            "type": kind,
            "tokens": tokens.unwrap_or(0),
            "is_free_trial": is_free_trial.unwrap_or(false),
        })
        .as_object()
        .cloned()
        .unwrap_or_default(),
    )
}

/// 用量汇总：总次数、免费体验次数、累计 tokens
#[tauri::command]
pub fn ai_usage_summary(state: State<'_, AppState>) -> CommandResult<Value> {
    let conn = conn!(state);
    let total = service::count(&conn, "ai_usage", &[])?;
    let free_trial = service::count(
        &conn,
        "ai_usage",
        &[Filter {
            column: "is_free_trial".to_string(),
            op: "eq".to_string(),
            value: Some(json!(1)),
        }],
    )?;
    let tokens: i64 = conn
        .query_row("SELECT COALESCE(SUM(tokens), 0) FROM ai_usage", [], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;

    Ok(json!({
        "total": total,
        "freeTrial": free_trial,
        "tokens": tokens,
    }))
}

/* ---------------------------------- 自检 ---------------------------------- */

/// 写一条 notes 再读回来，验证「前端 -> command -> service -> SQLite」整条链路。
#[tauri::command]
pub fn db_self_test(state: State<'_, AppState>) -> CommandResult<Value> {
    let conn = conn!(state);
    let stamp = conn
        .query_row("SELECT datetime('now', 'localtime')", [], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| error.to_string())?;
    let content = format!("数据库自检记录 · {stamp}");

    let written = service::insert(
        &conn,
        "notes",
        &json!({ "content": content, "converted_to": Value::Null })
            .as_object()
            .cloned()
            .unwrap_or_default(),
    )?;

    let id = written
        .get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| "写入后没有拿到 id".to_string())?;
    let read_back = service::get(&conn, "notes", id)?
        .ok_or_else(|| format!("读不到刚写入的 notes #{id}"))?;

    Ok(json!({
        "ok": read_back["content"] == written["content"],
        "table": "notes",
        "written": written,
        "readBack": read_back,
        "notesCount": service::count(&conn, "notes", &[])?,
    }))
}
