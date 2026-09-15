//! AI 适配层单元测试。全部走 MockTransport，不需要联网。
//! 需要真实网络的用例标了 #[ignore]，用 `cargo test -- --ignored` 单独跑。

use std::sync::Mutex;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::db::service::Filter;
use crate::db::{migrations, service};
use crate::services::secrets::Secrets;

use super::adapters::{adapter_for, Adapter};
use super::error::AiError;
use super::openai_compat::{build_chat_request, classify_status, parse_chat_response, Endpoint};
use super::provider::Provider;
use super::routing::{choose_route, Route};
use super::transport::{BoxFuture, HttpRequest, HttpResponse, Transport};
use super::trial::TrialConfig;
use super::types::ChatRequest;
use super::{chat, test_connection};

/* --------------------------------- 测试替身 --------------------------------- */

struct MockTransport {
    response: Result<HttpResponse, AiError>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl MockTransport {
    fn ok(body: Value) -> Self {
        Self::new(Ok(HttpResponse {
            status: 200,
            body: body.to_string(),
        }))
    }

    fn status(status: u16, body: Value) -> Self {
        Self::new(Ok(HttpResponse {
            status,
            body: body.to_string(),
        }))
    }

    fn failing(error: AiError) -> Self {
        Self::new(Err(error))
    }

    fn new(response: Result<HttpResponse, AiError>) -> Self {
        Self {
            response,
            requests: Mutex::new(Vec::new()),
        }
    }

    fn last_request(&self) -> HttpRequest {
        self.requests
            .lock()
            .unwrap()
            .last()
            .cloned()
            .expect("没有记录到请求")
    }

    fn call_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

impl Transport for MockTransport {
    fn send<'a>(&'a self, request: HttpRequest) -> BoxFuture<'a, Result<HttpResponse, AiError>> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            self.response.clone()
        })
    }
}

fn chat_reply(content: &str, tokens: i64) -> Value {
    json!({
        "id": "chatcmpl-test",
        "choices": [{ "index": 0, "message": { "role": "assistant", "content": content } }],
        "usage": { "prompt_tokens": 10, "completion_tokens": tokens - 10, "total_tokens": tokens }
    })
}

fn trial_config() -> TrialConfig {
    TrialConfig {
        enabled: true,
        free_quota: 3,
        provider: "deepseek".to_string(),
        model: "deepseek-chat".to_string(),
        // 长度足够、且不含占位文案，视为已配置
        api_key: "sk-trial-0123456789abcdef".to_string(),
    }
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tauri::async_runtime::block_on(future)
}

/* --------------------------------- 适配器 --------------------------------- */

#[test]
fn every_provider_has_a_usable_adapter() {
    for provider in Provider::ALL {
        let Adapter { endpoint, .. } = adapter_for(provider);

        assert!(
            endpoint.url.starts_with("https://"),
            "{} 的接入点必须是 https",
            provider.label()
        );
        assert!(
            endpoint.url.ends_with("/chat/completions"),
            "{} 应使用 OpenAI 兼容的 chat/completions 路径",
            provider.label()
        );
        assert_eq!(
            endpoint.default_model,
            provider.default_model(),
            "{} 的默认模型名不一致",
            provider.label()
        );
        assert!(!provider.console_url().is_empty());
        assert!(!provider.models().is_empty());
    }
}

#[test]
fn provider_ids_round_trip() {
    for provider in Provider::ALL {
        assert_eq!(Provider::from_id(provider.id()), Some(provider));
    }
    assert_eq!(Provider::from_id("  DeepSeek "), Some(Provider::DeepSeek));
    assert_eq!(Provider::from_id("unknown"), None);
}

/* --------------------------------- 请求构造 --------------------------------- */

