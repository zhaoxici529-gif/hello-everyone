use serde::{Deserialize, Serialize};

/// 对话式修改里的一条往返消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    /// user 或 assistant
    pub role: String,
    pub content: String,
}

/// 统一的文本生成入参。各模型 adapter 都接收这个结构。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChatRequest {
    pub prompt: String,
    /// 系统提示词（人设 / 品牌口吻）
    pub system: Option<String>,
    /// 不传就用该模型的默认模型名
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    /// 多模态输入：`data:image/jpeg;base64,...` 形式的图片列表。
    /// 只有支持视觉的模型能用，其余模型会返回友好错误。
    pub images: Vec<String>,
    /// 之前的对话轮次，用于「再口语化一点」这类追问
    pub history: Vec<ChatTurn>,
}

impl ChatRequest {
    // 便捷构造函数目前只在测试里用，保留给后续接入的业务代码
    #[allow(dead_code)]
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }
}

/// 统一的文本生成出参。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    /// 实际使用的模型供应商 id
    pub provider: String,
    pub provider_label: String,
    /// 实际使用的模型名
    pub model: String,
    pub content: String,
    /// 本次消耗的 token（供应商没返回就是 0）
    pub tokens: i64,
    /// 本次是否走的体验 Key
    pub used_trial_key: bool,
    /// 剩余免费次数
    pub remaining_free: i64,
    pub latency_ms: u64,
}

/// 图片生成入参（预留，暂未接入具体模型）
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRequest {
    pub prompt: String,
    pub size: Option<String>,
}

/// 图片生成出参（预留）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub provider: String,
    /// 图片地址或 base64
    pub images: Vec<String>,
    pub used_trial_key: bool,
}
