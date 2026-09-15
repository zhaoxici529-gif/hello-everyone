import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type {
  AiChatResponse,
  AiErrorPayload,
  AiSettings,
  AiTestResult,
  AiUsage,
  KeyStatus,
  ProviderInfo,
  QuotaInfo,
} from '../types'
import {
  aiChat,
  aiDeleteKey,
  aiProviders,
  aiQuota,
  aiSaveKey,
  aiSelect,
  aiSettings,
  aiTestConnection,
  aiUsageHistory,
  normalizeAiError,
  saveNickname,
} from '../api/ai'
import { readJson, writeJson } from '../utils/storage'

const NICKNAME_CACHE = 'nickname'

export const useAiStore = defineStore('ai', () => {
  const providers = ref<ProviderInfo[]>([])
  const settings = ref<AiSettings | null>(null)
  const quota = ref<QuotaInfo | null>(null)
  const usage = ref<AiUsage[]>([])
  const nickname = ref(readJson<string>(NICKNAME_CACHE, ''))

  const selectedProvider = ref('deepseek')
  const selectedModel = ref('')

  const loading = ref(false)
  const testing = ref(false)
  const busy = ref(false)
  const ready = ref(false)

  const testResult = ref<AiTestResult | null>(null)
  const lastError = ref<AiErrorPayload | null>(null)
  const lastReply = ref<AiChatResponse | null>(null)
  /** 额度用完时弹的引导框 */
  const showQuotaGuide = ref(false)

  const currentProvider = computed(
    () => providers.value.find((item) => item.id === selectedProvider.value) ?? null,
  )
  const keys = computed<KeyStatus[]>(() => settings.value?.keys ?? [])
  const currentKey = computed(
    () => keys.value.find((item) => item.provider === selectedProvider.value) ?? null,
  )
  const hasAnyKey = computed(() => keys.value.some((item) => item.hasKey))
  const remaining = computed(() => quota.value?.remaining ?? 0)
  const quotaLimit = computed(() => quota.value?.limit ?? 0)
  const quotaUsed = computed(() => quota.value?.used ?? 0)
  const quotaPercent = computed(() =>
    quotaLimit.value === 0 ? 0 : Math.round((quotaUsed.value / quotaLimit.value) * 100),
  )

  /* ---------------------------------- 加载 ---------------------------------- */

  async function load(): Promise<void> {
    if (loading.value) return
    loading.value = true

    try {
      providers.value = await aiProviders()
      await refresh()
      await refreshQuotaAndUsage()
      ready.value = true
      lastError.value = null
    } catch (error) {
      lastError.value = normalizeAiError(error)
    } finally {
      loading.value = false
    }
  }

  async function refresh(): Promise<void> {
    const data = await aiSettings()
    settings.value = data
    quota.value = data.quota
    selectedProvider.value = data.selectedProvider
    selectedModel.value =
      data.selectedModel ||
      providers.value.find((item) => item.id === data.selectedProvider)?.defaultModel ||
      ''

    if (data.nickname && data.nickname !== nickname.value) {
      nickname.value = data.nickname
      writeJson(NICKNAME_CACHE, data.nickname)
    }
  }

  async function refreshQuotaAndUsage(): Promise<void> {
    try {
      quota.value = await aiQuota()
      usage.value = await aiUsageHistory(8)
    } catch {
      // 浏览器预览下没有运行时，静默忽略
    }
  }

  /* ---------------------------------- 模型 ---------------------------------- */

  async function selectProvider(provider: string): Promise<void> {
    selectedProvider.value = provider
    selectedModel.value =
      providers.value.find((item) => item.id === provider)?.defaultModel ?? ''
    testResult.value = null
    lastError.value = null
    try {
      await aiSelect(provider, selectedModel.value)
    } catch (error) {
      lastError.value = normalizeAiError(error)
    }
  }

  async function selectModel(model: string): Promise<void> {
    selectedModel.value = model
    try {
      await aiSelect(selectedProvider.value, model)
    } catch (error) {
      lastError.value = normalizeAiError(error)
    }
  }

  /* ---------------------------------- 密钥 ---------------------------------- */

  async function saveKey(provider: string, apiKey: string): Promise<boolean> {
    try {
      await aiSaveKey(provider, apiKey)
      await refresh()
      return true
    } catch (error) {
      lastError.value = normalizeAiError(error)
      return false
    }
  }

  async function removeKey(provider: string): Promise<boolean> {
    try {
      await aiDeleteKey(provider)
      await refresh()
      return true
    } catch (error) {
      lastError.value = normalizeAiError(error)
      return false
    }
  }

  /** 测试连接：传 key 就先测后存，不传就测已保存的 */
  async function testConnection(provider: string, apiKey?: string): Promise<AiTestResult | null> {
    testing.value = true
    testResult.value = null
    lastError.value = null

    try {
      const result = await aiTestConnection(provider, apiKey)
      testResult.value = result
      return result
    } catch (error) {
      lastError.value = normalizeAiError(error)
      return null
    } finally {
      testing.value = false
    }
  }

  /* ---------------------------------- 昵称 ---------------------------------- */

  async function persistNickname(value: string): Promise<boolean> {
    try {
      nickname.value = await saveNickname(value)
      writeJson(NICKNAME_CACHE, nickname.value)
      return true
    } catch (error) {
      lastError.value = normalizeAiError(error)
      return false
    }
  }

  /* ---------------------------------- 调用 ---------------------------------- */

  async function run(prompt: string, options: { system?: string } = {}): Promise<AiChatResponse | null> {
    busy.value = true
    lastError.value = null

    try {
      const response = await aiChat({
        prompt,
        system: options.system,
        model: selectedModel.value || undefined,
      })

      lastReply.value = response
      await refreshQuotaAndUsage()
      return response
    } catch (error) {
      const payload = normalizeAiError(error)
      lastError.value = payload
      // 额度用完 → 弹图文引导
      if (payload.code === 'quota_exhausted') showQuotaGuide.value = true
      return null
    } finally {
      busy.value = false
    }
  }

  function dismissError(): void {
    lastError.value = null
  }

  return {
    providers,
    settings,
    quota,
    usage,
    nickname,
    selectedProvider,
    selectedModel,
    loading,
    testing,
    busy,
    ready,
    testResult,
    lastError,
    lastReply,
    showQuotaGuide,
    currentProvider,
    keys,
    currentKey,
    hasAnyKey,
    remaining,
    quotaLimit,
    quotaUsed,
    quotaPercent,
    load,
    refresh,
    refreshQuotaAndUsage,
    selectProvider,
    selectModel,
    saveKey,
    removeKey,
    testConnection,
    persistNickname,
    run,
    dismissError,
  }
})
