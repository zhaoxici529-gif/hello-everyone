//! AI 写作的 Tauri command。

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::db::service;
use crate::services::ai::commands::run_text;
use crate::services::ai::{AiError, ChatRequest, ChatTurn};
use crate::services::materials::{self, MaterialRecord};
use crate::services::personas::{PersonaFields, Purpose as PersonaPurpose};
use crate::services::writing::{
    build_revise_prompt, build_revise_system_prompt, build_system_prompt, build_user_prompt,
    export_file_stem, html_to_markdown, html_to_text, parse_draft, parse_revision, Genre,
    ImageSuggestion, WriteVersion, ALL_GENRES,
};
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenreInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WriteRequest {
    pub topic: String,
    pub genre: Option<String>,
    pub persona_id: Option<i64>,
    pub material_ids: Vec<i64>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    pub genre: String,
    pub genre_label: String,
    pub versions: Vec<WriteVersion>,
    pub image_keywords: Vec<ImageSuggestion>,
    pub persona_id: Option<i64>,
    /// 这次是否真的带上了人物档案
    pub used_persona: bool,
    pub material_count: usize,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
    pub used_trial_key: bool,
    pub remaining_free: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReviseRequest {
    pub content: String,
    pub instruction: String,
    pub genre: Option<String>,
    pub persona_id: Option<i64>,
    pub history: Vec<ChatTurn>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviseResult {
    pub content: String,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
    pub used_trial_key: bool,
    pub remaining_free: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub filename: String,
    pub content: String,
    pub format: String,
}

/* --------------------------------- 取档案 --------------------------------- */

fn persona_block(conn: &Connection, id: Option<i64>) -> Option<String> {
    let row = service::get(conn, "personas", id?).ok()??;
    let name = row.get("name").and_then(Value::as_str)?;
    let category = row
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let fields: PersonaFields =
        serde_json::from_str(row.get("fields").and_then(Value::as_str).unwrap_or("{}")).ok()?;

    Some(fields.to_prompt_block(name, category, PersonaPurpose::Writing))
}

fn material_blocks(conn: &Connection, ids: &[i64]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| {
            let row = service::get(conn, "materials", *id).ok()??;
            let record = MaterialRecord::from_row(&row).ok()?;
            Some(record.to_prompt_block(materials::Purpose::Writing))
        })
        .collect()
}

fn genre_of(id: Option<&str>) -> Genre {
    id.and_then(Genre::from_id).unwrap_or(Genre::Xiaohongshu)
}

/* --------------------------------- 命令 --------------------------------- */

#[tauri::command]
pub fn writing_genres() -> Vec<GenreInfo> {
    ALL_GENRES
        .iter()
        .map(|genre| GenreInfo {
            id: genre.id().to_string(),
            label: genre.label().to_string(),
        })
        .collect()
}

/// 一次生成 3 个版本 + 配图建议
#[tauri::command]
pub async fn ai_write_draft(
    state: State<'_, AppState>,
    request: WriteRequest,
) -> Result<WriteResult, AiError> {
    let topic = request.topic.trim().to_string();
    if topic.is_empty() {
        return Err(AiError::config("先写一个主题，比如「失眠调理」"));
    }

    let genre = genre_of(request.genre.as_deref());

    let (persona, materials) = {
        let conn = state
            .db
            .lock()
            .map_err(|error| AiError::internal(error.to_string()))?;
        (
            persona_block(&conn, request.persona_id),
            material_blocks(&conn, &request.material_ids),
        )
    };

    let used_persona = persona.is_some();
    let material_count = materials.len();
    let system = build_system_prompt(genre, persona.as_deref());
    let prompt = build_user_prompt(&topic, request.extra.as_deref(), &materials);

    let response = run_text(
        &state,
        ChatRequest {
            prompt,
            system: Some(system),
            temperature: Some(0.85),
            max_tokens: Some(4000),
            ..Default::default()
        },
    )
    .await?;

    let payload = parse_draft(&response.content)?;

    Ok(WriteResult {
        genre: genre.id().to_string(),
        genre_label: genre.label().to_string(),
        versions: payload.versions,
        image_keywords: payload.image_keywords,
        persona_id: request.persona_id,
        used_persona,
        material_count,
        provider: response.provider,
        provider_label: response.provider_label,
        model: response.model,
        used_trial_key: response.used_trial_key,
        remaining_free: response.remaining_free,
    })
}

/// 对话式修改：带着历史继续追问「再口语化一点」
#[tauri::command]
pub async fn ai_revise_draft(
    state: State<'_, AppState>,
    request: ReviseRequest,
) -> Result<ReviseResult, AiError> {
    let content = request.content.trim().to_string();
    let instruction = request.instruction.trim().to_string();

    if content.is_empty() {
        return Err(AiError::config("还没有可以修改的文案"));
    }
    if instruction.is_empty() {
        return Err(AiError::config("说一下你想怎么改，比如「再口语化一点」"));
    }

    let genre = request.genre.as_deref().and_then(Genre::from_id);

    let persona = {
        let conn = state
            .db
            .lock()
            .map_err(|error| AiError::internal(error.to_string()))?;
        persona_block(&conn, request.persona_id)
    };

    let system = build_revise_system_prompt(genre, persona.as_deref());
    let prompt = build_revise_prompt(&content, &instruction);

    let response = run_text(
        &state,
        ChatRequest {
            prompt,
            system: Some(system),
            temperature: Some(0.7),
            max_tokens: Some(4000),
            history: request.history,
            ..Default::default()
        },
    )
    .await?;

    Ok(ReviseResult {
        content: parse_revision(&response.content)?,
        provider: response.provider,
        provider_label: response.provider_label,
        model: response.model,
        used_trial_key: response.used_trial_key,
        remaining_free: response.remaining_free,
    })
}

/// 导出文案：返回文件名和正文，由前端落成 txt / md 文件。
#[tauri::command]
pub fn writing_export(
    state: State<'_, AppState>,
    id: i64,
    format: Option<String>,
) -> Result<ExportResult, AiError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| AiError::internal(error.to_string()))?;

    let row = service::get(&conn, "writings", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到文案 #{id}")))?;

    let title = row
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("未命名文案");
    let body = row.get("content").and_then(Value::as_str).unwrap_or_default();

    let want_markdown = !matches!(format.as_deref(), Some("txt"));

    let content = if want_markdown {
        format!("# {title}\n\n{}", html_to_markdown(body))
    } else {
        format!("{title}\n{}\n\n{}", "=".repeat(title.chars().count()), html_to_text(body))
    };

    Ok(ExportResult {
        filename: format!(
            "{}.{}",
            export_file_stem(title, id),
            if want_markdown { "md" } else { "txt" }
        ),
        content,
        format: if want_markdown { "md" } else { "txt" }.to_string(),
    })
}