#[test]
fn build_chat_request_sets_url_headers_and_body() {
    let endpoint = adapter_for(Provider::DeepSeek).endpoint;
    let request = ChatRequest::new("帮我写 3 个标题").with_system("你是心身同调领域的内容助手");

    let http = build_chat_request(&endpoint, "deepseek-chat", "sk-test-key", &request);

    assert_eq!(http.url, "https://api.deepseek.com/v1/chat/completions");
    assert!(http
        .headers
        .contains(&("Authorization".to_string(), "Bearer sk-test-key".to_string())));

    let body: Value = serde_json::from_str(&http.body).unwrap();
    assert_eq!(body["model"], json!("deepseek-chat"));
    assert_eq!(body["stream"], json!(false));
    assert_eq!(body["messages"][0]["role"], json!("system"));
    assert_eq!(body["messages"][1]["content"], json!("帮我写 3 个标题"));
}

#[test]
fn build_chat_request_skips_empty_system_prompt() {
    let endpoint = adapter_for(Provider::Kimi).endpoint;
    let request = ChatRequest::new("你好").with_system("   ");

    let http = build_chat_request(&endpoint, "moonshot-v1-8k", "sk-x", &request);
    let body: Value = serde_json::from_str(&http.body).unwrap();

    assert_eq!(body["messages"].as_array().unwrap().len(), 1);
    assert_eq!(body["messages"][0]["role"], json!("user"));
}

#[test]
fn build_chat_request_attaches_images_as_multimodal_parts() {
    let endpoint = adapter_for(Provider::Glm).endpoint;
    let mut request = ChatRequest::new("这张图适合做什么封面？");
    request.images = vec!["data:image/jpeg;base64,AAAA".to_string()];

    let http = build_chat_request(&endpoint, "glm-4v-flash", "sk-x", &request);
    let body: Value = serde_json::from_str(&http.body).unwrap();
    let content = body["messages"][0]["content"].as_array().unwrap();

    assert_eq!(content[0]["type"], json!("text"));
    assert_eq!(content[0]["text"], json!("这张图适合做什么封面？"));
    assert_eq!(content[1]["type"], json!("image_url"));
    assert_eq!(content[1]["image_url"]["url"], json!("data:image/jpeg;base64,AAAA"));
}

#[test]
fn build_chat_request_replays_conversation_history() {
    use super::types::ChatTurn;

    let endpoint = adapter_for(Provider::DeepSeek).endpoint;
    let mut request = ChatRequest::new("这是当前文案：\n\"\"\"\n早起三件事\n\"\"\"\n修改要求：再口语化一点");
    request.system = Some("你是内容助手".to_string());
    request.history = vec![
        ChatTurn {
            role: "user".to_string(),
            content: "再口语化一点".to_string(),
        },
        ChatTurn {
            role: "assistant".to_string(),
            content: "上一版改好的文案".to_string(),
        },
        // 非法角色会被忽略，避免把脏数据发给模型
        ChatTurn {
            role: "system".to_string(),
            content: "不该出现".to_string(),
        },
    ];

    let http = build_chat_request(&endpoint, "deepseek-chat", "sk-x", &request);
    let body: Value = serde_json::from_str(&http.body).unwrap();
    let messages = body["messages"].as_array().unwrap();

    assert_eq!(messages.len(), 4, "system + 两轮历史 + 当前提问");
    assert_eq!(messages[0]["role"], json!("system"));
    assert_eq!(messages[1]["role"], json!("user"));
    assert_eq!(messages[1]["content"], json!("再口语化一点"));
    assert_eq!(messages[2]["role"], json!("assistant"));
    assert_eq!(messages[2]["content"], json!("上一版改好的文案"));
    assert_eq!(messages[3]["role"], json!("user"));
    assert!(messages[3]["content"].as_str().unwrap().contains("修改要求"));
}

#[test]
fn vision_models_are_declared_per_provider() {
    // 只有声明了 vision_models 的供应商才可能看图
    assert!(Provider::DeepSeek.vision_models().is_empty());
    for provider in [Provider::Qwen, Provider::Doubao, Provider::Kimi, Provider::Glm] {
        assert!(
            !provider.vision_models().is_empty(),
            "{} 应该有可选的视觉模型",
            provider.label()
        );
        for model in provider.vision_models() {
            assert!(
                provider.models().contains(model),
                "{} 的视觉模型 {model} 应该在可选模型列表里",
                provider.label()
            );
        }
    }
}

