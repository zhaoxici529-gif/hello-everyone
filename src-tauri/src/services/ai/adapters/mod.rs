//! 一个模型一个 adapter。
//! 五家都提供 OpenAI 兼容的 chat/completions，所以公共逻辑在 openai_compat 里，
//! 每个文件只声明自己的接入点和默认模型。

pub mod deepseek;
pub mod doubao;
pub mod glm;
pub mod kimi;
pub mod qwen;

use super::openai_compat::Endpoint;
use super::provider::Provider;

#[derive(Debug, Clone, Copy)]
pub struct Adapter {
    /// 保留供应商信息，方便后续按供应商做差异化处理
    #[allow(dead_code)]
    pub provider: Provider,
    pub endpoint: Endpoint,
}

impl Adapter {
    pub fn default_model(&self) -> &'static str {
        self.endpoint.default_model
    }
}

pub fn adapter_for(provider: Provider) -> Adapter {
    let endpoint = match provider {
        Provider::DeepSeek => deepseek::ENDPOINT,
        Provider::Qwen => qwen::ENDPOINT,
        Provider::Doubao => doubao::ENDPOINT,
        Provider::Kimi => kimi::ENDPOINT,
        Provider::Glm => glm::ENDPOINT,
    };

    Adapter { provider, endpoint }
}
