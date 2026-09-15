use super::error::AiError;
use super::provider::Provider;
use super::trial::TrialConfig;

/// 一次调用实际走哪把 Key。
#[derive(Debug, Clone)]
pub enum Route {
    /// 用户自己的 Key，用用户选的模型 / 供应商
    User { provider: Provider, key: String },
    /// 内置体验 Key
    Trial { provider: Provider, key: String },
}

impl Route {
    pub fn provider(&self) -> Provider {
        match self {
            Route::User { provider, .. } | Route::Trial { provider, .. } => *provider,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Route::User { key, .. } | Route::Trial { key, .. } => key,
        }
    }

    pub fn is_trial(&self) -> bool {
        matches!(self, Route::Trial { .. })
    }
}

/// 决定这次调用用哪把 Key。
///
/// 规则：
/// 1. 用户为当前选中的模型填了自己的 Key → 用自己的；
/// 2. 否则如果体验额度还没用完 → 走内置体验 Key（用体验配置里的供应商和模型）；
/// 3. 都没有 → 返回 quota_exhausted / trial_not_configured，让前端弹引导。
pub fn choose_route(
    selected: Provider,
    user_key: Option<&str>,
    trial: &TrialConfig,
    free_used: i64,
) -> Result<Route, AiError> {
    if let Some(key) = user_key {
        let key = key.trim();
        if !key.is_empty() {
            return Ok(Route::User {
                provider: selected,
                key: key.to_string(),
            });
        }
    }

    if !trial.is_configured() {
        // 没配体验 Key：如果额度也耗尽了，按额度用完引导，否则按「未配置」提示
        return Err(if free_used >= trial.quota() && trial.quota() > 0 {
            AiError::quota_exhausted(trial.quota())
        } else {
            AiError::trial_not_configured()
        });
    }

    if free_used >= trial.quota() {
        return Err(AiError::quota_exhausted(trial.quota()));
    }

    let provider = trial
        .provider()
        .ok_or_else(|| AiError::config("体验配置里的模型供应商无法识别"))?;

    Ok(Route::Trial {
        provider,
        key: trial.api_key.trim().to_string(),
    })
}