/* ------------------------------- 响应与错误解析 ------------------------------- */

#[test]
fn parse_chat_response_reads_content_and_tokens() {
    let (content, tokens) = parse_chat_response(&chat_reply("写好了", 128).to_string()).unwrap();
    assert_eq!(content, "写好了");
    assert_eq!(tokens, 128);
}

#[test]
fn parse_chat_response_rejects_broken_payloads() {
    assert!(parse_chat_response("not json").is_err());
    assert!(parse_chat_response(&json!({ "choices": [] }).to_string()).is_err());
    assert!(parse_chat_response(&chat_reply("   ", 10).to_string()).is_err());
}

#[test]
fn status_codes_map_to_friendly_errors() {
    let unauthorized = classify_status(401, &json!({ "error": { "message": "Authentication Fails" } }).to_string());
    assert_eq!(unauthorized.code, "invalid_key");
    assert!(unauthorized.message.contains("Authentication Fails"));
    assert!(!unauthorized.hint.is_empty());

    assert_eq!(classify_status(403, "{}").code, "invalid_key");
    assert_eq!(classify_status(402, "{}").code, "insufficient_balance");
    assert_eq!(classify_status(429, "{}").code, "rate_limited");
    assert_eq!(classify_status(500, "{}").code, "provider_error");
    assert_eq!(classify_status(503, "{}").code, "provider_error");
}

/* -------------------------------- 额度路由 -------------------------------- */

#[test]
fn user_key_wins_over_trial() {
    let trial = trial_config();
    let route = choose_route(Provider::Kimi, Some("sk-mine"), &trial, 0).unwrap();

    match route {
        Route::User { provider, key } => {
            assert_eq!(provider, Provider::Kimi, "应使用用户选中的模型");
            assert_eq!(key, "sk-mine");
        }
        other => panic!("应该走用户 Key，实际是 {other:?}"),
    }
}

#[test]
fn blank_user_key_falls_back_to_trial() {
    let trial = trial_config();
    let route = choose_route(Provider::Glm, Some("   "), &trial, 0).unwrap();

    assert!(route.is_trial());
    assert_eq!(route.provider(), Provider::DeepSeek, "体验额度固定用体验配置的模型");
}

#[test]
fn trial_is_used_for_first_three_calls_then_exhausted() {
    let trial = trial_config();

    for used in 0..3 {
        let route = choose_route(Provider::DeepSeek, None, &trial, used).unwrap();
        assert!(route.is_trial(), "第 {} 次应该还能用体验额度", used + 1);
    }

    let error = choose_route(Provider::DeepSeek, None, &trial, 3).unwrap_err();
    assert_eq!(error.code, "quota_exhausted");
    assert!(error.is_quota_exhausted());
    assert!(error.message.contains('3'));
}

#[test]
fn trial_placeholder_key_is_treated_as_unconfigured() {
    let trial = TrialConfig {
        enabled: true,
        free_quota: 3,
        provider: "deepseek".to_string(),
        model: "deepseek-chat".to_string(),
        api_key: "sk-在此填入体验Key".to_string(),
    };

    assert!(!trial.is_configured());

    let error = choose_route(Provider::DeepSeek, None, &trial, 0).unwrap_err();
    assert_eq!(error.code, "trial_not_configured");
    assert!(error.hint.contains("trial.json"));
}

/* --------------------------------- 完整调用 --------------------------------- */

#[test]
fn chat_uses_trial_key_and_returns_content() {
    let transport = MockTransport::ok(chat_reply("这是体验额度生成的内容", 66));
    let trial = trial_config();

    let outcome = block_on(chat(
        Provider::DeepSeek,
        None,
        &trial,
        0,
        None,
        &transport,
        ChatRequest::new("写一句开场白"),
    ))
    .unwrap();

    assert!(outcome.used_trial);
    assert!(outcome.response.used_trial_key);
    assert_eq!(outcome.response.content, "这是体验额度生成的内容");
    assert_eq!(outcome.response.tokens, 66);
    assert_eq!(outcome.response.provider, "deepseek");
    assert_eq!(outcome.response.model, "deepseek-chat");

    let sent = transport.last_request();
    assert!(sent
        .headers
        .contains(&("Authorization".to_string(), "Bearer sk-trial-0123456789abcdef".to_string())));
}

