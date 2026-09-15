//! 人物小传的 Tauri command。
//! 通用增删改查走 db 层的通用 CRUD，这里只补「AI 提取」和「给别的模块用的提示词 API」。

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::db::service;
use crate::services::ai::commands::run_text;
use crate::services::ai::{AiError, ChatRequest};
use crate::services::personas::{
    category_label, extraction_user_prompt, parse_extraction, PersonaFields, Purpose, CATEGORIES,
    EXTRACTION_SYSTEM_PROMPT,
};
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaExtraction {
    pub fields: PersonaFields,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
    pub used_trial_key: bool,
    pub remaining_free: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBlock {
    pub purpose: String,
    pub purpose_label: String,
    pub text: String,
}

fn parse_fields(row: &Value) -> Result<PersonaFields, AiError> {
    let raw = row.get("fields").and_then(Value::as_str).unwrap_or("{}");
    if raw.trim().is_empty() {
        return Ok(PersonaFields::default());
    }
    serde_json::from_str(raw)
        .map_err(|error| AiError::internal(format!("人物档案内容解析失败：{error}")))
}

#[tauri::command]
pub fn persona_categories() -> Vec<CategoryInfo> {
    CATEGORIES
        .iter()
        .map(|id| CategoryInfo {
            id: (*id).to_string(),
            label: category_label(id).to_string(),
        })
        .collect()
}

/// 给其它模块（AI 写作 / 模拟采访 / 图片生成）取人物档案的提示词块。
#[tauri::command]
pub fn persona_prompt_block(
    state: State<'_, AppState>,
    id: i64,
    purpose: Option<String>,
) -> Result<PromptBlock, AiError> {
    let purpose = purpose
        .as_deref()
        .and_then(Purpose::from_id)
        .unwrap_or(Purpose::Writing);

    let conn = state
        .db
        .lock()
        .map_err(|error| AiError::internal(error.to_string()))?;

    let row = service::get(&conn, "personas", id)
        .map_err(AiError::internal)?
        .ok_or_else(|| AiError::config(format!("找不到人物档案 #{id}")))?;

    let name = row
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("未命名");
    let category = row
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let fields = parse_fields(&row)?;
    let mut text = fields.to_prompt_block(name, category, purpose);

    // 让调用方知道这份档案其实是空的，避免把空白人设喂给模型
    if fields.is_blank() {
        text.push_str("\n（这份人物档案还没有填写内容）");
    }

    Ok(PromptBlock {
        purpose: purpose.id().to_string(),
        purpose_label: purpose.label().to_string(),
        text,
    })
}

/// AI 提取特征：把一段已有文案喂给模型，抽取成人物档案字段。
#[tauri::command]
pub async fn ai_extract_persona(
    state: State<'_, AppState>,
    source_text: String,
    name: Option<String>,
) -> Result<PersonaExtraction, AiError> {
    let source = source_text.trim();
    if source.chars().count() < 20 {
        return Err(AiError::config("至少粘贴 20 个字的文案，提取才有意义"));
    }

    let response = run_text(
        &state,
        ChatRequest {
            prompt: extraction_user_prompt(source, name.as_deref()),
            system: Some(EXTRACTION_SYSTEM_PROMPT.to_string()),
            temperature: Some(0.3),
            max_tokens: Some(1500),
            ..Default::default()
        },
    )
    .await?;

    let fields = parse_extraction(&response.content)?;

    Ok(PersonaExtraction {
        fields,
        provider: response.provider,
        provider_label: response.provider_label,
        model: response.model,
        used_trial_key: response.used_trial_key,
        remaining_free: response.remaining_free,
    })
}
