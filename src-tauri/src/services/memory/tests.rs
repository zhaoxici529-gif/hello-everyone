use rusqlite::Connection;
use serde_json::{json, Value};

use super::commands::{
    collect_stats, export_json, import_json, list_memories, reindex_all, remove_for_table,
    search_memories, sync_for_table,
};
use super::{plain_text, score, similar, source_label, source_of_table, summarize, SOURCES};
use crate::db::{self as database, migrations, service};

fn open_db(tag: &str) -> (Connection, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "media-ai-workbench-mem-{tag}-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    let conn = Connection::open(&path).unwrap();
    database::configure(&conn).unwrap();
    migrations::run(&conn).unwrap();
    (conn, path)
}

fn cleanup(path: &std::path::Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}

fn object(value: Value) -> serde_json::Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

#[test]
fn summary_covers_every_source_type() {
    assert_eq!(source_of_table("personas"), Some("persona"));
    assert_eq!(source_of_table("writings"), Some("writing"));
    assert_eq!(source_of_table("settings"), None);
    assert_eq!(SOURCES.len(), 4);
    for source in SOURCES {
        assert!(!source_label(source).is_empty());
    }

    let persona = json!({
        "id": 1,
        "name": "小林",
        "category": "self_ip",
        "fields": json!({
            "identity": "心理咨询师",
            "traits": ["温和", "直接"],
            "audience": "职场妈妈",
            "painPoints": "辅导作业就发火"
        })
        .to_string()
    });
    let text = summarize("persona", &persona);
    assert!(text.contains("人物小传《小林》"));
    assert!(text.contains("自我 IP"));
    assert!(text.contains("温和、直接"));
    assert!(text.contains("辅导作业就发火"));

    let writing = json!({
        "id": 2,
        "title": "总是睡不好",
        "genre": "xiaohongshu",
        "content": "<p>很多人以为自己<strong>睡不着</strong>是脑子太兴奋</p>"
    });
    let text = summarize("writing", &writing);
    assert!(text.contains("文案《总是睡不好》"));
    assert!(text.contains("小红书"));
    assert!(text.contains("睡不着"));
    assert!(!text.contains("<p>"), "正文里的标签应该被去掉");

    let material = json!({ "id": 3, "name": "封面图", "type": "image", "tags": "暖色" });
    assert!(summarize("material", &material).contains("素材《封面图》"));

    let topic = json!({ "id": 4, "title": "肩颈放松", "source": "热点", "tags": "痛点" });
    assert!(summarize("topic", &topic).contains("选题《肩颈放松》"));
}

#[test]
fn plain_text_strips_tags_and_truncates() {
    let text = plain_text("<h2>标题</h2><p>正文里 &amp; 有实体</p>", 100);
    assert!(text.contains("标题"));
    assert!(text.contains("正文里 & 有实体"));
    assert!(!text.contains('<'));

    let long = plain_text(&"很长的内容".repeat(100), 20);
    assert_eq!(long.chars().count(), 20);
}

#[test]
fn scoring_prefers_keyword_hits() {
    // 关键词完全命中
    let exact = score("失眠", "文案《失眠调理》正文讲了三个方法");
    // 部分字重叠（查询「失眠调理」，内容里只有「失眠」）
    let partial = score("失眠调理", "文案《失眠了怎么办》讲了呼吸训练");
    // 完全无关
    let unrelated = score("失眠调理", "素材《背景音乐》适合放在开头");

    assert!(exact > partial, "关键词命中的分数应该最高");
    assert!(partial > unrelated, "有字重叠的应该排在无关内容前面");
    assert_eq!(unrelated, 0.0);
    assert_eq!(similar("", "abc"), 0.0);
    assert!(similar("失眠调理", "失眠调理") > 0.99);
}