#[test]
fn chat_uses_user_key_when_provided() {
    let transport = MockTransport::ok(chat_reply("用自己的 Key 生成的内容", 42));
    let trial = trial_config();

    let outcome = block_on(chat(
        Provider::Qwen,
        Some("sk-my-own-key"),
        &trial,
        3,
        Some("qwen-max"),
        &transport,
        ChatRequest::new("你好"),
    ))
    .unwrap();

    assert!(!outcome.used_trial, "有用户 Key 时不该再消耗体验额度");
    assert_eq!(outcome.response.provider, "qwen");
    assert_eq!(outcome.response.model, "qwen-max");

    let sent = transport.last_request();
    assert!(sent.url.contains("dashscope.aliyuncs.com"));
    assert!(sent
        .headers
        .contains(&("Authorization".to_string(), "Bearer sk-my-own-key".to_string())));
}

#[test]
fn chat_blocks_before_network_when_quota_is_used_up() {
    let transport = MockTransport::ok(chat_reply("不该被调用", 1));
    let trial = trial_config();

    let error = block_on(chat(
        Provider::DeepSeek,
        None,
        &trial,
        3,
        None,
        &transport,
        ChatRequest::new("第 4 次"),
    ))
    .unwrap_err();

    assert!(error.is_quota_exhausted());
    assert_eq!(transport.call_count(), 0, "额度用尽时不应该发出网络请求");
}

#[test]
fn chat_surfaces_provider_errors_and_network_failures() {
    let trial = trial_config();

    let bad_key = MockTransport::status(401, json!({ "error": { "message": "invalid api key" } }));
    let error = block_on(chat(
        Provider::DeepSeek,
        Some("sk-wrong"),
        &trial,
        0,
        None,
        &bad_key,
        ChatRequest::new("你好"),
    ))
    .unwrap_err();
    assert_eq!(error.code, "invalid_key");

    let offline = MockTransport::failing(AiError::network("连不上模型服务"));
    let error = block_on(chat(
        Provider::DeepSeek,
        Some("sk-any"),
        &trial,
        0,
        None,
        &offline,
        ChatRequest::new("你好"),
    ))
    .unwrap_err();
    assert_eq!(error.code, "network_error");
    assert!(error.message.contains("网络连接失败"));
}

#[test]
fn test_connection_reports_success_and_failure() {
    let good = MockTransport::ok(chat_reply("正常", 8));
    let result = block_on(test_connection(
        Provider::Glm,
        "sk-good",
        None,
        &good,
    ))
    .unwrap();
    assert!(result.ok);
    assert_eq!(result.model, "glm-4-flash");
    assert_eq!(result.reply, "正常");
    assert!(good.last_request().url.contains("bigmodel.cn"));

    let bad = MockTransport::status(401, json!({ "error": { "message": "unauthorized" } }));
    let error = block_on(test_connection(Provider::Glm, "sk-bad", None, &bad)).unwrap_err();
    assert_eq!(error.code, "invalid_key");
}

#[test]
fn image_generation_is_explicitly_not_implemented_yet() {
    let transport = MockTransport::ok(json!({ "data": [{ "url": "https://example.com/a.png" }] }));
    let error = block_on(super::generate_image(
        Provider::DeepSeek,
        "sk-x",
        super::ImageRequest {
            prompt: "一张封面图".to_string(),
            size: None,
        },
        &transport,
    ))
    .unwrap_err();

    assert_eq!(error.code, "not_implemented");
    assert_eq!(transport.call_count(), 0);
}

/* ------------------------------ 真实网络（可选） ------------------------------ */

