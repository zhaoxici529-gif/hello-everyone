import type {
  AiChatResponse,
  AiSettings,
  AiTestResult,
  AiUsage,
  ProviderInfo,
  QuotaInfo,
} from '../types'
import { invokeCommand } from './invoke'

export { isAiError, normalizeAiError } from './invoke'

const invokeAi = invokeCommand

/* ---------------------------------- 元信息 ---------------------------------- */

export function aiProviders(): Promise<ProviderInfo[]> {
  return invokeAi<ProviderInfo[]>('ai_providers')
}

export function aiSettings(): Promise<AiSettings> {
  return invokeAi<AiSettings>('ai_settings')
}

export function aiQuota(): Promise<QuotaInfo> {
  return invokeAi<QuotaInfo>('ai_quota')
}

export function aiUsageHistory(limit = 10): Promise<AiUsage[]> {
  return invokeAi<AiUsage[]>('ai_usage_history', { limit })
}

/* ---------------------------------- 密钥 ---------------------------------- */

export function aiSaveKey(provider: string, apiKey: string): Promise<AiSettings['keys'][number]> {
  return invokeAi('ai_save_key', { provider, apiKey })
}

export function aiDeleteKey(provider: string): Promise<AiSettings['keys'][number]> {
  return invokeAi('ai_delete_key', { provider })
}

export function aiSelect(provider: string, model?: string): Promise<void> {
  return invokeAi<void>('ai_select', { provider, model: model ?? '' })
}

export function saveNickname(nickname: string): Promise<string> {
  return invokeAi<string>('save_nickname', { nickname })
}

/* ---------------------------------- 调用 ---------------------------------- */

/** 测试连接：带 apiKey 就先测再存；不带走已保存的 */
export function aiTestConnection(provider: string, apiKey?: string): Promise<AiTestResult> {
  return invokeAi<AiTestResult>('ai_test_connection', { provider, apiKey })
}

export function aiChat(payload: {
  prompt: string
  system?: string
  model?: string
  temperature?: number
  maxTokens?: number
}): Promise<AiChatResponse> {
  return invokeAi<AiChatResponse>('ai_chat', { request: payload })
}