#[test]
fn data_changes_are_indexed_and_searchable() {
    let (conn, path) = open_db("index");

    let persona = service::insert(
        &conn,
        "personas",
        &object(json!({
            "name": "小林",
            "category": "self_ip",
            "fields": json!({ "identity": "心理咨询师", "painPoints": "辅导作业就发火" }).to_string()
        })),
    )
    .unwrap();
    assert!(sync_for_table(&conn, "personas", &persona).is_some());

    let writing = service::insert(
        &conn,
        "writings",
        &object(json!({
            "title": "总是睡不好",
            "content": "很多人以为睡不着是脑子太兴奋",
            "status": "draft",
            "genre": "xiaohongshu",
            "images": "[]",
            "history": "[]"
        })),
    )
    .unwrap();
    assert!(sync_for_table(&conn, "writings", &writing).is_some());

    let stats = collect_stats(&conn).unwrap();
    assert_eq!(stats.total, 2, "两条数据应该各有一条记忆");
    assert_eq!(stats.by_source.len(), 4);

    let hits = list_memories(&conn, None, Some("睡不着".to_string()), None).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].source_type, "writing");

    let only_persona = list_memories(&conn, Some("persona".to_string()), None, None).unwrap();
    assert_eq!(only_persona.len(), 1);
    assert_eq!(only_persona[0].source_label, "人物小传");

    let writing_id = writing["id"].as_i64().unwrap();
    let updated = service::update(
        &conn,
        "writings",
        writing_id,
        &object(json!({ "title": "总是睡不好？先看肩膀" })),
    )
    .unwrap();
    sync_for_table(&conn, "writings", &updated).unwrap();
    assert_eq!(collect_stats(&conn).unwrap().total, 2, "更新后仍然是两条，不重复");

    service::delete(&conn, "writings", writing_id).unwrap();
    remove_for_table(&conn, "writings", writing_id);
    assert_eq!(collect_stats(&conn).unwrap().total, 1);

    let fragments = search_memories(&conn, "心理咨询师".to_string(), None, Some(3)).unwrap();
    assert_eq!(fragments.len(), 1);
    assert!(fragments[0].content.contains("心理咨询师"));

    cleanup(&path);
}

#[test]
fn reindex_rebuilds_everything() {
    let (conn, path) = open_db("reindex");

    service::insert(
        &conn,
        "personas",
        &object(json!({ "name": "甲", "category": "self_ip", "fields": "{}" })),
    )
    .unwrap();
    service::insert(
        &conn,
        "topics",
        &object(json!({ "title": "乙选题", "source": "热点", "status": "pending", "tags": "" })),
    )
    .unwrap();

    assert_eq!(collect_stats(&conn).unwrap().total, 0, "还没有索引");

    let stats = reindex_all(&conn).unwrap();
    assert_eq!(stats.total, 2, "重建后两条数据都被索引");
    assert_eq!(reindex_all(&conn).unwrap().total, 2, "重复重建不会翻倍");

    cleanup(&path);
}

#[test]
fn export_and_import_round_trip() {
    let (source_conn, source_path) = open_db("export");

    let persona = service::insert(
        &source_conn,
        "personas",
        &object(json!({
            "name": "小林",
            "category": "self_ip",
            "fields": "{\"traits\":[\"温和\"]}"
        })),
    )
    .unwrap();
    let persona_id = persona["id"].as_i64().unwrap();

    let writing = service::insert(
        &source_conn,
        "writings",
        &object(json!({
            "title": "总是睡不好",
            "content": "正文",
            "status": "published",
            "genre": "wechat",
            "persona_id": persona_id,
            "images": "[]",
            "history": "[]"
        })),
    )
    .unwrap();
    let writing_id = writing["id"].as_i64().unwrap();

    sync_for_table(&source_conn, "personas", &persona).unwrap();
    sync_for_table(&source_conn, "writings", &writing).unwrap();

    let payload = export_json(&source_conn).unwrap();
    let parsed: Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed["version"], json!(1));
    assert!(parsed["exportedAt"].is_string());
    assert_eq!(parsed["tables"]["personas"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["tables"]["writings"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["tables"]["memories"].as_array().unwrap().len(), 2);

    let (target_conn, target_path) = open_db("import");
    let result = import_json(&target_conn, &payload).unwrap();

    // 导出里有多少行，导入就该写回多少行（含迁移自带的默认分类）
    let expected_rows: i64 = parsed["tables"]
        .as_object()
        .unwrap()
        .values()
        .map(|rows| rows.as_array().map(|items| items.len() as i64).unwrap_or(0))
        .sum();
    assert_eq!(result.rows, expected_rows, "导出的行数应该全部写回");
    assert!(result.rows >= 3, "至少有 人物 + 文案 + 记忆");

    let restored = service::get(&target_conn, "personas", persona_id)
        .unwrap()
        .unwrap();
    assert_eq!(restored["name"], json!("小林"), "导入后 id 和字段都要保留");

    let restored_writing = service::get(&target_conn, "writings", writing_id)
        .unwrap()
        .unwrap();
    assert_eq!(restored_writing["persona_id"], json!(persona_id), "关联关系要保留");
    assert_eq!(collect_stats(&target_conn).unwrap().total, 2, "记忆也恢复了");

    assert!(import_json(&target_conn, "不是 JSON").is_err());
    assert!(import_json(&target_conn, "{\"foo\":1}").is_err());

    cleanup(&source_path);
    cleanup(&target_path);
}