#[test]
#[ignore = "需要联网，用 cargo test -- --ignored 单独跑"]
fn real_deepseek_rejects_an_invalid_key() {
    let transport = super::HttpTransport::new(20).unwrap();
    let error = block_on(test_connection(
        Provider::DeepSeek,
        "sk-definitely-not-a-real-key-000",
        None,
        &transport,
    ))
    .unwrap_err();

    // 真实服务返回 401，被映射成友好错误
    assert_eq!(error.code, "invalid_key", "实际错误：{error:?}");
    println!("真实网络验证通过：{} / {}", error.message, error.hint);
}

#[test]
#[ignore = "需要联网，用 cargo test -- --ignored 单独跑"]
fn real_openai_compatible_endpoints_are_reachable() {
    let transport = super::HttpTransport::new(20).unwrap();

    for provider in Provider::ALL {
        let endpoint = adapter_for(provider).endpoint;
        // 用一个必然无效的 Key 打过去：能拿到 401/403 就说明域名、路径、TLS 都通了
        let request = build_chat_request(
            &endpoint,
            provider.default_model(),
            "sk-invalid-key-for-reachability-check",
            &ChatRequest::new("ping"),
        );
        let response = block_on(transport.send(request));

        match response {
            Ok(response) => {
                println!(
                    "{} -> HTTP {} ({})",
                    provider.label(),
                    response.status,
                    super::openai_compat::parse_error_message(&response.body)
                );
                assert!(
                    response.status < 500,
                    "{} 返回了服务端错误 {}",
                    provider.label(),
                    response.status
                );
            }
            Err(error) => panic!("{} 连不上：{error:?}", provider.label()),
        }
    }
}

/* ------------------------------- 兼容性回归 ------------------------------- */

#[test]
fn endpoint_type_is_copy_so_adapters_can_be_passed_around() {
    fn take(_endpoint: Endpoint) {}
    let endpoint = adapter_for(Provider::Doubao).endpoint;
    take(endpoint);
    take(endpoint);
    assert!(endpoint.url.contains("volces.com"));
}

/* ------------------------- 端到端：走真实 SQLite 文件 ------------------------- */

fn temp_db_path(tag: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "media-ai-workbench-{tag}-{}-{:?}.db",
        std::process::id(),
        std::thread::current().id()
    ));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    path
}

fn open_db(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).expect("打开测试库失败");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    migrations::run(&conn).expect("迁移失败");
    conn
}

/// 复刻命令层里「已用免费次数」的口径
fn free_used_of(conn: &Connection) -> i64 {
    service::count(
        conn,
        "ai_usage",
        &[Filter {
            column: "is_free_trial".to_string(),
            op: "eq".to_string(),
            value: Some(json!(1)),
        }],
    )
    .unwrap()
}

fn record_usage(
    conn: &Connection,
    model: &str,
    tokens: i64,
    used_trial: bool,
) -> Result<(), String> {
    service::insert(
        conn,
        "ai_usage",
        &json!({
            "model": model,
            "type": "text",
            "tokens": tokens,
            "is_free_trial": used_trial,
        })
        .as_object()
        .cloned()
        .unwrap_or_default(),
    )
    .map(|_| ())
}

