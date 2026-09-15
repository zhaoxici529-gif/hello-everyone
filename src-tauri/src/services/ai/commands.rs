//! 暴露给前端的 AI 命令。
//! 数据库读写和网络请求都分块执行，避免 MutexGuard 跨 await。

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::db::service::{self, Filter, ListQuery};
use crate::services::ai::{
    self, AiError, ChatRequest, ChatResponse, Provider, ProviderInfo, QuotaInfo, TestResult,
    TrialConfig,
};
use crate::services::secrets::Secrets;
use crate::AppState;

const KEY_PREFIX: &str = "api_key:";
const SELECTED_PROVIDER: &str = "ai_provider";
const SELECTED_MODEL: &str = "ai_model";
const NICKNAME: &str = "nickname";

type Guard<'a> = std::sync::MutexGuard<'a, Connection>;

fn lock<'a>(state: &'a State<'_, AppState>) -> Result<Guard<'a>, AiError> {
    state
        .db
        .lock()
        .map_err(|error| AiError::internal(error.to_string()))
}

/* --------------------------------- 读写助手 --------------------------------- */

fn key_setting_name(provider: Provider) -> String {
    format!("{KEY_PREFIX}{}", provider.id())
}

/// 取出并解密某个供应商的 Key。解密失败视为没配。
fn load_user_key(conn: &Connection, secrets: &Secrets, provider: Provider) -> Option<String> {
    let stored = service::setting_get(conn, &key_setting_name(provider))
        .ok()
        .flatten()?;
    secrets.decrypt(&stored).ok()
}

/// 已用掉的免费体验次数
fn free_used(conn: &Connection) -> i64 {
    service::count(
        conn,
        "ai_usage",
        &[Filter {
            column: "is_free_trial".to_string(),
            op: "eq".to_string(),
            value: Some(json!(1)),
        }],
    )
    .unwrap_or(0)
}

fn selected_provider(conn: &Connection) -> Provider {
    service::setting_get(conn, SELECTED_PROVIDER)
        .ok()
        .flatten()
        .and_then(|id| Provider::from_id(&id))
        .unwrap_or(Provider::DeepSeek)
}

fn read_setting(conn: &Connection, key: &str) -> String {
    service::setting_get(conn, key)
        .ok()
        .flatten()
        .unwrap_or_default()
}

/* --------------------------------- 返回结构 --------------------------------- */

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyStatus {
    pub provider: String,
    pub label: String,
    pub has_key: bool,
    pub masked: String,
    pub console_url: String,
    pub key_prefix: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub selected_provider: String,
    pub selected_model: String,
    pub nickname: String,
    /// 主密钥托管方式：keyring / keyFile
    pub key_source_id: String,
    pub key_source_label: String,
    pub keys: Vec<KeyStatus>,
    pub quota: QuotaInfo,
}

fn key_status(conn: &Connection, secrets: &Secrets, provider: Provider) -> KeyStatus {
    let plain = load_user_key(conn, secrets, provider);

    KeyStatus {
        provider: provider.id().to_string(),
        label: provider.label().to_string(),
        has_key: plain.is_some(),
        masked: plain.as_deref().map(Secrets::mask).unwrap_or_default(),
        console_url: provider.console_url().to_string(),
        key_prefix: provider.key_prefix().to_string(),
    }
}

fn key_statuses(conn: &Connection, secrets: &Secrets) -> Vec<KeyStatus> {
    Provider::ALL
        .iter()
        .map(|provider| key_status(conn, secrets, *provider))
        .collect()
}

/* --------------------------------- 命令实现 --------------------------------- */

#[tauri::command]
pub fn ai_providers() -> Vec<ProviderInfo> {
    Provider::ALL.iter().map(|p| (*p).into()).collect()
}

#[tauri::command]
pub fn ai_settings(state: State<'_, AppState>) -> Result<AiSettings, AiError> {
    let conn = lock(&state)?;
    let trial = TrialConfig::load();

    Ok(AiSettings {
        selected_provider: selected_provider(&conn).id().to_string(),
        selected_model: read_setting(&conn, SELECTED_MODEL),
        nickname: read_setting(&conn, NICKNAME),
        key_source_id: state.secrets.source().id().to_string(),
        key_source_label: state.secrets.source().label().to_string(),
        keys: key_statuses(&conn, &state.secrets),
        quota: QuotaInfo::build(&trial, free_used(&conn)),
    })
}

#[tauri::command]
pub fn ai_quota(state: State<'_, AppState>) -> Result<QuotaInfo, AiError> {
    let conn = lock(&state)?;
    Ok(QuotaInfo::build(&TrialConfig::load(), free_used(&conn)))
}

