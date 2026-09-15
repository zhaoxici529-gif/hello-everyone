//! 记忆库的 Tauri command + 供其它模块调用的索引/检索接口。

use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;
use serde_json::{json, Map, Value};
use tauri::State;

use crate::db::service::{self, to_sql_value, Filter, ListQuery};
use crate::services::ai::AiError;
use crate::services::memory::{is_source, score, source_label, source_of_table, summarize, SOURCES};
use crate::AppState;

/// 导出时包含的表（settings 刻意排除：里面的 API Key 密文换台机器解不开）
const EXPORT_TABLES: [&str; 13] = [
    "personas",
    "materials",
    "material_categories",
    "writings",
    "topics",
    "bloggers",
    "blogger_posts",
    "todos",
    "notes",
    "plans",
    "pomodoro",
    "memories",
    "ai_usage",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRow {
    pub id: i64,
    pub source_type: String,
    pub source_label: String,
    pub source_id: i64,
    pub content: String,
    pub created_at: String,
    /// 检索时的相关度，列表接口不填
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub total: i64,
    pub by_source: Vec<SourceCount>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCount {
    pub source_type: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub tables: usize,
    pub rows: i64,
}

fn lock<'a>(state: &'a State<'_, AppState>) -> Result<std::sync::MutexGuard<'a, Connection>, AiError> {
    state
        .db
        .lock()
        .map_err(|error| AiError::internal(error.to_string()))
}

/* ------------------------------- 索引写入 ------------------------------- */

/// 删掉某条数据已有的记忆（重建时先删后插，保证幂等）
pub fn remove_source(conn: &Connection, source_type: &str, source_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM memories WHERE source_type = ?1 AND source_id = ?2",
        params![source_type, source_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// 把一条业务数据写进记忆库。数据变更后自动调用。
pub fn sync_row(conn: &Connection, source_type: &str, row: &Value) -> Result<bool, String> {
    if !is_source(source_type) {
        return Ok(false);
    }

    let source_id = row.get("id").and_then(Value::as_i64).unwrap_or_default();
    if source_id <= 0 {
        return Ok(false);
    }

    let content = summarize(source_type, row);
    if content.trim().is_empty() {
        return Ok(false);
    }

    remove_source(conn, source_type, source_id)?;
    conn.execute(
        "INSERT INTO memories (source_type, source_id, content) VALUES (?1, ?2, ?3)",
        params![source_type, source_id, content],
    )
    .map_err(|error| error.to_string())?;

    Ok(true)
}

/// 通用 CRUD 里根据表名同步索引
pub fn sync_for_table(conn: &Connection, table: &str, row: &Value) -> Option<String> {
    let source_type = source_of_table(table)?;
    sync_row(conn, source_type, row).ok()?;
    Some(source_type.to_string())
}

pub fn remove_for_table(conn: &Connection, table: &str, id: i64) {
    if let Some(source_type) = source_of_table(table) {
        let _ = remove_source(conn, source_type, id);
    }
}

/* --------------------------------- 检索 --------------------------------- */

fn row_to_memory(row: &Value, related: f64) -> MemoryRow {
    let source_type = row
        .get("source_type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    MemoryRow {
        id: row.get("id").and_then(Value::as_i64).unwrap_or_default(),
        source_label: source_label(&source_type).to_string(),
        source_type,
        source_id: row.get("source_id").and_then(Value::as_i64).unwrap_or_default(),
        content: row
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        created_at: row
            .get("created_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        score: related,
    }
}

fn load_all(conn: &Connection, sources: &[String]) -> Result<Vec<Value>, AiError> {
    let filters = if sources.is_empty() {
        Vec::new()
    } else {
        vec![Filter {
            column: "source_type".to_string(),
            op: "in".to_string(),
            value: Some(json!(sources)),
        }]
    };

    service::list(
        conn,
        "memories",
        &ListQuery {
            filters,
            order_by: Some("id".to_string()),
            desc: true,
            ..Default::default()
        },
    )
    .map_err(AiError::internal)
}

/// 列表 / 关键词检索（供 command 和测试共用）
pub fn list_memories(
    conn: &Connection,
    source_type: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MemoryRow>, AiError> {
    let sources = source_type
        .filter(|value| !value.is_empty())
        .map(|value| vec![value])
        .unwrap_or_default();

    let rows = load_all(conn, &sources)?;
    let text = keyword.unwrap_or_default().trim().to_lowercase();
    let limit = limit.unwrap_or(200).clamp(1, 1000) as usize;

    let mut scored: Vec<MemoryRow> = rows
        .iter()
        .filter_map(|row| {
            let memory = row_to_memory(row, 0.0);
            if text.is_empty() {
                return Some(memory);
            }
            let related = score(&text, &memory.content);
            (related > 0.05).then_some(MemoryRow { score: related, ..memory })
        })
        .collect();

    if !text.is_empty() {
        scored.sort_by(|first, second| second.score.partial_cmp(&first.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    scored.truncate(limit);
    Ok(scored)
}

#[tauri::command]
pub fn memory_list(
    state: State<'_, AppState>,
    source_type: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MemoryRow>, AiError> {
    let conn = lock(&state)?;
    list_memories(&conn, source_type, keyword, limit)
}

/// 给其它模块用：给定查询返回相关记忆片段，可直接拼进 prompt。
pub fn search_memories(
    conn: &Connection,
    query: String,
    source_types: Option<Vec<String>>,
    limit: Option<i64>,
) -> Result<Vec<MemoryRow>, AiError> {
    let rows = load_all(conn, &source_types.unwrap_or_default())?;
    let limit = limit.unwrap_or(5).clamp(1, 50) as usize;

    let mut hits: Vec<MemoryRow> = rows
        .iter()
        .map(|row| row_to_memory(row, 0.0))
        .filter_map(|memory| {
            let related = score(&query, &memory.content);
            (related > 0.12).then_some(MemoryRow { score: related, ..memory })
        })
        .collect();

    hits.sort_by(|first, second| second.score.partial_cmp(&first.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    Ok(hits)
}

#[tauri::command]
pub fn memory_search(
    state: State<'_, AppState>,
    query: String,
    source_types: Option<Vec<String>>,
    limit: Option<i64>,
) -> Result<Vec<MemoryRow>, AiError> {
    let conn = lock(&state)?;
    search_memories(&conn, query, source_types, limit)
}

pub fn collect_stats(conn: &Connection) -> Result<MemoryStats, AiError> {
    let mut by_source = Vec::new();

    for source in SOURCES {
        let count = service::count(
            conn,
            "memories",
            &[Filter {
                column: "source_type".to_string(),
                op: "eq".to_string(),
                value: Some(json!(source)),
            }],
        )
        .map_err(AiError::internal)?;

        by_source.push(SourceCount {
            source_type: source.to_string(),
            label: source_label(source).to_string(),
            count,
        });
    }

    Ok(MemoryStats {
        total: service::count(conn, "memories", &[]).map_err(AiError::internal)?,
        by_source,
    })
}

#[tauri::command]
pub fn memory_stats(state: State<'_, AppState>) -> Result<MemoryStats, AiError> {
    let conn = lock(&state)?;
    collect_stats(&conn)
}

#[tauri::command]
pub fn memory_delete(state: State<'_, AppState>, id: i64) -> Result<bool, AiError> {
    let conn = lock(&state)?;
    service::delete(&conn, "memories", id).map_err(AiError::internal)
}

/// 重建索引：把所有模块的数据重新摘要一遍（已删除的数据对应的记忆会被清掉）
pub fn reindex_all(conn: &Connection) -> Result<MemoryStats, AiError> {
    conn.execute("DELETE FROM memories", [])
        .map_err(|error| AiError::internal(error.to_string()))?;

    for table in ["personas", "materials", "writings", "topics"] {
        let rows = service::list(
            conn,
            table,
            &ListQuery {
                order_by: Some("id".to_string()),
                ..Default::default()
            },
        )
        .map_err(AiError::internal)?;

        for row in &rows {
            let _ = sync_for_table(conn, table, row);
        }
    }

    collect_stats(conn)
}

#[tauri::command]
pub fn memory_reindex(state: State<'_, AppState>) -> Result<MemoryStats, AiError> {
    let conn = lock(&state)?;
    reindex_all(&conn)
}

/* -------------------------------- 导出导入 -------------------------------- */

pub fn export_json(conn: &Connection) -> Result<String, AiError> {
    let mut tables = Map::new();

    for table in EXPORT_TABLES {
        let rows = service::list(
            conn,
            table,
            &ListQuery {
                order_by: Some("id".to_string()),
                ..Default::default()
            },
        )
        .map_err(AiError::internal)?;
        tables.insert(table.to_string(), Value::Array(rows));
    }

    let exported_at: String = conn
        .query_row("SELECT datetime('now', 'localtime')", [], |row| row.get(0))
        .unwrap_or_default();

    serde_json::to_string_pretty(&json!({
        "version": 1,
        "exportedAt": exported_at,
        "tables": tables,
    }))
    .map_err(|error| AiError::internal(error.to_string()))
}

#[tauri::command]
pub fn memory_export(state: State<'_, AppState>) -> Result<String, AiError> {
    let conn = lock(&state)?;
    export_json(&conn)
}

/// 导入恢复：按 id 覆盖写入（replace 模式会先清空这些表）
pub fn import_json(conn: &Connection, payload: &str) -> Result<ImportResult, AiError> {
    let parsed: Value = serde_json::from_str(payload)
        .map_err(|error| AiError::config(format!("JSON 解析失败：{error}")))?;

    let tables = parsed
        .get("tables")
        .and_then(Value::as_object)
        .ok_or_else(|| AiError::config("这个 JSON 看起来不是本应用导出的备份"))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| AiError::internal(error.to_string()))?;

    let mut imported_tables = 0usize;
    let mut imported_rows = 0i64;

    for (table, rows) in tables {
        if !EXPORT_TABLES.contains(&table.as_str()) {
            continue;
        }
        let Some(spec) = service::table_spec(table) else {
            continue;
        };
        let Some(rows) = rows.as_array() else {
            continue;
        };

        tx.execute(&format!("DELETE FROM {}", spec.name), [])
            .map_err(|error| AiError::internal(error.to_string()))?;

        for row in rows {
            let columns: Vec<&str> = spec
                .columns
                .iter()
                .copied()
                .filter(|column| row.get(*column).is_some())
                .collect();
            if columns.is_empty() {
                continue;
            }

            let values: Vec<rusqlite::types::Value> = columns
                .iter()
                .map(|column| to_sql_value(row.get(*column)))
                .collect();

            let marks = vec!["?"; columns.len()].join(", ");
            tx.execute(
                &format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    spec.name,
                    columns.join(", "),
                    marks
                ),
                params_from_iter(values),
            )
            .map_err(|error| AiError::internal(error.to_string()))?;
            imported_rows += 1;
        }

        imported_tables += 1;
    }

    tx.commit()
        .map_err(|error| AiError::internal(error.to_string()))?;

    Ok(ImportResult {
        tables: imported_tables,
        rows: imported_rows,
    })
}

#[tauri::command]
pub fn memory_import(state: State<'_, AppState>, payload: String) -> Result<ImportResult, AiError> {
    let conn = lock(&state)?;
    import_json(&conn, &payload)
}

/// 记忆片段 → 给 AI 用的提示词块
#[tauri::command]
pub fn memory_prompt_block(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<String, AiError> {
    let conn = lock(&state)?;
    let hits = search_memories(&conn, query, None, limit)?;
    if hits.is_empty() {
        return Ok(String::new());
    }

    let mut block = String::from("【工作记忆】\n");
    for hit in hits {
        block.push_str(&format!("- [{}] {}\n", hit.source_label, hit.content));
    }
    Ok(block)
}
