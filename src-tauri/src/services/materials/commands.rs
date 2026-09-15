//! 素材管理的 Tauri command。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::db::service;
use crate::services::ai::commands::run_text;
use crate::services::ai::{AiError, ChatRequest};
use crate::services::materials::{
    ensure_inside_library, join_tags, library_root, make_thumbnail, month_folder, thumb_path_for,
    unique_file_name, MaterialRecord, MaterialType, Purpose, THUMB_MAX_EDGE,
};
use crate::AppState;

/// 送进模型前给图片留的字节上限，超过就不附图（只做文字描述）
const MAX_VISION_BYTES: u64 = 4 * 1024 * 1024;
/// 文本素材读取上限
const MAX_TEXT_CHARS: usize = 20_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkip {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOutcome {
    pub imported: Vec<Value>,
    pub skipped: Vec<ImportSkip>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub directory: String,
    pub count: i64,
    pub total_bytes: i64,
    pub categories: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBlock {
    pub purpose: String,
    pub purpose_label: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialDescription {
    pub description: String,
    pub tags: Vec<String>,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
    pub used_trial_key: bool,
    pub remaining_free: i64,
    /// 是否附带图片给模型看
    pub sent_image: bool,
    pub material: Value,
}

/* --------------------------------- 小工具 --------------------------------- */

fn stamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn year_month(conn: &Connection) -> String {
    conn.query_row("SELECT strftime('%Y-%m', 'now', 'localtime')", [], |row| {
        row.get(0)
    })
    .unwrap_or_else(|_| "unknown".to_string())
}

fn data_dir_of(state: &State<'_, AppState>) -> Result<PathBuf, AiError> {
    state
        .path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| AiError::internal("找不到应用数据目录"))
}

fn lock<'a>(state: &'a State<'_, AppState>) -> Result<std::sync::MutexGuard<'a, Connection>, AiError> {
    state
        .db
        .lock()
        .map_err(|error| AiError::internal(error.to_string()))
}

/// 把「图片/缩略图」读成 data URL，供多模态模型使用
fn read_as_data_url(path: &str) -> Option<String> {
    let path = Path::new(path);
    let size = fs::metadata(path).ok()?.len();
    if size == 0 || size > MAX_VISION_BYTES {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    let mime = match path
        .extension()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/jpeg",
    };

    Some(format!("data:{mime};base64,{}", BASE64.encode(bytes)))
}

fn parse_description(content: &str) -> Result<(String, Vec<String>), AiError> {
    let cleaned = content.trim();
    let cleaned = if cleaned.starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|line| !line.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    };

    let start = cleaned
        .find('{')
        .ok_or_else(|| AiError::internal("模型没有返回 JSON 结构，请重试或换一个模型"))?;
    let end = cleaned
        .rfind('}')
        .filter(|end| *end > start)
        .ok_or_else(|| AiError::internal("模型返回的 JSON 不完整，请重试"))?;

    let value: Value = serde_json::from_str(&cleaned[start..=end])
        .map_err(|error| AiError::internal(format!("解析素材描述失败：{error}")))?;

    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    if description.is_empty() {
        return Err(AiError::internal("模型没有给出描述内容，请重试"));
    }

    let tags = value
        .get("tags")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(|tag| tag.trim().replace(',', ""))
                .filter(|tag| !tag.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok((description, tags))
}

/* --------------------------------- 导入 --------------------------------- */

/// 打开系统文件选择框。返回用户选中的绝对路径，前端再调 material_import 导入。
/// 用 rfd 而不是额外插件，是为了少装一个 npm 包、也不吃 IPC 带宽。
#[tauri::command]
pub async fn material_pick() -> Result<Vec<String>, AiError> {
    let picked = rfd::AsyncFileDialog::new()
        .set_title("选择要导入的素材")
        .add_filter(
            "素材文件",
            &[
                "jpg", "jpeg", "png", "webp", "gif", "bmp", "mp4", "mov", "m4v", "webm", "mp3",
                "wav", "m4a", "aac", "ogg", "txt", "md", "markdown", "csv", "json",
            ],
        )
        .pick_files()
        .await;

    Ok(picked
        .map(|handles| {
            handles
                .into_iter()
                .map(|handle| handle.path().display().to_string())
                .collect()
        })
        .unwrap_or_default())
}

pub(crate) fn import_one(
    conn: &Connection,
    root: &Path,
    source: &Path,
    category_id: Option<i64>,
) -> Result<Value, String> {
    if !source.is_file() {
        return Err("文件不存在或不是普通文件".to_string());
    }

    let kind = MaterialType::from_path(source).ok_or_else(|| {
        let extension = source
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_else(|| "未知".to_string());
        format!("不支持的文件类型：.{extension}")
    })?;

    let folder = month_folder(root, &year_month(conn));
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

    let original_name = source
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "material".to_string());

    let destination = folder.join(unique_file_name(&original_name, stamp_millis()));
    fs::copy(source, &destination).map_err(|error| format!("复制文件失败：{error}"))?;

    let size_bytes = fs::metadata(&destination)
        .map(|meta| meta.len() as i64)
        .unwrap_or(0);

    // 图片本地生成缩略图；视频的缩略图由前端抓首帧后回传
    let mut thumb_path: Option<String> = None;
    if kind == MaterialType::Image {
        let target = thumb_path_for(&destination);
        if make_thumbnail(&destination, &target, THUMB_MAX_EDGE).is_ok() {
            thumb_path = Some(target.display().to_string());
        }
    }

    let display_name = source
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| original_name.clone());

    service::insert(
        conn,
        "materials",
        &json!({
            "type": kind.id(),
            "name": display_name,
            "file_path": destination.display().to_string(),
            "tags": "",
            "description": Value::Null,
            "category_id": category_id,
            "thumb_path": thumb_path,
            "size_bytes": size_bytes,
            "source_path": source.display().to_string(),
        })
        .as_object()
        .cloned()
        .unwrap_or_default(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn material_import(
    state: State<'_, AppState>,
    paths: Vec<String>,
    category_id: Option<i64>,
) -> Result<ImportOutcome, AiError> {
    if paths.is_empty() {
        return Err(AiError::config("没有选择任何文件"));
    }

    let root = library_root(&data_dir_of(&state)?);
    fs::create_dir_all(&root).map_err(|error| AiError::internal(error.to_string()))?;

    let conn = lock(&state)?;
    let mut imported = Vec::new();
    let mut skipped = Vec::new();

    for raw in paths {
        match import_one(&conn, &root, Path::new(&raw), category_id) {
            Ok(row) => imported.push(row),
            Err(reason) => skipped.push(ImportSkip { path: raw, reason }),
        }
    }

    Ok(ImportOutcome { imported, skipped })
}

/* --------------------------------- 删除 --------------------------------- */

#[tauri::command]
pub fn material_delete(
    state: State<'_, AppState>,
    id: i64,
    delete_file: Option<bool>,
) -> Result<bool, AiError> {
    let root = library_root(&data_dir_of(&state)?);
    let conn = lock(&state)?;

    let row = service::get(&conn, "materials", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到素材 #{id}")))?;

    if delete_file.unwrap_or(true) {
        remove_material_files(&root, &row);
    }

    service::delete(&conn, "materials", id).map_err(AiError::internal)
}

/// 删除素材对应的文件与缩略图。只会动素材库目录里的文件。
pub(crate) fn remove_material_files(root: &Path, row: &Value) -> Vec<String> {
    let mut removed = Vec::new();

    for key in ["file_path", "thumb_path"] {
        let Some(path) = row.get(key).and_then(Value::as_str) else {
            continue;
        };

        match ensure_inside_library(root, Path::new(path)) {
            Ok(target) => {
                if fs::remove_file(&target).is_ok() {
                    removed.push(target.display().to_string());
                }
            }
            Err(reason) => eprintln!("[materials] 跳过删除 {path}：{reason}"),
        }
    }

    removed
}

/* --------------------------------- 分类 --------------------------------- */

#[tauri::command]
pub fn material_categories(state: State<'_, AppState>) -> Result<Vec<Value>, AiError> {
    let conn = lock(&state)?;
    service::list(
        &conn,
        "material_categories",
        &service::ListQuery {
            order_by: Some("id".to_string()),
            ..Default::default()
        },
    )
    .map_err(AiError::internal)
}

#[tauri::command]
pub fn material_category_create(state: State<'_, AppState>, name: String) -> Result<Value, AiError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AiError::config("分类名不能为空"));
    }
    if name.chars().count() > 20 {
        return Err(AiError::config("分类名请控制在 20 个字以内"));
    }

    let conn = lock(&state)?;
    service::insert(
        &conn,
        "material_categories",
        &json!({ "name": name })
            .as_object()
            .cloned()
            .unwrap_or_default(),
    )
    .map_err(AiError::internal)
}

#[tauri::command]
pub fn material_category_delete(state: State<'_, AppState>, id: i64) -> Result<bool, AiError> {
    let conn = lock(&state)?;
    // 外键是 ON DELETE SET NULL，素材不会被一起删掉，只会回到「未分类」
    service::delete(&conn, "material_categories", id).map_err(AiError::internal)
}

/* --------------------------------- 文件内容 --------------------------------- */

#[tauri::command]
pub fn material_read_text(state: State<'_, AppState>, id: i64) -> Result<String, AiError> {
    let conn = lock(&state)?;
    let row = service::get(&conn, "materials", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到素材 #{id}")))?;

    let path = row
        .get("file_path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let bytes = fs::read(path).map_err(|error| AiError::internal(error.to_string()))?;
    let text = String::from_utf8_lossy(&bytes);

    Ok(text.chars().take(MAX_TEXT_CHARS).collect())
}

/// 视频缩略图：前端抓首帧后把 JPEG data URL 回传，这里落盘
#[tauri::command]
pub fn material_save_thumbnail(
    state: State<'_, AppState>,
    id: i64,
    data_url: String,
) -> Result<Value, AiError> {
    let payload = data_url
        .split(',')
        .nth(1)
        .ok_or_else(|| AiError::config("缩略图数据格式不正确"))?;
    let bytes = BASE64
        .decode(payload)
        .map_err(|error| AiError::internal(format!("缩略图解码失败：{error}")))?;

    if bytes.is_empty() {
        return Err(AiError::config("缩略图内容为空"));
    }

    let root = library_root(&data_dir_of(&state)?);
    let conn = lock(&state)?;

    let row = service::get(&conn, "materials", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到素材 #{id}")))?;

    let file_path = row
        .get("file_path")
        .and_then(Value::as_str)
        .ok_or_else(|| AiError::internal("这条素材没有文件路径"))?;

    let target = thumb_path_for(Path::new(file_path));

    // 目标目录必须落在素材库内，避免被篡改的路径写到别处
    let folder = target
        .parent()
        .ok_or_else(|| AiError::internal("缩略图路径不合法"))?
        .to_path_buf();
    fs::create_dir_all(&folder).map_err(|error| AiError::internal(error.to_string()))?;
    ensure_inside_library(&root, &folder).map_err(AiError::internal)?;

    fs::write(&target, &bytes).map_err(|error| AiError::internal(error.to_string()))?;

    service::update(
        &conn,
        "materials",
        id,
        &json!({ "thumb_path": target.display().to_string() })
            .as_object()
            .cloned()
            .unwrap_or_default(),
    )
    .map_err(AiError::internal)
}

/* ------------------------------- 给别的模块用 ------------------------------- */

#[tauri::command]
pub fn material_prompt_block(
    state: State<'_, AppState>,
    id: i64,
    purpose: Option<String>,
) -> Result<PromptBlock, AiError> {
    let purpose = purpose
        .as_deref()
        .and_then(Purpose::from_id)
        .unwrap_or(Purpose::Writing);

    let conn = lock(&state)?;
    let row = service::get(&conn, "materials", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到素材 #{id}")))?;

    let record = MaterialRecord::from_row(&row)?;

    Ok(PromptBlock {
        purpose: purpose.id().to_string(),
        purpose_label: purpose.label().to_string(),
        text: record.to_prompt_block(purpose),
    })
}

#[tauri::command]
pub fn material_library_info(state: State<'_, AppState>) -> Result<LibraryInfo, AiError> {
    let root = library_root(&data_dir_of(&state)?);
    let conn = lock(&state)?;

    let total_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM materials",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(LibraryInfo {
        directory: root.display().to_string(),
        count: service::count(&conn, "materials", &[]).map_err(AiError::internal)?,
        total_bytes,
        categories: service::count(&conn, "material_categories", &[])
            .map_err(AiError::internal)?,
    })
}

/* ------------------------------- AI 素材描述 ------------------------------- */

const DESCRIBE_SYSTEM_PROMPT: &str = "你是素材库整理助手。\n\
请为给定的素材写一段中文描述，并给出合适的标签。\n\
只输出 JSON，不要解释、不要用 markdown 代码块。";

fn describe_prompt(record: &MaterialRecord, text_excerpt: Option<&str>) -> String {
    let mut prompt = String::from(
        "请按下面这个 JSON 结构输出，键名必须完全一致：\n\
         {\n\
         \x20 \"description\": \"2-3 句中文描述：画面/内容是什么，适合用在什么地方\",\n\
         \x20 \"tags\": [\"3-5 个中文标签\"]\n\
         }\n\n素材信息：\n",
    );

    prompt.push_str(&format!("- 名称：{}\n", record.name));
    prompt.push_str(&format!("- 类型：{}\n", record.kind.label()));
    if !record.tags.is_empty() {
        prompt.push_str(&format!("- 现有标签：{}\n", record.tags.join("、")));
    }
    if let Some(description) = record.description.as_deref().filter(|text| !text.trim().is_empty()) {
        prompt.push_str(&format!("- 现有描述：{description}\n"));
    }
    if let Some(excerpt) = text_excerpt {
        prompt.push_str(&format!("\n文本内容节选：\n\"\"\"\n{excerpt}\n\"\"\"\n"));
    }

    prompt.push_str("\n标签要求：贴近「心身同调」领域的内容创作场景，例如画面类型、用途、情绪氛围。");
    prompt
}

#[tauri::command]
pub async fn ai_describe_material(
    state: State<'_, AppState>,
    id: i64,
) -> Result<MaterialDescription, AiError> {
    let (record, image_payload, text_excerpt) = {
        let conn = lock(&state)?;
        let row = service::get(&conn, "materials", id)
            .map_err(AiError::internal)?
            .ok_or_else(|| AiError::config(format!("找不到素材 #{id}")))?;

        let record = MaterialRecord::from_row(&row)?;

        // 图片优先用缩略图，视频用前端抓的首帧；文本素材带一段节选
        let image_payload = if record.kind.is_visual() {
            record
                .thumb_path
                .as_deref()
                .and_then(read_as_data_url)
                .or_else(|| {
                    (record.kind == MaterialType::Image).then(|| read_as_data_url(&record.file_path)).flatten()
                })
        } else {
            None
        };

        let text_excerpt = if record.kind == MaterialType::Text {
            fs::read(&record.file_path)
                .ok()
                .map(|bytes| {
                    String::from_utf8_lossy(&bytes)
                        .chars()
                        .take(2000)
                        .collect::<String>()
                })
        } else {
            None
        };

        (record, image_payload, text_excerpt)
    };

    let sent_image = image_payload.is_some();

    let request = ChatRequest {
        prompt: describe_prompt(&record, text_excerpt.as_deref()),
        system: Some(DESCRIBE_SYSTEM_PROMPT.to_string()),
        temperature: Some(0.4),
        max_tokens: Some(700),
        images: image_payload.into_iter().collect(),
        ..Default::default()
    };

    let response = run_text(&state, request).await?;
    let (description, tags) = parse_description(&response.content)?;

    // 自动写回：描述覆盖，标签合并去重
    let merged_tags = {
        let mut merged = record.tags.clone();
        for tag in &tags {
            if !merged.contains(tag) {
                merged.push(tag.clone());
            }
        }
        merged
    };

    let updated = {
        let conn = lock(&state)?;
        let stamp: String = conn
            .query_row("SELECT datetime('now', 'localtime')", [], |row| row.get(0))
            .unwrap_or_default();

        service::update(
            &conn,
            "materials",
            id,
            &json!({
                "description": description.clone(),
                "tags": join_tags(&merged_tags),
                "ai_description_at": stamp,
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )
        .map_err(AiError::internal)?
    };

    Ok(MaterialDescription {
        description,
        tags,
        provider: response.provider,
        provider_label: response.provider_label,
        model: response.model,
        used_trial_key: response.used_trial_key,
        remaining_free: response.remaining_free,
        sent_image,
        material: updated,
    })
}
