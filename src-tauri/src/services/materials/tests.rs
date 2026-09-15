use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgb};
use rusqlite::Connection;
use serde_json::{json, Value};

use super::commands::{import_one, remove_material_files};
use super::{
    ensure_inside_library, join_tags, library_root, make_thumbnail, sanitize_file_name, split_tags,
    thumb_path_for, unique_file_name, MaterialRecord, MaterialType, Purpose, THUMB_MAX_EDGE,
};
use crate::db::service;
use crate::db::{self as database, migrations};

fn workspace(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "media-ai-workbench-mat-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn open_db(path: &Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    database::configure(&conn).unwrap();
    migrations::run(&conn).unwrap();
    conn
}

/// 写一张真实的 PNG，用来验证缩略图
fn write_png(path: &Path, width: u32, height: u32) {
    let buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(width, height, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 120]));
    buffer.save(path).unwrap();
}

fn object(value: Value) -> serde_json::Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

/* --------------------------------- 类型判定 --------------------------------- */

#[test]
fn supported_extensions_map_to_the_right_type() {
    let cases = [
        ("封面.PNG", MaterialType::Image),
        ("照片.jpg", MaterialType::Image),
        ("插画.webp", MaterialType::Image),
        ("动图.gif", MaterialType::Image),
        ("口播.mp4", MaterialType::Video),
        ("切片.MOV", MaterialType::Video),
        ("背景乐.mp3", MaterialType::Audio),
        ("录音.wav", MaterialType::Audio),
        ("脚本.txt", MaterialType::Text),
        ("笔记.md", MaterialType::Text),
    ];

    for (name, expected) in cases {
        assert_eq!(
            MaterialType::from_path(Path::new(name)),
            Some(expected),
            "{name} 的类型判断不对"
        );
    }

    for name in ["安装包.exe", "文档.pdf", "压缩包.zip", "没有扩展名"] {
        assert_eq!(
            MaterialType::from_path(Path::new(name)),
            None,
            "{name} 不该被接受"
        );
    }

    assert!(MaterialType::Image.is_visual());
    assert!(MaterialType::Video.is_visual());
    assert!(!MaterialType::Audio.is_visual());
    assert!(!MaterialType::Text.is_visual());
}

/* ---------------------------------- 文件名 ---------------------------------- */

#[test]
fn file_names_are_sanitized_and_unique() {
    assert_eq!(sanitize_file_name("封面:图.png"), "封面_图.png");
    assert_eq!(sanitize_file_name("a/b\\c*d?.jpg"), "a_b_c_d_.jpg");
    assert_eq!(sanitize_file_name("   "), "material");
    assert_eq!(sanitize_file_name("..."), "material");

    let unique = unique_file_name("封面.png", 1757851234567);
    assert!(unique.starts_with("1757851234567_"));
    assert!(unique.ends_with("封面.png"));

    let long = unique_file_name(&format!("{}.png", "很长".repeat(80)), 1);
    assert!(long.chars().count() <= 135);
}

#[test]
fn tags_round_trip_through_a_comma_string() {
    let tags = vec![
        "封面".to_string(),
        "暖色".to_string(),
        "呼吸训练".to_string(),
    ];
    let joined = join_tags(&tags);
    assert_eq!(joined, "封面,暖色,呼吸训练");
    assert_eq!(split_tags(&joined), tags);

    assert!(split_tags("").is_empty());
    assert_eq!(split_tags("封面,,  ,暖色"), vec!["封面", "暖色"]);
    assert_eq!(join_tags(&["".to_string(), "  ".to_string()]), "");
}

/* ---------------------------------- 缩略图 ---------------------------------- */

#[test]
fn image_thumbnail_is_generated_and_smaller() {
    let dir = workspace("thumb");
    let source = dir.join("大图.png");
    write_png(&source, 1600, 1200);

    let target = thumb_path_for(&source);
    assert_eq!(target.file_name().unwrap().to_string_lossy(), "大图.thumb.jpg");

    make_thumbnail(&source, &target, THUMB_MAX_EDGE).expect("缩略图生成失败");

    assert!(target.exists(), "缩略图文件没有生成");
    assert!(
        fs::metadata(&target).unwrap().len() < fs::metadata(&source).unwrap().len(),
        "缩略图应该比原图小"
    );

    let thumb = image::open(&target).unwrap();
    assert!(thumb.width() <= THUMB_MAX_EDGE && thumb.height() <= THUMB_MAX_EDGE);

    assert!(make_thumbnail(&dir.join("不存在.png"), &target, THUMB_MAX_EDGE).is_err());

    let _ = fs::remove_dir_all(&dir);
}

/* --------------------------------- 安全边界 --------------------------------- */

#[test]
fn only_files_inside_the_library_can_be_touched() {
    let dir = workspace("scope");
    let root = library_root(&dir);
    fs::create_dir_all(root.join("2026-09")).unwrap();

    let inside = root.join("2026-09").join("a.png");
    fs::write(&inside, b"x").unwrap();
    assert!(ensure_inside_library(&root, &inside).is_ok());

    let outside = dir.join("b.png");
    fs::write(&outside, b"x").unwrap();
    assert!(
        ensure_inside_library(&root, &outside).is_err(),
        "素材库以外的文件必须拒绝"
    );

    let _ = fs::remove_dir_all(&dir);
}

