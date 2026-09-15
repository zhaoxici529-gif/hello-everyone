use serde_json::json;

use super::{
    build_revise_prompt, build_revise_system_prompt, build_system_prompt, build_user_prompt,
    export_file_stem, html_to_markdown, html_to_text, parse_draft, parse_revision,
    suggestion_to_prompt, Genre, ImageSuggestion, ALL_GENRES,
};

#[test]
fn html_converts_to_plain_text_for_export() {
    let html = "<p>第一段</p><p>第二段<br>换行</p><p>&nbsp;</p><p>带 &amp; 符号 &lt;测试&gt;</p>";
    let text = html_to_text(html);

    assert!(text.contains("第一段"));
    assert!(text.contains("第二段\n换行"), "br 应该变成换行：{text}");
    assert!(text.contains("带 & 符号 <测试>"), "实体应该被还原：{text}");
    // 注意：实体解码后正文里合法的 < > 会保留，这里只检查没有残留标签
    assert!(!text.contains("<p>") && !text.contains("</p>"), "不应该残留标签：{text}");
    assert!(!text.contains("\n\n\n"), "多余空行应该被压掉");
}

#[test]
fn html_converts_to_markdown_for_export() {
    let html =
        "<h2>小标题</h2><p>正文里有<strong>重点</strong>和<em>语气</em></p><ul><li>要点一</li><li>要点二</li></ul>";
    let markdown = html_to_markdown(html);

    assert!(markdown.contains("## 小标题"), "标题应该转成 #：{markdown}");
    assert!(markdown.contains("**重点**"), "加粗应该转成 **：{markdown}");
    assert!(markdown.contains("*语气*"), "斜体应该转成 *：{markdown}");
    assert!(markdown.contains("- 要点一"), "列表应该转成 - ：{markdown}");
    assert!(!markdown.contains('<'));
}

#[test]
fn export_file_stem_is_safe() {
    // 中文标点保留，只有路径非法字符被替换
    assert_eq!(export_file_stem("失眠调理：3 个方法", 1), "失眠调理：3 个方法");
    assert_eq!(export_file_stem("a/b:c*d?e", 2), "a_b_c_d_e");
    assert_eq!(export_file_stem("   ", 7), "文案-7");
    assert!(export_file_stem(&"很长".repeat(80), 1).chars().count() <= 60);
}

fn sample() -> String {
    json!({
        "versions": [
            { "angle": "痛点切入", "title": "一到晚上就清醒？", "content": "正文一" },
            { "angle": "故事切入", "title": "我是怎么睡着的", "content": "正文二" },
            { "angle": "金句切入", "title": "先安顿身体，再谈情绪", "content": "正文三" }
        ],
        "配图建议": [
            { "位置": "封面", "关键词": "温暖的卧室，柔和灯光，宁静氛围", "风格": "治愈系", "比例": "3:4" },
            { "位置": "正文插图", "关键词": "手放在腹部做呼吸", "风格": "纪实感", "比例": "1:1" }
        ]
    })
    .to_string()
}

/* --------------------------------- 文体 --------------------------------- */

#[test]
fn genres_round_trip_and_have_style_guides() {
    assert_eq!(ALL_GENRES.len(), 4);

    for genre in ALL_GENRES {
        assert_eq!(Genre::from_id(genre.id()), Some(genre));
        assert!(!genre.label().is_empty());
        assert!(genre.style_guide().contains('【'), "{} 应该有写作规范", genre.label());
    }

    assert_eq!(Genre::from_id("  WeChat "), Some(Genre::Wechat));
    assert_eq!(Genre::from_id("unknown"), None);

    // 小红书的关键要求：标题、emoji、分段、话题标签
    let red = Genre::Xiaohongshu.style_guide();
    for keyword in ["标题", "emoji", "分段", "#"] {
        assert!(red.contains(keyword), "小红书规范里应该提到 {keyword}");
    }
}

/* --------------------------------- 提示词 --------------------------------- */

#[test]
fn system_prompt_embeds_the_persona_block() {
    let without = build_system_prompt(Genre::Xiaohongshu, None);
    assert!(!without.contains("必须遵守人物档案"));
    assert!(without.contains("心身同调"));

    let block = "【人物档案：小林（自我 IP）】\n- 口头禅：你先别急着讲道理";
    let with = build_system_prompt(Genre::Xiaohongshu, Some(block));
    assert!(with.contains("必须遵守人物档案"));
    assert!(with.contains("你先别急着讲道理"));

    // 空字符串等同于没选
    assert!(!build_system_prompt(Genre::Wechat, Some("   ")).contains("必须遵守人物档案"));
}

#[test]
fn user_prompt_carries_topic_extra_materials_and_json_contract() {
    let prompt = build_user_prompt(
        "失眠调理",
        Some("我自己试过的方法"),
        &["【素材：封面图（图片）】\n- 标签：治愈".to_string()],
    );

    assert!(prompt.contains("主题：失眠调理"));
    assert!(prompt.contains("我自己试过的方法"));
    assert!(prompt.contains("【素材：封面图（图片）】"));

    for key in ["\"versions\"", "\"配图建议\"", "\"位置\"", "\"关键词\"", "\"风格\"", "\"比例\""] {
        assert!(prompt.contains(key), "提示词里应该约定字段 {key}");
    }
    assert!(prompt.contains("3 个版本必须是明显不同"));
}