#[tauri::command]
pub fn ai_save_key(
    state: State<'_, AppState>,
    provider: String,
    api_key: String,
) -> Result<KeyStatus, AiError> {
    let provider = Provider::from_id(&provider).ok_or_else(|| AiError::config("未知的模型供应商"))?;
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(AiError::config("请先粘贴 API Key"));
    }

    let encrypted = state
        .secrets
        .encrypt(trimmed)
        .map_err(AiError::internal)?;

    let conn = lock(&state)?;
    service::setting_set(&conn, &key_setting_name(provider), &encrypted)
        .map_err(AiError::internal)?;

    Ok(key_status(&conn, &state.secrets, provider))
}

#[tauri::command]
pub fn ai_delete_key(state: State<'_, AppState>, provider: String) -> Result<KeyStatus, AiError> {
    let provider = Provider::from_id(&provider).ok_or_else(|| AiError::config("未知的模型供应商"))?;
    let conn = lock(&state)?;
    service::setting_delete(&conn, &key_setting_name(provider)).map_err(AiError::internal)?;
    Ok(key_status(&conn, &state.secrets, provider))
}

#[tauri::command]
pub fn ai_select(
    state: State<'_, AppState>,
    provider: String,
    model: Option<String>,
) -> Result<(), AiError> {
    let provider = Provider::from_id(&provider).ok_or_else(|| AiError::config("未知的模型供应商"))?;
    let conn = lock(&state)?;

    service::setting_set(&conn, SELECTED_PROVIDER, provider.id()).map_err(AiError::internal)?;
    service::setting_set(
        &conn,
        SELECTED_MODEL,
        model.unwrap_or_default().trim(),
    )
    .map_err(AiError::internal)?;

    Ok(())
}

#[tauri::command]
pub fn save_nickname(state: State<'_, AppState>, nickname: String) -> Result<String, AiError> {
    let value = nickname.trim().to_string();
    let conn = lock(&state)?;
    service::setting_set(&conn, NICKNAME, &value).map_err(AiError::internal)?;
    Ok(value)
}

/// 测试连接。带 apiKey 就测这一把（可以先测后存），不带就用已保存的。
#[tauri::command]
pub async fn ai_test_connection(
    state: State<'_, AppState>,
    provider: String,
    api_key: Option<String>,
) -> Result<TestResult, AiError> {
    let provider = Provider::from_id(&provider).ok_or_else(|| AiError::config("未知的模型供应商"))?;
    let typed = api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let key = match typed {
        Some(key) => key,
        None => {
            let conn = lock(&state)?;
            load_user_key(&conn, &state.secrets, provider).ok_or_else(AiError::missing_key)?
        }
    };

    ai::test_connection(provider, &key, None, &state.transport).await
}

/// 统一的文本生成执行：读配置 → 选 Key → 调用 → 落库。
/// 任何模块要调大模型都走这里，额度口径和用量统计才是统一的。
pub async fn run_text(
    state: &State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, AiError> {
    if request.prompt.trim().is_empty() {
        return Err(AiError::config("请先输入要生成的内容"));
    }

    let (selected, user_key, trial, used_before, model_override) = {
        let conn = lock(state)?;
        let selected = selected_provider(&conn);
        let user_key = load_user_key(&conn, &state.secrets, selected);
        let model = read_setting(&conn, SELECTED_MODEL);
        (selected, user_key, TrialConfig::load(), free_used(&conn), model)
    };
    let model_override = model_override.trim().to_string();

    let outcome = ai::chat(
        selected,
        user_key.as_deref(),
        &trial,
        used_before,
        Some(model_override.as_str()).filter(|value| !value.is_empty()),
        &state.transport,
        request,
    )
    .await?;

    let mut response = outcome.response;

    // 调用成功才计入用量：失败的调用（Key 无效 / 网络问题）不扣用户额度
    let remaining = {
        let conn = lock(state)?;
        service::insert(
            &conn,
            "ai_usage",
            &json!({
                "model": response.model,
                "type": "text",
                "tokens": response.tokens,
                "is_free_trial": outcome.used_trial,
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )
        .map_err(AiError::internal)?;

        QuotaInfo::build(&trial, free_used(&conn)).remaining
    };

    response.remaining_free = remaining;
    Ok(response)
}

/// 统一文本生成入口。额度用尽会返回 code = quota_exhausted，由前端弹引导。
#[tauri::command]
pub async fn ai_chat(
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, AiError> {
    run_text(&state, request).await
}

/// 最近的 AI 调用记录，给设置页展示
#[tauri::command]
pub fn ai_usage_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<Value>, AiError> {
    let conn = lock(&state)?;
    service::list(
        &conn,
        "ai_usage",
        &ListQuery {
            order_by: Some("id".to_string()),
            desc: true,
            limit: Some(limit.unwrap_or(10).clamp(1, 100)),
            ..Default::default()
        },
    )
    .map_err(AiError::internal)
}