/// 验收标准 2/3/4/5 的完整走查：
/// 不填 Key 连用 3 次 → 第 4 次拦住 → 填自己的 Key 恢复 → 重启后一切还在。
#[test]
fn trial_then_user_key_flow_persists_across_restart() {
    let path = temp_db_path("flow");
    let secrets = Secrets::from_key([3u8; 32]);
    let trial = trial_config();
    let transport = MockTransport::ok(chat_reply("这是模型生成的文案", 50));

    // 首次启动：写入昵称和模型选择
    {
        let conn = open_db(&path);
        service::setting_set(&conn, "nickname", "小林").unwrap();
        service::setting_set(&conn, "ai_provider", "deepseek").unwrap();
    }

    // 没有 Key，前 3 次走体验额度
    for index in 0..3 {
        let outcome = {
            let conn = open_db(&path);
            let used = free_used_of(&conn);
            block_on(chat(
                Provider::DeepSeek,
                None,
                &trial,
                used,
                None,
                &transport,
                ChatRequest::new("写一句开场白"),
            ))
            .expect("体验额度内应该调用成功")
        };

        assert!(outcome.used_trial, "第 {} 次应该走体验 Key", index + 1);
        assert_eq!(outcome.response.content, "这是模型生成的文案");

        let conn = open_db(&path);
        record_usage(&conn, &outcome.response.model, outcome.response.tokens, true).unwrap();
        assert_eq!(free_used_of(&conn), index + 1);
    }

    // 第 4 次：额度用完，且在发网络请求之前就被拦住
    {
        let conn = open_db(&path);
        let used = free_used_of(&conn);
        assert_eq!(used, 3);

        let error = block_on(chat(
            Provider::DeepSeek,
            None,
            &trial,
            used,
            None,
            &transport,
            ChatRequest::new("第 4 次"),
        ))
        .unwrap_err();

        assert_eq!(error.code, "quota_exhausted");
        assert!(error.message.contains("免费体验额度已用完"));
    }

    // 用户填了自己的 Key（加密后落库）
    {
        let conn = open_db(&path);
        let encrypted = secrets.encrypt("sk-user-own-key-123456").unwrap();
        service::setting_set(&conn, "api_key:deepseek", &encrypted).unwrap();
    }

    // 重新打开数据库 = 模拟关掉软件再打开
    {
        let conn = open_db(&path);

        // 5. 昵称、模型选择、加密后的 Key、剩余次数都还在
        assert_eq!(
            service::setting_get(&conn, "nickname").unwrap().unwrap(),
            "小林"
        );
        assert_eq!(
            service::setting_get(&conn, "ai_provider").unwrap().unwrap(),
            "deepseek"
        );
        assert_eq!(free_used_of(&conn), 3, "免费次数应保留 3 条记录");

        let stored = service::setting_get(&conn, "api_key:deepseek")
            .unwrap()
            .unwrap();
        assert!(stored.starts_with("v1:"), "Key 应该是加密后的密文");
        assert!(!stored.contains("sk-user-own-key"), "库里不能出现明文 Key");

        let decrypted = secrets.decrypt(&stored).unwrap();
        assert_eq!(decrypted, "sk-user-own-key-123456");
        assert!(Secrets::mask(&decrypted).contains("••••"));

        // 4. 用自己的 Key 之后功能恢复，而且不再消耗体验额度
        let outcome = block_on(chat(
            Provider::DeepSeek,
            Some(&decrypted),
            &trial,
            3,
            None,
            &transport,
            ChatRequest::new("继续帮我写"),
        ))
        .unwrap();

        assert!(!outcome.used_trial);
        record_usage(&conn, &outcome.response.model, outcome.response.tokens, false).unwrap();

        assert_eq!(free_used_of(&conn), 3, "自己的 Key 不应该增加免费次数");
        assert_eq!(service::count(&conn, "ai_usage", &[]).unwrap(), 4, "总共 4 条调用记录");
    }

    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}

/// 调用失败（Key 无效 / 网络问题）不应该扣掉用户的免费次数。
#[test]
fn failed_calls_do_not_consume_the_free_quota() {
    let path = temp_db_path("failed");
    let trial = trial_config();
    let transport = MockTransport::status(401, json!({ "error": { "message": "invalid api key" } }));

    let conn = open_db(&path);

    let error = block_on(chat(
        Provider::DeepSeek,
        Some("sk-wrong-key-000000"),
        &trial,
        0,
        None,
        &transport,
        ChatRequest::new("你好"),
    ))
    .unwrap_err();

    assert_eq!(error.code, "invalid_key");
    // 命令层只在成功时落库，所以这里没有新记录
    assert_eq!(free_used_of(&conn), 0);
    assert_eq!(service::count(&conn, "ai_usage", &[]).unwrap(), 0);

    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
