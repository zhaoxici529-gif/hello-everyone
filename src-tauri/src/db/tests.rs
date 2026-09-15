//! 数据库层的单元测试：验证迁移建表、通用 CRUD、白名单防护。
//! 跑法：`cargo test --offline`（在 src-tauri 目录下）

use rusqlite::Connection;
use serde_json::{json, Map, Value};

use super::service::{self, Filter, ListQuery};
use super::{migrations, service::TABLES};

const CORE_TABLES: [&str; 14] = [
    "settings",
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

fn memory_db() -> Connection {
    let conn = Connection::open_in_memory().expect("打开内存库失败");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("开启外键失败");
    migrations::run(&conn).expect("迁移失败");
    conn
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

#[test]
fn migrations_create_all_core_tables() {
    let conn = memory_db();

    let tables = service::describe(&conn).expect("读表结构失败");
    let names: Vec<&str> = tables.iter().map(|table| table.name.as_str()).collect();

    for expected in CORE_TABLES {
        assert!(names.contains(&expected), "缺少数据表: {expected}");
    }
    assert_eq!(tables.len(), CORE_TABLES.len(), "表数量与预期不符");
    assert_eq!(TABLES.len(), CORE_TABLES.len(), "白名单与建表清单不一致");

    // 每张表都至少要有列，且主键列存在
    for table in &tables {
        assert!(!table.columns.is_empty(), "表 {} 没有列", table.name);
        assert!(
            table.columns.iter().any(|column| column.primary_key),
            "表 {} 没有主键",
            table.name
        );
    }

    let latest = migrations::MIGRATIONS.last().unwrap().version;
    assert_eq!(migrations::schema_version(&conn).unwrap(), latest);
    // 幂等：再跑一次不应该重复执行
    assert!(migrations::run(&conn).expect("重复迁移失败").is_empty());
}

#[test]
fn todos_crud_round_trip() {
    let conn = memory_db();

    let created = service::insert(
        &conn,
        "todos",
        &object(json!({ "title": "写一条肩颈紧张的口播稿", "type": "task" })),
    )
    .expect("插入失败");

    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["status"], json!("pending"));
    assert_eq!(created["title"], json!("写一条肩颈紧张的口播稿"));

    let read = service::get(&conn, "todos", id)
        .expect("读取失败")
        .expect("读不到刚插入的记录");
    assert_eq!(read["type"], json!("task"));

    let updated = service::update(&conn, "todos", id, &object(json!({ "status": "done" })))
        .expect("更新失败");
    assert_eq!(updated["status"], json!("done"));

    let done_filter = vec![Filter {
        column: "status".into(),
        op: "eq".into(),
        value: Some(json!("done")),
    }];
    assert_eq!(service::list(&conn, "todos", &ListQuery::default()).unwrap().len(), 1);
    assert_eq!(service::count(&conn, "todos", &done_filter).unwrap(), 1);
    assert_eq!(
        service::list(
            &conn,
            "todos",
            &ListQuery {
                filters: done_filter,
                limit: Some(1),
                ..Default::default()
            }
        )
        .unwrap()
        .len(),
        1
    );

    assert!(service::delete(&conn, "todos", id).unwrap());
    assert!(service::get(&conn, "todos", id).unwrap().is_none());
}

#[test]
fn settings_upsert_and_json_columns() {
    let conn = memory_db();

    // settings 主键是 key，重写同 key 应该覆盖而不是报错
    let first = service::setting_set(&conn, "current_model", "gpt-4o-mini").unwrap();
    assert_eq!(first, "gpt-4o-mini");
    let second = service::setting_set(&conn, "current_model", "gpt-5").unwrap();
    assert_eq!(second, "gpt-5");
    assert_eq!(service::setting_get(&conn, "current_model").unwrap().unwrap(), "gpt-5");
    assert_eq!(service::setting_all(&conn).unwrap().len(), 1);
    assert!(service::setting_get(&conn, "不存在的键").unwrap().is_none());

    // JSON 列：对象会被序列化成文本存进 TEXT 列
    let persona = service::insert(
        &conn,
        "personas",
        &object(json!({
            "name": "我自己",
            "category": "self_ip",
            "fields": { "口吻": "温和、口语化", "禁忌": ["夸大疗效"] }
        })),
    )
    .expect("插入人物小传失败");

    let fields = persona["fields"].as_str().unwrap();
    assert!(fields.contains("温和、口语化"), "JSON 字段没有正确落库: {fields}");

    let id = persona["id"].as_i64().unwrap();
    let updated =
        service::update(&conn, "personas", id, &object(json!({ "name": "我自己（改）" })))
            .expect("更新失败");
    assert_eq!(updated["name"], json!("我自己（改）"));
    assert!(updated["updated_at"].is_string(), "updated_at 没有自动刷新");
}

#[test]
fn foreign_key_cascade_works() {
    let conn = memory_db();

    let blogger = service::insert(
        &conn,
        "bloggers",
        &object(json!({ "name": "对标账号 A", "platform": "视频号" })),
    )
    .unwrap();
    let blogger_id = blogger["id"].as_i64().unwrap();

    service::insert(
        &conn,
        "blogger_posts",
        &object(json!({ "blogger_id": blogger_id, "title": "一条爆款笔记" })),
    )
    .unwrap();
    assert_eq!(service::count(&conn, "blogger_posts", &[]).unwrap(), 1);

    // 删博主应级联删掉他的内容
    assert!(service::delete(&conn, "bloggers", blogger_id).unwrap());
    assert_eq!(service::count(&conn, "blogger_posts", &[]).unwrap(), 0);
}

#[test]
fn invalid_table_column_and_value_are_rejected() {
    let conn = memory_db();

    // 表名不在白名单
    assert!(service::list(&conn, "users", &ListQuery::default()).is_err());
    assert!(
        service::list(&conn, "todos; DROP TABLE todos", &ListQuery::default()).is_err(),
        "非法表名必须被拒绝"
    );

    // 排序列不在白名单
    assert!(service::list(
        &conn,
        "todos",
        &ListQuery {
            order_by: Some("id; DROP TABLE todos".into()),
            ..Default::default()
        }
    )
    .is_err());

    // 过滤列不在白名单
    assert!(service::count(
        &conn,
        "todos",
        &[Filter {
            column: "1=1 OR id".into(),
            op: "eq".into(),
            value: Some(json!(1)),
        }]
    )
    .is_err());

    // 写入不存在的列
    assert!(service::insert(&conn, "todos", &object(json!({ "evil": 1 }))).is_err());

    // CHECK 约束：type / status 只能是枚举值
    assert!(service::insert(
        &conn,
        "todos",
        &object(json!({ "title": "非法类型", "type": "unknown" }))
    )
    .is_err());

    // settings 不支持通用主键操作
    assert!(service::get(&conn, "settings", 1).is_err());
}
