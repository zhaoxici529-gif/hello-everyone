//! 智谱 GLM 适配器（BigModel 开放平台 v4）。
//! Key 申请地址：<https://open.bigmodel.cn/usercenter/apikeys>

use super::super::openai_compat::Endpoint;

pub const ENDPOINT: Endpoint = Endpoint {
    url: "https://open.bigmodel.cn/api/paas/v4/chat/completions",
    default_model: "glm-4-flash",
};
