import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { DbStatus } from '../types'

export type DatabaseStatus = 'idle' | 'ready' | 'browser' | 'error'

export const useAppStore = defineStore('app', () => {
  const version = ref('0.1.0')
  const databaseStatus = ref<DatabaseStatus>('idle')
  const databasePath = ref('')
  const databaseDirectory = ref('')
  const databaseVersion = ref('')
  const schemaVersion = ref(0)
  const tableCount = ref(0)

  /** 顶部导航右侧状态胶囊的文案 */
  const databaseText = computed(() => {
    switch (databaseStatus.value) {
      case 'ready':
        return '本地数据库已就绪'
      case 'browser':
        return '浏览器预览模式'
      case 'error':
        return '数据库连接失败'
      default:
        return '正在连接本地数据库'
    }
  })

  const isDatabaseReady = computed(() => databaseStatus.value === 'ready')

  function markReady(status: DbStatus): void {
    databaseStatus.value = 'ready'
    databasePath.value = status.path
    databaseDirectory.value = status.directory
    databaseVersion.value = status.version
    schemaVersion.value = status.schemaVersion
    tableCount.value = status.tableCount
  }

  function markBrowser(): void {
    databaseStatus.value = 'browser'
  }

  function markError(message = ''): void {
    databaseStatus.value = 'error'
    databasePath.value = message
  }

  return {
    version,
    databaseStatus,
    databasePath,
    databaseDirectory,
    databaseVersion,
    schemaVersion,
    tableCount,
    databaseText,
    isDatabaseReady,
    markReady,
    markBrowser,
    markError,
  }
})
