import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { AiErrorPayload, MemoryRow, MemorySource, MemoryStats } from '../types'
import {
  memoryDelete,
  memoryExport,
  memoryImport,
  memoryList,
  memoryReindex,
  memoryStats,
} from '../api/memory'
import { normalizeAiError } from '../api/invoke'

export const useMemoryStore = defineStore('memory', () => {
  const rows = ref<MemoryRow[]>([])
  const stats = ref<MemoryStats | null>(null)
  const keyword = ref('')
  const sourceFilter = ref<'all' | MemorySource>('all')
  const loading = ref(false)
  const busy = ref(false)
  const usingDatabase = ref(false)
  const error = ref<AiErrorPayload | null>(null)

  const filtered = computed(() => rows.value)

  async function load(): Promise<void> {
    loading.value = true
    error.value = null

    try {
      const sourceType = sourceFilter.value === 'all' ? null : sourceFilter.value
      rows.value = await memoryList({ sourceType, keyword: keyword.value })
      stats.value = await memoryStats()
      usingDatabase.value = true
    } catch (cause) {
      error.value = normalizeAiError(cause)
    } finally {
      loading.value = false
    }
  }

  async function reindex(): Promise<void> {
    busy.value = true
    try {
      stats.value = await memoryReindex()
      await load()
    } catch (cause) {
      error.value = normalizeAiError(cause)
    } finally {
      busy.value = false
    }
  }

  async function remove(id: number): Promise<void> {
    try {
      await memoryDelete(id)
      rows.value = rows.value.filter((row) => row.id !== id)
      stats.value = await memoryStats()
    } catch (cause) {
      error.value = normalizeAiError(cause)
    }
  }

  async function exportAll(): Promise<string | null> {
    busy.value = true
    try {
      return await memoryExport()
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    } finally {
      busy.value = false
    }
  }

  async function importAll(payload: string) {
    busy.value = true
    try {
      const result = await memoryImport(payload)
      await load()
      return result
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    } finally {
      busy.value = false
    }
  }

  return {
    rows,
    stats,
    keyword,
    sourceFilter,
    loading,
    busy,
    usingDatabase,
    error,
    filtered,
    load,
    reindex,
    remove,
    exportAll,
    importAll,
  }
})
