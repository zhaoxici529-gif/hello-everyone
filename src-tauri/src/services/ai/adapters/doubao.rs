//! 豆包（火山方舟 Ark）适配器。
//! Key 申请地址：<https://console.volcengine.com/ark>
//! 注意：火山方舟通常要求用「推理接入点 ID」(ep-xxxx) 作为模型名，
//! 如果报模型不存在，把设置页的模型名换成自己的接入点 ID。

use super::super::openai_compat::Endpoint;

pub const ENDPOINT: Endpoint = Endpoint {
    url: "https://ark.cn-beijing.volces.com/api/v3/chat/completions",
    default_model: "doubao-pro-32k",
};
