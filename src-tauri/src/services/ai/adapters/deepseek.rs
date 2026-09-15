//! DeepSeek 适配器。OpenAI 兼容，鉴权头 `Authorization: Bearer sk-xxx`。
//! Key 申请地址：<https://platform.deepseek.com/api_keys>

use super::super::openai_compat::Endpoint;

pub const ENDPOINT: Endpoint = Endpoint {
    url: "https://api.deepseek.com/v1/chat/completions",
    default_model: "deepseek-chat",
};
