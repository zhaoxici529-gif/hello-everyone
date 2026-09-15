//! AI 适配层。
//!
//! 结构：
//! - `provider`   —— 已接入的模型供应商
//! - `adapters`   —— 每个模型一个 adapter，声明接入点
//! - `openai_compat` —— 统一的请求构造与响应解析
//! - `transport`  —— 传输层抽象（真实用 reqwest，测试用 Mock）
//! - `routing`    —— 决定这次用体验 Key 还是用户自己的 Key
//! - `trial`      —— 内置体验 Key 与免费额度
//! - `types` / `error` —— 统一入参出参、统一错误
//! - `commands`   —— 暴露给前端的 Tauri command

pub mod adapters;
pub mod commands;
pub mod error;
pub mod openai_compat;
pub mod provider;
pub mod routing;
pub mod transport;
pub mod trial;
pub mod types;

#[cfg(test)]
mod tests;

use std::time::Instant;

use serde::Serialize;

pub use error::AiError;
pub use provider::{Provider, ProviderInfo};
pub use routing::choose_route;
pub use transport::{HttpTransport, Transport};
pub use trial::{QuotaInfo, TrialConfig};
pub use types::{ChatRequest, ChatResponse, ChatTurn, ImageRequest, ImageResponse};

/// 一次对话调用的结果
#[derive(Debug)]
pub struct ChatOutcome {
    pub response: ChatResponse,
    pub used_trial: bool,
}

/// 走完「选 Key → 挑 adapter → 发请求 → 解析响应」的完整链路。
#[allow(clippy::too_many_arguments)]
pub async fn chat(
    selected: Provider,
    user_key: Option<&str>,
    trial: &TrialConfig,
    free_used: i64,
    model_override: Option<&str>,
    transport: &dyn Transport,
    request: ChatRequest,
) -> Result<ChatOutcome, AiError> {
    let route = choose_route(selected, user_key, trial, free_used)?;
    let provider = route.provider();
    let adapter = adapters::adapter_for(provider);

    // 模型优先级：调用方指定 > 请求里带的 > 体验配置 > adapter 默认
    let model = model_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| request.model.clone().filter(|value| !value.trim().is_empty()))
        .or_else(|| {
            (route.is_trial() && !trial.model.trim().is_empty()).then(|| trial.model.clone())
        })
        .unwrap_or_else(|| adapter.default_model().to_string());

    let http = openai_compat::build_chat_request(&adapter.endpoint, &model, route.key(), &request);

    let started = Instant::now();
    let response = transport.send(http).await?;
    let latency_ms = started.elapsed().as_millis() as u64;

    if !(200..=299).contains(&response.status) {
        return Err(openai_compat::classify_status(response.status, &response.body));
    }

    let (content, tokens) = openai_compat::parse_chat_response(&response.body)?;
    let used_trial = route.is_trial();

    Ok(ChatOutcome {
        used_trial,
        response: ChatResponse {
            provider: provider.id().to_string(),
            provider_label: provider.label().to_string(),
            model,
            content,
            tokens,
            used_trial_key: used_trial,
            remaining_free: 0,
            latency_ms,
        },
    })
}

/// 测试连接：发一句最小请求，看 Key 能不能用。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub ok: bool,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
    pub latency_ms: u64,
    pub reply: String,
}

pub async fn test_connection(
    provider: Provider,
    api_key: &str,
    model: Option<&str>,
    transport: &dyn Transport,
) -> Result<TestResult, AiError> {
    let adapter = adapters::adapter_for(provider);
    let model = model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(adapter.default_model())
        .to_string();

    let request = ChatRequest {
        prompt: "你好，请只回复两个字：正常".to_string(),
        max_tokens: Some(16),
        ..Default::default()
    };

    let http = openai_compat::build_chat_request(&adapter.endpoint, &model, api_key, &request);

    let started = Instant::now();
    let response = transport.send(http).await?;
    let latency_ms = started.elapsed().as_millis() as u64;

    if !(200..=299).contains(&response.status) {
        return Err(openai_compat::classify_status(response.status, &response.body));
    }

    let (reply, _) = openai_compat::parse_chat_response(&response.body)?;

    Ok(TestResult {
        ok: true,
        provider: provider.id().to_string(),
        provider_label: provider.label().to_string(),
        model,
        latency_ms,
        reply,
    })
}

/// 图片生成占位：接口签名先定下来，后续接入具体模型时补实现。
#[allow(dead_code)]
pub async fn generate_image(
    _provider: Provider,
    _api_key: &str,
    _request: ImageRequest,
    _transport: &dyn Transport,
) -> Result<ImageResponse, AiError> {
    Err(AiError::new(
        "not_implemented",
        "图片生成暂未接入",
        "当前版本先支持文本生成，图片能力会在后续版本开放。",
    ))
}
