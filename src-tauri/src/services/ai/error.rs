use serde::Serialize;

impl std::fmt::Display for AiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "[{}] {}（{}）",
            self.code, self.message, self.hint
        )
    }
}

impl std::error::Error for AiError {}

/// 统一的 AI 错误。序列化后前端按 `code` 分支，`message` / `hint` 直接展示给用户。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiError {
    pub code: String,
    pub message: String,
    pub hint: String,
}

impl AiError {
    pub fn new(code: &str, message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            hint: hint.into(),
        }
    }

    /// Key 无效 / 没有权限
    pub fn invalid_key(detail: impl Into<String>) -> Self {
        Self::new(
            "invalid_key",
            format!("API Key 无效或没有权限：{}", detail.into()),
            "请到服务商官网重新复制一次 Key，注意不要带多余空格。",
        )
    }

    pub fn missing_key() -> Self {
        Self::new(
            "missing_key",
            "还没有可用的 API Key",
            "在设置页选一个模型并填入自己的 API Key，或先使用内置的免费体验额度。",
        )
    }

    pub fn network(detail: impl Into<String>) -> Self {
        Self::new(
            "network_error",
            format!("网络连接失败：{}", detail.into()),
            "请检查网络或代理设置，稍后重试。",
        )
    }

    pub fn rate_limited() -> Self {
        Self::new(
            "rate_limited",
            "请求太频繁或额度被服务商限制",
            "等几十秒再试；如果反复出现，去服务商后台看看余额和限流设置。",
        )
    }

    /// 账户余额不足
    pub fn insufficient_balance(detail: impl Into<String>) -> Self {
        Self::new(
            "insufficient_balance",
            format!("账户余额不足：{}", detail.into()),
            "去服务商后台充值，或在设置页换一个模型 / 填另一家的 API Key。",
        )
    }

    /// 免费体验次数用完
    pub fn quota_exhausted(limit: i64) -> Self {
        Self::new(
            "quota_exhausted",
            format!("免费体验额度已用完（共 {limit} 次）"),
            "填写自己的 API Key 后可以继续无限使用。",
        )
    }

    /// 体验 Key 没配置
    pub fn trial_not_configured() -> Self {
        Self::new(
            "trial_not_configured",
            "内置体验 Key 尚未配置",
            "请在 src-tauri/config/trial.json 里填入体验 Key，或直接在设置页填写自己的 API Key。",
        )
    }

    pub fn provider(status: u16, detail: impl Into<String>) -> Self {
        Self::new(
            "provider_error",
            format!("模型服务返回错误（HTTP {status}）：{}", detail.into()),
            "换个模型或稍后重试；持续失败请检查账户余额与模型权限。",
        )
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new("config_error", message, "请检查设置页的模型配置。")
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new("internal_error", detail, "可以重试一次；如果一直失败请反馈。")
    }

    pub fn is_quota_exhausted(&self) -> bool {
        self.code == "quota_exhausted"
    }
}
