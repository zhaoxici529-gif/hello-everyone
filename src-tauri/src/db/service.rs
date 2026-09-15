//! 统一的数据库访问层。
//!
//! 设计要点：
//! 1. 表名 / 列名一律走 `TABLES` 白名单校验，SQL 只拼接校验过的标识符；
//! 2. 所有值都用占位符绑定，不做字符串拼接；
//! 3. 行数据统一转成 `serde_json::Value`，前端拿到的就是对象，不用为每张表写映射代码。

use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Statement};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/* --------------------------------- 表定义 --------------------------------- */

pub struct TableSpec {
    pub name: &'static str,
    /// 主键列名。`settings` 用 `key`，其余都是自增 `id`。
    pub primary_key: &'static str,
    pub columns: &'static [&'static str],
    /// 更新时是否自动刷新 updated_at
    pub updated_at: bool,
}

/// 允许被通用 CRUD 操作的表。不在这个列表里的表名一律拒绝。
pub const TABLES: &[TableSpec] = &[
    TableSpec {
        name: "settings",
        primary_key: "key",
        columns: &["key", "value", "updated_at"],
        updated_at: true,
    },
    TableSpec {
        name: "personas",
        primary_key: "id",
        columns: &["id", "name", "category", "fields", "created_at", "updated_at"],
        updated_at: true,
    },
    TableSpec {
        name: "materials",
        primary_key: "id",
        columns: &[
            "id",
            "type",
            "name",
            "file_path",
            "tags",
            "description",
            "created_at",
            "category_id",
            "thumb_path",
            "size_bytes",
            "source_path",
            "ai_description_at",
        ],
        updated_at: false,
    },
    TableSpec {
        name: "material_categories",
        primary_key: "id",
        columns: &["id", "name", "created_at"],
        updated_at: false,
    },
    TableSpec {
        name: "writings",
        primary_key: "id",
        columns: &[
            "id", "title", "content", "status", "persona_id", "topic_id", "images", "created_at",
            "updated_at", "genre", "history", "published_at", "version_history", "deal_count",
            "lead_count", "private_count", "consult_count", "stats_updated_at",
        ],
        updated_at: true,
    },
    TableSpec {
        name: "topics",
        primary_key: "id",
        columns: &[
            "id", "title", "source", "status", "tags", "note", "created_at", "updated_at",
        ],
        updated_at: true,
    },
    TableSpec {
        name: "bloggers",
        primary_key: "id",
        columns: &["id", "name", "platform", "homepage_url", "avatar", "tags", "created_at"],
        updated_at: false,
    },
    TableSpec {
        name: "blogger_posts",
        primary_key: "id",
        columns: &[
            "id",
            "blogger_id",
            "title",
            "url",
            "cover",
            "likes",
            "collects",
            "comments",
            "shares",
            "publish_date",
            "is_abnormal",
            "abnormal_reason",
            "status",
            "created_at",
        ],
        updated_at: false,
    },
    TableSpec {
        name: "todos",
        primary_key: "id",
        columns: &["id", "title", "type", "status", "plan_date", "done_at", "created_at"],
        updated_at: false,
    },
    TableSpec {
        name: "notes",
        primary_key: "id",
        columns: &["id", "content", "converted_to", "created_at"],
        updated_at: false,
    },
    TableSpec {
        name: "plans",
        primary_key: "id",
        columns: &["id", "period", "goal", "breakdown", "created_at", "updated_at"],
        updated_at: true,
    },
    TableSpec {
        name: "pomodoro",
        primary_key: "id",
        columns: &["id", "task", "duration", "started_at", "finished_at"],
        updated_at: false,
    },
    TableSpec {
        name: "memories",
        primary_key: "id",
        columns: &["id", "source_type", "source_id", "content", "embedding", "created_at"],
        updated_at: false,
    },
    TableSpec {
        name: "ai_usage",
        primary_key: "id",
        columns: &["id", "model", "type", "tokens", "created_at", "is_free_trial"],
        updated_at: false,
    },
];

pub fn table_spec(name: &str) -> Option<&'static TableSpec> {
    TABLES.iter().find(|spec| spec.name == name)
}

fn ensure_table(name: &str) -> Result<&'static TableSpec, String> {
    table_spec(name).ok_or_else(|| format!("未知数据表: {name}"))
}

