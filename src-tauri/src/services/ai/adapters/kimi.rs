//! Kimi（月之暗面 Moonshot）适配器。
//! Key 申请地址：<https://platform.moonshot.cn/console/api-keys>

use super::super::openai_compat::Endpoint;

pub const ENDPOINT: Endpoint = Endpoint {
    url: "https://api.moonshot.cn/v1/chat/completions",
    default_model: "moonshot-v1-8k",
};