#[test]
fn revise_prompts_wrap_content_and_instruction() {
    let system = build_revise_system_prompt(Some(Genre::Moments), None);
    assert!(system.contains("朋友圈"));
    assert!(system.contains("直接输出修改后的完整文案"));

    let prompt = build_revise_prompt("原来的文案", "再口语化一点");
    assert!(prompt.contains("原来的文案"));
    assert!(prompt.contains("再口语化一点"));
}

/* --------------------------------- 解析 --------------------------------- */

#[test]
fn parses_three_versions_and_image_suggestions() {
    let payload = parse_draft(&sample()).unwrap();

    assert_eq!(payload.versions.len(), 3);
    assert_eq!(payload.versions[0].angle, "痛点切入");
    assert_eq!(payload.versions[0].title, "一到晚上就清醒？");

    assert_eq!(payload.image_keywords.len(), 2);
    let cover = &payload.image_keywords[0];
    assert_eq!(cover.position, "封面");
    assert_eq!(cover.keywords, "温暖的卧室，柔和灯光，宁静氛围");
    assert_eq!(cover.style, "治愈系");
    assert_eq!(cover.ratio, "3:4");
}

#[test]
fn parsing_tolerates_fences_and_missing_image_block() {
    let fenced = format!("```json\n{}\n```", sample());
    assert_eq!(parse_draft(&fenced).unwrap().versions.len(), 3);

    let chatty = format!("好的，我写了三版：\n{}", sample());
    assert_eq!(parse_draft(&chatty).unwrap().image_keywords.len(), 2);

    // 没有配图建议也能用，只是空列表
    let no_images = json!({ "versions": [{ "angle": "A", "title": "T", "content": "C" }] });
    let payload = parse_draft(&no_images.to_string()).unwrap();
    assert!(payload.image_keywords.is_empty());
}

#[test]
fn parsing_accepts_english_aliases_too() {
    let payload = parse_draft(
        &json!({
            "versions": [{ "angle": "A", "title": "T", "content": "C" }],
            "imageKeywords": [
                { "position": "封面", "keywords": "暖色房间", "style": "治愈", "ratio": "3:4" }
            ]
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(payload.image_keywords[0].position, "封面");
    assert_eq!(payload.image_keywords[0].keywords, "暖色房间");
}

#[test]
fn parsing_rejects_broken_payloads() {
    assert!(parse_draft("抱歉，我做不到").is_err());
    assert!(parse_draft("{ 非法 JSON }").is_err());
    assert!(parse_draft(&json!({ "versions": [] }).to_string()).is_err());
}

#[test]
fn revision_parsing_strips_fences_and_rejects_empty() {
    assert_eq!(parse_revision("改好了的文案").unwrap(), "改好了的文案");
    assert_eq!(
        parse_revision("```\n改好了的文案\n```").unwrap(),
        "改好了的文案"
    );
    assert!(parse_revision("   ").is_err());
}

#[test]
fn suggestion_becomes_a_reusable_image_prompt() {
    let suggestion = ImageSuggestion {
        position: "封面".to_string(),
        keywords: "温暖的卧室，柔和灯光".to_string(),
        style: "治愈系".to_string(),
        ratio: "3:4".to_string(),
    };

    let prompt = suggestion_to_prompt(&suggestion, "失眠调理");
    assert!(prompt.contains("温暖的卧室，柔和灯光"));
    assert!(prompt.contains("治愈系风格"));
    assert!(prompt.contains("主题：失眠调理"));
}

/// 验收标准 5 的数据库侧：保存的文案能被读回来，且文体、配图建议、对话历史都在。
#[test]
fn saving_a_writing_round_trips_genre_history_and_suggestions() {
    use crate::db::service::{self, ListQuery};
    use crate::db::{self as database, migrations};
    use rusqlite::Connection;

    let path = std::env::temp_dir().join(format!(
        "media-ai-workbench-writing-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }

    let conn = Connection::open(&path).unwrap();
    database::configure(&conn).unwrap();
    migrations::run(&conn).unwrap();

    let payload = parse_draft(&sample()).unwrap();
    let history = r#"[{"role":"user","content":"再口语化一点"},{"role":"assistant","content":"改好的版本"}]"#;

    let saved = service::insert(
        &conn,
        "writings",
        &json!({
            "title": payload.versions[0].title,
            "content": payload.versions[0].content,
            "status": "ready",
            "genre": "xiaohongshu",
            "persona_id": null,
            "images": serde_json::to_string(&payload.image_keywords).unwrap(),
            "history": history,
        })
        .as_object()
        .cloned()
        .unwrap(),
    )
    .unwrap();

    let id = saved["id"].as_i64().unwrap();
    assert_eq!(saved["genre"], json!("xiaohongshu"));
    assert_eq!(saved["status"], json!("ready"));

    // 读回来：文体、配图建议、历史都还在
    let row = service::get(&conn, "writings", id).unwrap().unwrap();
    let suggestions: Vec<ImageSuggestion> =
        serde_json::from_str(row["images"].as_str().unwrap()).unwrap();
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].ratio, "3:4");

    let turns: Vec<serde_json::Value> = serde_json::from_str(row["history"].as_str().unwrap()).unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0]["role"], json!("user"));

    // 「发布与经营」按更新时间倒序列出
    let listed = service::list(
        &conn,
        "writings",
        &ListQuery {
            order_by: Some("updated_at".to_string()),
            desc: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["title"], payload.versions[0].title);

    // 标记为已发布
    let published =
        service::update(&conn, "writings", id, &json!({ "status": "published" }).as_object().cloned().unwrap())
            .unwrap();
    assert_eq!(published["status"], json!("published"));
    assert!(published["updated_at"].is_string());

    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