fn ensure_column(spec: &TableSpec, column: &str) -> Result<(), String> {
    if spec.columns.contains(&column) {
        Ok(())
    } else {
        Err(format!("表 {} 没有列 {}", spec.name, column))
    }
}

/// 需要走主键定位的表（settings 用 setting_* 命令单独处理）
fn ensure_id_table(name: &str) -> Result<&'static TableSpec, String> {
    let spec = ensure_table(name)?;
    if spec.primary_key == "id" {
        Ok(spec)
    } else {
        Err(format!("表 {} 不支持通用主键操作", spec.name))
    }
}

/* --------------------------------- 查询参数 --------------------------------- */

fn default_op() -> String {
    "eq".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub column: String,
    #[serde(default = "default_op")]
    pub op: String,
    pub value: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ListQuery {
    pub filters: Vec<Filter>,
    pub order_by: Option<String>,
    /// true = 倒序（默认按主键倒序更适合列表）
    pub desc: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/* --------------------------------- 返回值 --------------------------------- */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub primary_key: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
    pub rows: i64,
    pub columns: Vec<ColumnInfo>,
}

/* --------------------------------- 内部工具 --------------------------------- */

fn to_err(error: rusqlite::Error) -> String {
    error.to_string()
}

fn column_names(stmt: &Statement<'_>) -> Vec<String> {
    stmt.column_names().into_iter().map(String::from).collect()
}

/// JSON 值 -> SQLite 绑定值。对象/数组统一序列化成 JSON 文本，方便存 JSON 列。
pub(crate) fn to_sql_value(value: Option<&Value>) -> SqlValue {
    match value {
        None | Some(Value::Null) => SqlValue::Null,
        Some(Value::Bool(flag)) => SqlValue::Integer(i64::from(*flag)),
        Some(Value::Number(number)) => match number.as_i64() {
            Some(int) => SqlValue::Integer(int),
            None => SqlValue::Real(number.as_f64().unwrap_or_default()),
        },
        Some(Value::String(text)) => SqlValue::Text(text.clone()),
        Some(other) => SqlValue::Text(other.to_string()),
    }
}

/// 一行 -> JSON 对象
fn row_to_value(row: &rusqlite::Row<'_>, names: &[String]) -> rusqlite::Result<Value> {
    let mut map = Map::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        let value = match row.get_ref(index)? {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(number) => Value::from(number),
            ValueRef::Real(number) => Value::from(number),
            ValueRef::Text(text) => Value::from(String::from_utf8_lossy(text).into_owned()),
            ValueRef::Blob(blob) => Value::from(format!("<blob {} bytes>", blob.len())),
        };
        map.insert(name.clone(), value);
    }
    Ok(Value::Object(map))
}

