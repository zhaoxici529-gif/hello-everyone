use serde_json::{json, Value};

use super::error::AiError;
use super::transport::HttpRequest;
use super::types::ChatRequest;

/// 一个 OpenAI 兼容的 chat/completions 端点。
/// 五家供应商都是这个协议，差异只在 url 和默认模型名。
#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    pub url: &'static str,
    pub default_model: &'static str,
}

/// 统一的请求构造：所有 adapter 共用。
pub fn build_chat_request(
    endpoint: &Endpoint,
    model: &str,
    api_key: &str,
    request: &ChatRequest,
) -> HttpRequest {
    let mut messages = Vec::new();

    if let Some(system) = request.system.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        messages.push(json!({ "role": "system", "content": system }));
    }

    // 对话式修改：把之前的往返带上，模型才知道「再口语化一点」是在改哪一版
    for turn in &request.history {
        let role = turn.role.trim().to_ascii_lowercase();
        if role != "user" && role != "assistant" {
            continue;
        }
        messages.push(json!({ "role": role, "content": turn.content }));
    }

    // 纯文本用字符串；带图片时用 OpenAI 的多模态内容数组
    let user_content = if request.images.is_empty() {
        json!(request.prompt)
    } else {
        let mut parts = vec![json!({ "type": "text", "text": request.prompt })];
        for image in &request.images {
            parts.push(json!({
                "type": "image_url",
                "image_url": { "url": image }
            }));
        }
        json!(parts)
    };
    messages.push(json!({ "role": "user", "content": user_content }));

    let mut body = json!({
        "model": model,
        "messages": messages,
        "stream": false,
    });

    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }
    if let Some(max_tokens) = request.max_tokens {
        body["max_tokens"] = json!(max_tokens);
    }

    HttpRequest {
        url: endpoint.url.to_string(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {api_key}")),
        ],
        body: body.to_string(),
    }
}

/// 统一的响应解析：返回 (正文, 总 token 数)
pub fn parse_chat_response(body: &str) -> Result<(String, i64), AiError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| AiError::internal("模型返回的内容不是合法 JSON，可能被网关改写过"))?;

    let content = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    if content.is_empty() {
        return Err(AiError::internal("模型没有返回文本内容，换一个模型或稍后重试"));
    }

    let tokens = value
        .pointer("/usage/total_tokens")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    Ok((content, tokens))
}

/// 从错误响应里捞出可读信息
pub fn parse_error_message(body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        for pointer in ["/error/message", "/message", "/error/code", "/error/type"] {
            if let Some(text) = value.pointer(pointer).and_then(Value::as_str) {
                if !text.trim().is_empty() {
                    return text.trim().to_string();
                }
            }
        }
    }

    let trimmed = body.trim();
    if trimmed.is_empty() {
        "服务端没有返回错误详情".to_string()
    } else {
        trimmed.chars().take(180).collect()
    }
}

/// HTTP 状态码 → 友好错误
pub fn classify_status(status: u16, body: &str) -> AiError {
    let detail = parse_error_message(body);

    match status {
        401 | 403 => AiError::invalid_key(detail),
        402 => AiError::insufficient_balance(detail),
        404 => AiError::provider(404, format!("{detail}（模型名可能不对）")),
        429 => AiError::rate_limited(),
        500..=599 => AiError::provider(status, "模型服务暂时不可用，稍后再试"),
        _ => AiError::provider(status, detail),
    }
}

/// 解析图片生成响应（预留）
#[allow(dead_code)]
pub fn parse_image_response(body: &str) -> Result<Vec<String>, AiError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| AiError::internal("图片接口返回的内容无法解析"))?;

    let images = value
        .pointer("/data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("url")
                        .or_else(|| item.get("b64_json"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if images.is_empty() {
        return Err(AiError::internal("图片接口没有返回图片"));
    }
    Ok(images)
}
