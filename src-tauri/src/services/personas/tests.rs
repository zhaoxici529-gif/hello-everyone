use serde_json::json;

use super::{
    category_label, extraction_user_prompt, parse_extraction, PersonaFields, Purpose, CATEGORIES,
};

fn sample_json() -> String {
    json!({
        "age": "38 岁",
        "identity": "心理咨询师、两个孩子的妈妈",
        "experience": "在中学做了 12 年心理老师，后来专职做家庭情绪辅导。",
        "traits": ["温和", "直接", "爱举例"],
        "catchphrases": "你先别急着讲道理",
        "expressions": "常用「我举个例子啊」开头",
        "audience": "30-45 岁的妈妈",
        "painPoints": "一辅导作业就控制不住发火，事后又自责",
        "taboos": "不聊具体用药，不做诊断",
        "viewpoints": "先安顿自己的情绪，再谈孩子的成绩",
        "visualTone": "暖米色、浅木色",
        "visualStyle": "自然光、纪实感",
        "visualScene": "家里的书桌旁、咨询室",
        "sourceNote": "提取自公众号文章《先别急着讲道理》"
    })
    .to_string()
}

#[test]
fn category_labels_cover_all_categories() {
    assert_eq!(category_label("self_ip"), "自我 IP");
    assert_eq!(category_label("target_customer"), "目标客户");
    assert_eq!(category_label("case"), "案例人物");
    assert_eq!(category_label("unknown"), "未分类");
    assert_eq!(CATEGORIES.len(), 3);
}

#[test]
fn purpose_ids_round_trip() {
    for purpose in [Purpose::Writing, Purpose::Interview, Purpose::Image] {
        assert_eq!(Purpose::from_id(purpose.id()), Some(purpose));
        assert!(!purpose.label().is_empty());
    }
    assert_eq!(Purpose::from_id("  WRITING "), Some(Purpose::Writing));
    assert_eq!(Purpose::from_id("nope"), None);
}

#[test]
fn parses_a_plain_json_reply() {
    let fields = parse_extraction(&sample_json()).unwrap();

    assert_eq!(fields.age, "38 岁");
    assert_eq!(fields.traits, vec!["温和", "直接", "爱举例"]);
    assert_eq!(fields.catchphrases, "你先别急着讲道理");
    assert_eq!(fields.pain_points, "一辅导作业就控制不住发火，事后又自责");
    assert_eq!(fields.visual_style, "自然光、纪实感");
    assert!(!fields.is_blank());
}

#[test]
fn parses_fenced_and_chatty_replies() {
    // 模型常见的两种不规范输出：套了代码块 / 前后加了寒暄
    let fenced = format!("```json\n{}\n```", sample_json());
    assert_eq!(parse_extraction(&fenced).unwrap().age, "38 岁");

    let chatty = format!("好的，我整理好了：\n{}\n希望有帮助！", sample_json());
    assert_eq!(parse_extraction(&chatty).unwrap().identity, "心理咨询师、两个孩子的妈妈");
}

#[test]
fn missing_keys_fall_back_to_empty_defaults() {
    let fields = parse_extraction(&json!({ "age": "40 岁" }).to_string()).unwrap();
    assert_eq!(fields.age, "40 岁");
    assert!(fields.traits.is_empty());
    assert_eq!(fields.experience, "");
}

#[test]
fn rejects_replies_without_json() {
    assert!(parse_extraction("抱歉，我无法从这段文字提取人物特征。").is_err());
    assert!(parse_extraction("{ 这不是合法 JSON }").is_err());
    assert!(parse_extraction("").is_err());
}

#[test]
fn extraction_prompt_carries_source_and_name() {
    let prompt = extraction_user_prompt("这是我写了很久的一段文案，讲的是我自己怎么走出来的。", Some("小林"));

    assert!(prompt.contains("小林"));
    assert!(prompt.contains("这是我写了很久的一段文案"));
    for key in ["\"traits\"", "\"painPoints\"", "\"visualTone\"", "\"sourceNote\""] {
        assert!(prompt.contains(key), "提示词里应该说明字段 {key}");
    }

    // 不传名字也不应该出问题
    let anonymous = extraction_user_prompt("一段没有名字的文案内容，用来测试提示词拼装。", None);
    assert!(!anonymous.contains("这位人物叫"));
}