fn build_where(spec: &TableSpec, filters: &[Filter]) -> Result<(String, Vec<SqlValue>), String> {
    if filters.is_empty() {
        return Ok((String::new(), Vec::new()));
    }

    let mut clauses = Vec::new();
    let mut params = Vec::new();

    for filter in filters {
        ensure_column(spec, &filter.column)?;
        let column = filter.column.as_str();

        match filter.op.as_str() {
            "eq" => {
                clauses.push(format!("{column} = ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "ne" => {
                clauses.push(format!("{column} <> ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "gt" => {
                clauses.push(format!("{column} > ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "gte" => {
                clauses.push(format!("{column} >= ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "lt" => {
                clauses.push(format!("{column} < ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "lte" => {
                clauses.push(format!("{column} <= ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "like" => {
                clauses.push(format!("{column} LIKE ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "notLike" => {
                clauses.push(format!("{column} NOT LIKE ?"));
                params.push(to_sql_value(filter.value.as_ref()));
            }
            "isNull" => clauses.push(format!("{column} IS NULL")),
            "notNull" => clauses.push(format!("{column} IS NOT NULL")),
            "in" => {
                let items = filter
                    .value
                    .as_ref()
                    .and_then(Value::as_array)
                    .ok_or_else(|| "in 操作需要一个数组值".to_string())?;

                if items.is_empty() {
                    // 空集合恒不成立
                    clauses.push("1 = 0".to_string());
                } else {
                    let marks = vec!["?"; items.len()].join(", ");
                    clauses.push(format!("{column} IN ({marks})"));
                    for item in items {
                        params.push(to_sql_value(Some(item)));
                    }
                }
            }
            other => return Err(format!("不支持的过滤操作: {other}")),
        }
    }

    Ok((format!(" WHERE {}", clauses.join(" AND ")), params))
}

/* --------------------------------- 通用 CRUD -------------------------------- */

pub fn list(conn: &Connection, table: &str, query: &ListQuery) -> Result<Vec<Value>, String> {
    let spec = ensure_table(table)?;
    let (where_sql, mut params) = build_where(spec, &query.filters)?;

    let mut sql = format!(
        "SELECT {} FROM {}{}",
        spec.columns.join(", "),
        spec.name,
        where_sql
    );

    let order_column = match &query.order_by {
        Some(column) => {
            ensure_column(spec, column)?;
            Some(column.clone())
        }
        None => spec
            .columns
            .contains(&spec.primary_key)
            .then(|| spec.primary_key.to_string()),
    };

    if let Some(column) = order_column {
        sql.push_str(&format!(
            " ORDER BY {column} {}",
            if query.desc { "DESC" } else { "ASC" }
        ));
    }

    if let Some(limit) = query.limit {
        sql.push_str(" LIMIT ?");
        params.push(SqlValue::Integer(limit));
    } else if query.offset.is_some() {
        sql.push_str(" LIMIT -1");
    }
    if let Some(offset) = query.offset {
        sql.push_str(" OFFSET ?");
        params.push(SqlValue::Integer(offset));
    }

    let mut stmt = conn.prepare(&sql).map_err(to_err)?;
    let names = column_names(&stmt);
    let rows = stmt
        .query_map(params_from_iter(params), |row| row_to_value(row, &names))
        .map_err(to_err)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(to_err)?);
    }
    Ok(result)
}

pub fn get(conn: &Connection, table: &str, id: i64) -> Result<Option<Value>, String> {
    let spec = ensure_id_table(table)?;
    let sql = format!(
        "SELECT {} FROM {} WHERE id = ?1",
        spec.columns.join(", "),
        spec.name
    );

    let mut stmt = conn.prepare(&sql).map_err(to_err)?;
    let names = column_names(&stmt);
    let mut rows = stmt
        .query_map([id], |row| row_to_value(row, &names))
        .map_err(to_err)?;

    match rows.next() {
        Some(row) => Ok(Some(row.map_err(to_err)?)),
        None => Ok(None),
    }
}

pub fn insert(conn: &Connection, table: &str, data: &Map<String, Value>) -> Result<Value, String> {
    let spec = ensure_table(table)?;

    let mut columns = Vec::new();
    let mut values = Vec::new();
    for (column, value) in data {
        // id 交给 AUTOINCREMENT，不接受外部指定
        if column == spec.primary_key {
            continue;
        }
        ensure_column(spec, column)?;
        columns.push(column.as_str());
        values.push(to_sql_value(Some(value)));
    }

    if columns.is_empty() {
        return Err("插入数据不能为空".to_string());
    }

    let marks = vec!["?"; columns.len()].join(", ");
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        spec.name,
        columns.join(", "),
        marks
    );
    conn.execute(&sql, params_from_iter(values))
        .map_err(to_err)?;

    if spec.primary_key == "id" {
        get(conn, table, conn.last_insert_rowid())?.ok_or_else(|| "插入后读取失败".to_string())
    } else {
        // settings：按 key 读回
        let key = data
            .get(spec.primary_key)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{} 必须提供 {}", spec.name, spec.primary_key))?;

        let sql = format!(
            "SELECT {} FROM {} WHERE {} = ?1",
            spec.columns.join(", "),
            spec.name,
            spec.primary_key
        );
        let mut stmt = conn.prepare(&sql).map_err(to_err)?;
        let names = column_names(&stmt);
        let mut rows = stmt
            .query_map([key], |row| row_to_value(row, &names))
            .map_err(to_err)?;
        match rows.next() {
            Some(row) => row.map_err(to_err),
            None => Err("插入后读取失败".to_string()),
        }
    }
}

pub fn update(
    conn: &Connection,
    table: &str,
    id: i64,
    data: &Map<String, Value>,
) -> Result<Value, String> {
    let spec = ensure_id_table(table)?;

    let mut assignments = Vec::new();
    let mut values = Vec::new();
    for (column, value) in data {
        if column == "id" {
            continue;
        }
        ensure_column(spec, column)?;
        assignments.push(format!("{column} = ?"));
        values.push(to_sql_value(Some(value)));
    }

    if spec.updated_at {
        assignments.push("updated_at = datetime('now', 'localtime')".to_string());
    }
    if assignments.is_empty() {
        return Err("没有需要更新的字段".to_string());
    }

    values.push(SqlValue::Integer(id));
    let sql = format!(
        "UPDATE {} SET {} WHERE id = ?",
        spec.name,
        assignments.join(", ")
    );
    let affected = conn
        .execute(&sql, params_from_iter(values))
        .map_err(to_err)?;

    if affected == 0 {
        return Err(format!("没有找到 id = {id} 的记录"));
    }
    get(conn, table, id)?.ok_or_else(|| "更新后读取失败".to_string())
}

pub fn delete(conn: &Connection, table: &str, id: i64) -> Result<bool, String> {
    let spec = ensure_id_table(table)?;
    let sql = format!("DELETE FROM {} WHERE id = ?1", spec.name);
    let affected = conn.execute(&sql, [id]).map_err(to_err)?;
    Ok(affected > 0)
}

pub fn count(conn: &Connection, table: &str, filters: &[Filter]) -> Result<i64, String> {
    let spec = ensure_table(table)?;
    let (where_sql, params) = build_where(spec, filters)?;
    let sql = format!("SELECT COUNT(*) FROM {}{}", spec.name, where_sql);

    conn.query_row(&sql, params_from_iter(params), |row| row.get(0))
        .map_err(to_err)
}

/* ---------------------------------- 设置 ---------------------------------- */

/// 读一个配置项
pub fn setting_get(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| {
        row.get(0)
    })
    .optional()
    .map_err(to_err)
}

/// 写一个配置项（存在则覆盖）。
/// API Key 这类敏感值应该在调用前加密，库里只存密文。
pub fn setting_set(conn: &Connection, key: &str, value: &str) -> Result<String, String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE
         SET value = excluded.value, updated_at = datetime('now', 'localtime')",
        params![key, value],
    )
    .map_err(to_err)?;

    setting_get(conn, key)?.ok_or_else(|| "写入后读取失败".to_string())
}

/// 列出所有配置项（不含敏感值也可由调用方过滤）
pub fn setting_all(conn: &Connection) -> Result<Vec<Value>, String> {
    list(conn, "settings", &ListQuery::default())
}

/// 删除一个配置项
pub fn setting_delete(conn: &Connection, key: &str) -> Result<bool, String> {
    let affected = conn
        .execute("DELETE FROM settings WHERE key = ?1", params![key])
        .map_err(to_err)?;
    Ok(affected > 0)
}

/* -------------------------------- 结构自省 -------------------------------- */

/// 列出所有业务表和它们的列定义 + 当前行数，用于自检和设置页展示。
pub fn describe(conn: &Connection) -> Result<Vec<TableInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
             ORDER BY name",
        )
        .map_err(to_err)?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(to_err)?
        .collect::<rusqlite::Result<Vec<String>>>()
        .map_err(to_err)?;
    drop(stmt);

    let mut tables = Vec::new();
    for name in names {
        if table_spec(&name).is_none() {
            continue;
        }

        let mut info_stmt = conn
            .prepare(
                "SELECT name, type, \"notnull\", pk, dflt_value
                 FROM pragma_table_info(?1)",
            )
            .map_err(to_err)?;

        let columns = info_stmt
            .query_map(params![name], |row| {
                Ok(ColumnInfo {
                    name: row.get(0)?,
                    data_type: row.get(1)?,
                    not_null: row.get::<_, i64>(2)? != 0,
                    primary_key: row.get::<_, i64>(3)? != 0,
                    default_value: row.get(4)?,
                })
            })
            .map_err(to_err)?
            .collect::<rusqlite::Result<Vec<ColumnInfo>>>()
            .map_err(to_err)?;
        drop(info_stmt);

        let rows = count(conn, &name, &[])?;
        tables.push(TableInfo {
            name,
            rows,
            columns,
        });
    }

    Ok(tables)
}