/* ------------------------------- 完整导入流程 ------------------------------- */

#[test]
fn importing_a_real_image_creates_file_thumbnail_and_row() {
    let dir = workspace("import");
    let data_dir = dir.join("data");
    let root = library_root(&data_dir);
    fs::create_dir_all(&root).unwrap();

    let conn = open_db(&dir.join("workbench.db"));

    let source = dir.join("封面图.png");
    write_png(&source, 1200, 900);

    let category = service::insert(
        &conn,
        "material_categories",
        &object(json!({ "name": "测试分类" })),
    )
    .unwrap();
    let category_id = category["id"].as_i64().unwrap();

    let row = import_one(&conn, &root, &source, Some(category_id)).expect("导入失败");
    let id = row["id"].as_i64().unwrap();

    assert_eq!(row["type"], json!("image"));
    assert_eq!(row["name"], json!("封面图"), "名称应该去掉扩展名");
    assert_eq!(row["category_id"], json!(category_id));
    assert!(row["size_bytes"].as_i64().unwrap() > 0);

    let file = PathBuf::from(row["file_path"].as_str().unwrap());
    assert!(file.exists(), "导入后文件应该存在于素材库");
    assert!(file.starts_with(&root), "文件必须落在素材库目录内");
    assert!(file.to_string_lossy().contains("2026"), "应该按年月分目录");

    let thumb = PathBuf::from(row["thumb_path"].as_str().unwrap());
    assert!(thumb.exists(), "图片应该自动生成缩略图");

    // 打标签 + 描述
    let updated = service::update(
        &conn,
        "materials",
        id,
        &object(json!({
            "tags": join_tags(&["封面".to_string(), "暖色".to_string()]),
            "description": "清晨窗边的封面图"
        })),
    )
    .unwrap();

    let record = MaterialRecord::from_row(&updated).unwrap();
    assert_eq!(record.tags, vec!["封面", "暖色"]);
    assert_eq!(record.kind, MaterialType::Image);

    // 按标签筛选（前端用 like 做，这里确认 SQL 层也支持）
    let hits = service::list(
        &conn,
        "materials",
        &service::ListQuery {
            filters: vec![service::Filter {
                column: "tags".to_string(),
                op: "like".to_string(),
                value: Some(json!("%暖色%")),
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);

    // 删除：数据库行和文件都要没
    let removed = remove_material_files(&root, &updated);
    assert_eq!(removed.len(), 2, "文件和缩略图都应该被删掉");
    assert!(!file.exists());
    assert!(!thumb.exists());

    assert!(service::delete(&conn, "materials", id).unwrap());
    assert_eq!(service::count(&conn, "materials", &[]).unwrap(), 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unsupported_and_missing_files_are_skipped_with_a_reason() {
    let dir = workspace("skip");
    let data_dir = dir.join("data");
    let root = library_root(&data_dir);
    fs::create_dir_all(&root).unwrap();

    let conn = open_db(&dir.join("workbench.db"));

    let bad = dir.join("安装包.exe");
    fs::write(&bad, b"x").unwrap();
    let reason = import_one(&conn, &root, &bad, None).unwrap_err();
    assert!(reason.contains("不支持的文件类型"), "实际原因：{reason}");

    let missing = dir.join("不存在.png");
    let reason = import_one(&conn, &root, &missing, None).unwrap_err();
    assert!(reason.contains("文件不存在"), "实际原因：{reason}");

    assert_eq!(service::count(&conn, "materials", &[]).unwrap(), 0);
    let _ = fs::remove_dir_all(&dir);
}

/* ------------------------------ 供别的模块引用的提示词 ------------------------------ */

#[test]
fn prompt_block_changes_by_purpose() {
    let record = MaterialRecord {
        id: 1,
        name: "封面图".to_string(),
        kind: MaterialType::Image,
        file_path: "C:/素材库/2026-09/1_封面图.png".to_string(),
        thumb_path: None,
        tags: vec!["封面".to_string(), "暖色".to_string()],
        description: Some("清晨窗边的暖色调封面".to_string()),
    };

    let writing = record.to_prompt_block(Purpose::Writing);
    assert!(writing.contains("【素材：封面图（图片）】"));
    assert!(writing.contains("标签：封面、暖色"));
    assert!(writing.contains("描述：清晨窗边的暖色调封面"));
    assert!(writing.contains("文件："));
    assert!(writing.contains("参考或引用"));

    assert!(record.to_prompt_block(Purpose::Image).contains("作为画面参考"));
    assert!(record.to_prompt_block(Purpose::Video).contains("作为剪辑素材"));

    assert_eq!(Purpose::from_id(" writing "), Some(Purpose::Writing));
    assert_eq!(Purpose::from_id("video"), Some(Purpose::Video));
    assert_eq!(Purpose::from_id("nope"), None);
}

#[test]
fn record_from_row_survives_missing_optional_fields() {
    let row = object(json!({
        "id": 7,
        "name": "脚本",
        "type": "text",
        "file_path": "C:/素材库/2026-09/7_脚本.txt",
        "tags": ""
    }));

    let record = MaterialRecord::from_row(&Value::Object(row)).unwrap();

    assert_eq!(record.kind, MaterialType::Text);
    assert!(record.tags.is_empty());
    assert!(record.thumb_path.is_none());
    assert_eq!(record.name, "脚本");
}