#[test]
fn writing_block_includes_voice_and_audience() {
    let fields = parse_extraction(&sample_json()).unwrap();
    let block = fields.to_prompt_block("小林", "self_ip", Purpose::Writing);

    assert!(block.starts_with("【人物档案：小林（自我 IP）】"));
    assert!(block.contains("性格特点：温和、直接、爱举例"));
    assert!(block.contains("口头禅：你先别急着讲道理"));
    assert!(block.contains("受众痛点："));
    assert!(block.contains("禁忌话题："));
    assert!(block.contains("代表观点："));
}

#[test]
fn image_block_only_keeps_visual_preferences() {
    let fields = parse_extraction(&sample_json()).unwrap();
    let block = fields.to_prompt_block("小林", "self_ip", Purpose::Image);

    assert!(block.contains("喜欢的色调：暖米色、浅木色"));
    assert!(block.contains("喜欢的风格：自然光、纪实感"));
    assert!(block.contains("常用场景：家里的书桌旁、咨询室"));
    // 图片用不到人生经历和禁忌
    assert!(!block.contains("核心经历"));
    assert!(!block.contains("禁忌话题"));
    assert!(!block.contains("口头禅"));
}

#[test]
fn empty_fields_produce_just_a_header() {
    let fields = PersonaFields::default();
    assert!(fields.is_blank());

    let block = fields.to_prompt_block("新人物", "case", Purpose::Writing);
    assert_eq!(block, "【人物档案：新人物（案例人物）】");
}

/// 人物档案在真实数据库上的完整来回：建 → 读 → 按分类筛 → 搜 → 改 → 删。
#[test]
fn persona_crud_round_trip_on_a_real_database() {
    use crate::db::service::{self, Filter, ListQuery};
    use crate::db::{self as database, migrations};
    use rusqlite::Connection;

    let path = std::env::temp_dir().join(format!(
        "media-ai-workbench-persona-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }

    let conn = Connection::open(&path).unwrap();
    database::configure(&conn).unwrap();
    migrations::run(&conn).unwrap();

    let fields = parse_extraction(&sample_json()).unwrap();
    let as_object = |value: serde_json::Value| value.as_object().cloned().unwrap_or_default();

    // 1. 创建
    let created = service::insert(
        &conn,
        "personas",
        &as_object(json!({
            "name": "小林",
            "category": "self_ip",
            "fields": serde_json::to_string(&fields).unwrap(),
        })),
    )
    .unwrap();

    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["name"], json!("小林"));
    assert_eq!(created["category"], json!("self_ip"));

    // 2. 读回：JSON 列能还原成同样的结构
    let row = service::get(&conn, "personas", id).unwrap().unwrap();
    assert_eq!(parse_extraction(row["fields"].as_str().unwrap()).unwrap(), fields);

    // 3. 按分类筛选
    service::insert(
        &conn,
        "personas",
        &as_object(json!({
            "name": "客户 A",
            "category": "target_customer",
            "fields": "{}",
        })),
    )
    .unwrap();

    let category_filter = |value: &str| {
        vec![Filter {
            column: "category".to_string(),
            op: "eq".to_string(),
            value: Some(json!(value)),
        }]
    };

    assert_eq!(
        service::count(&conn, "personas", &category_filter("self_ip")).unwrap(),
        1
    );
    assert_eq!(
        service::count(&conn, "personas", &category_filter("target_customer")).unwrap(),
        1
    );
    assert_eq!(
        service::count(&conn, "personas", &category_filter("case")).unwrap(),
        0
    );

    // 4. 关键词搜索
    let hits = service::list(
        &conn,
        "personas",
        &ListQuery {
            filters: vec![Filter {
                column: "name".to_string(),
                op: "like".to_string(),
                value: Some(json!("%小林%")),
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["name"], json!("小林"));

    // 5. 编辑
    let mut edited = fields.clone();
    edited.traits.push("爱较真".to_string());
    let updated = service::update(
        &conn,
        "personas",
        id,
        &as_object(json!({
            "name": "小林（改）",
            "fields": serde_json::to_string(&edited).unwrap(),
        })),
    )
    .unwrap();

    assert_eq!(updated["name"], json!("小林（改）"));
    assert!(updated["updated_at"].is_string(), "updated_at 应该自动刷新");
    assert_eq!(
        parse_extraction(updated["fields"].as_str().unwrap()).unwrap().traits.len(),
        fields.traits.len() + 1
    );

    // 6. 删除
    assert!(service::delete(&conn, "personas", id).unwrap());
    assert_eq!(service::count(&conn, "personas", &[]).unwrap(), 1);
    assert!(service::get(&conn, "personas", id).unwrap().is_none());

    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
