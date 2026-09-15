//! 通义千问（阿里云百炼）适配器，走 DashScope 的 OpenAI 兼容模式。
//! Key 申请地址：<https://bailian.console.aliyun.com/>

use super::super::openai_compat::Endpoint;

pub const ENDPOINT: Endpoint = Endpoint {
    url: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
    default_model: "qwen-plus",
};
