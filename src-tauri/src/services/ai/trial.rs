use serde::{Deserialize, Serialize};

use super::provider::Provider;

/// 内置体验 Key。改 `src-tauri/config/trial.json` 就行，不用动代码。
const TRIAL_CONFIG: &str = include_str!("../../../config/trial.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrialConfig {
    pub enabled: bool,
    pub free_quota: i64,
    pub provider: String,
    pub model: String,
    pub api_key: String,
}

impl Default for TrialConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            free_quota: 3,
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: String::new(),
        }
    }
}

impl TrialConfig {
    /// 读取编译进程序的体验配置。解析失败就退化成「未开启」。
    pub fn load() -> Self {
        serde_json::from_str(TRIAL_CONFIG).unwrap_or_default()
    }

    pub fn provider(&self) -> Option<Provider> {
        Provider::from_id(&self.provider)
    }

    /// 体验 Key 是否真的填了。
    /// 占位值（含「填入」两字）或长度不够都算没配，避免用户拿到莫名其妙的 401。
    pub fn is_configured(&self) -> bool {
        self.enabled
            && self.provider().is_some()
            && self.api_key.len() >= 20
            && !self.api_key.contains("填入")
            && !self.api_key.contains("REPLACE")
    }

    pub fn quota(&self) -> i64 {
        self.free_quota.max(0)
    }
}

/// 给前端看的体验额度信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaInfo {
    pub limit: i64,
    pub used: i64,
    pub remaining: i64,
    pub configured: bool,
    pub provider: String,
    pub provider_label: String,
    pub model: String,
}

impl QuotaInfo {
    pub fn build(trial: &TrialConfig, used: i64) -> Self {
        let limit = trial.quota();
        let provider = trial.provider();

        Self {
            limit,
            used,
            remaining: (limit - used).max(0),
            configured: trial.is_configured(),
            provider: provider.map(|p| p.id().to_string()).unwrap_or_default(),
            provider_label: provider.map(|p| p.label().to_string()).unwrap_or_default(),
            model: trial.model.clone(),
        }
    }

}
