use serde::Serialize;

/// 已接入的模型供应商。五家都提供 OpenAI 兼容的 chat/completions 接口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    DeepSeek,
    Qwen,
    Doubao,
    Kimi,
    Glm,
}

impl Provider {
    pub const ALL: [Provider; 5] = [
        Provider::DeepSeek,
        Provider::Qwen,
        Provider::Doubao,
        Provider::Kimi,
        Provider::Glm,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Provider::DeepSeek => "deepseek",
            Provider::Qwen => "qwen",
            Provider::Doubao => "doubao",
            Provider::Kimi => "kimi",
            Provider::Glm => "glm",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::DeepSeek => "DeepSeek",
            Provider::Qwen => "通义千问",
            Provider::Doubao => "豆包",
            Provider::Kimi => "Kimi",
            Provider::Glm => "智谱 GLM",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let normalized = id.trim().to_ascii_lowercase();
        Provider::ALL
            .into_iter()
            .find(|provider| provider.id() == normalized)
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Provider::DeepSeek => "deepseek-chat",
            Provider::Qwen => "qwen-plus",
            Provider::Doubao => "doubao-pro-32k",
            Provider::Kimi => "moonshot-v1-8k",
            Provider::Glm => "glm-4-flash",
        }
    }

    /// 可选模型，用于设置页的下拉
    pub fn models(self) -> &'static [&'static str] {
        match self {
            Provider::DeepSeek => &["deepseek-chat", "deepseek-reasoner"],
            Provider::Qwen => &["qwen-plus", "qwen-max", "qwen-turbo", "qwen-vl-plus", "qwen-vl-max"],
            Provider::Doubao => &[
                "doubao-pro-32k",
                "doubao-pro-128k",
                "doubao-lite-32k",
                "doubao-vision-pro-32k",
            ],
            Provider::Kimi => &[
                "moonshot-v1-8k",
                "moonshot-v1-32k",
                "moonshot-v1-128k",
                "moonshot-v1-8k-vision-preview",
            ],
            Provider::Glm => &["glm-4-flash", "glm-4-air", "glm-4-plus", "glm-4v-flash", "glm-4v-plus"],
        }
    }

    /// 支持「看图」的模型。素材描述功能要配合这些模型使用。
    pub fn vision_models(self) -> &'static [&'static str] {
        match self {
            Provider::DeepSeek => &[],
            Provider::Qwen => &["qwen-vl-plus", "qwen-vl-max"],
            Provider::Doubao => &["doubao-vision-pro-32k"],
            Provider::Kimi => &["moonshot-v1-8k-vision-preview"],
            Provider::Glm => &["glm-4v-flash", "glm-4v-plus"],
        }
    }

    /// Key 申请页（给不熟技术的用户直接点）
    pub fn console_url(self) -> &'static str {
        match self {
            Provider::DeepSeek => "https://platform.deepseek.com/api_keys",
            Provider::Qwen => "https://bailian.console.aliyun.com/",
            Provider::Doubao => "https://console.volcengine.com/ark",
            Provider::Kimi => "https://platform.moonshot.cn/console/api-keys",
            Provider::Glm => "https://open.bigmodel.cn/usercenter/apikeys",
        }
    }

    /// 这个 Key 长什么样，帮用户在页面上认出来
    pub fn key_prefix(self) -> &'static str {
        match self {
            Provider::DeepSeek => "sk-",
            Provider::Qwen => "sk-",
            Provider::Doubao => "通常是一串字母数字",
            Provider::Kimi => "sk-",
            Provider::Glm => "形如 xxxx.yyyy",
        }
    }
}

/// 返回给前端的供应商元信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub label: String,
    pub default_model: String,
    pub models: Vec<String>,
    /// 其中支持看图的模型
    pub vision_models: Vec<String>,
    pub console_url: String,
    pub key_prefix: String,
}

impl From<Provider> for ProviderInfo {
    fn from(provider: Provider) -> Self {
        Self {
            id: provider.id().to_string(),
            label: provider.label().to_string(),
            default_model: provider.default_model().to_string(),
            models: provider.models().iter().map(|m| m.to_string()).collect(),
            vision_models: provider
                .vision_models()
                .iter()
                .map(|m| m.to_string())
                .collect(),
            console_url: provider.console_url().to_string(),
            key_prefix: provider.key_prefix().to_string(),
        }
    }
}
