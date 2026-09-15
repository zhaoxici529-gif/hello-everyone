import type {
  AiUsage,
  AiUsageSummary,
  DbStatus,
  Filter,
  ListQuery,
  SelfTestResult,
  SettingRow,
  TableInfo,
  TableName,
} from '../types'

/**
 * 是否运行在 Tauri 容器里。
 * 浏览器里跑 `pnpm dev` 时没有注入 `__TAURI_INTERNALS__`，所有 command 调用
 * 都会返回 null，由 store 退回到 localStorage。
 */
export function hasTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!hasTauri()) return null
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(command, args)
  } catch (error) {
    console.warn(`[db] 调用 ${command} 失败：`, error)
    return null
  }
}

/* --------------------------------- 状态与结构 -------------------------------- */

export function dbStatus(): Promise<DbStatus | null> {
  return call<DbStatus>('db_status')
}

/** 所有业务表的结构 + 行数 */
export function dbDescribe(): Promise<TableInfo[] | null> {
  return call<TableInfo[]>('db_describe')
}

/* --------------------------------- 通用 CRUD -------------------------------- */

export function dbList<T>(table: TableName, query?: ListQuery): Promise<T[] | null> {
  return call<T[]>('db_list', { table, query: query ?? {} })
}

/** 找不到记录时返回 null */
export function dbGet<T>(table: TableName, id: number): Promise<T | null> {
  return call<T | null>('db_get', { table, id })
}

export function dbInsert<T>(table: TableName, data: Record<string, unknown>): Promise<T | null> {
  return call<T>('db_insert', { table, data })
}

export function dbUpdate<T>(
  table: TableName,
  id: number,
  data: Record<string, unknown>,
): Promise<T | null> {
  return call<T>('db_update', { table, id, data })
}

export function dbDelete(table: TableName, id: number): Promise<boolean | null> {
  return call<boolean>('db_delete', { table, id })
}

export function dbCount(table: TableName, filters?: Filter[]): Promise<number | null> {
  return call<number>('db_count', { table, filters: filters ?? [] })
}

/* ---------------------------------- 配置项 ---------------------------------- */

export function settingGet(key: string): Promise<string | null> {
  return call<string | null>('setting_get', { key })
}

export function settingSet(key: string, value: string): Promise<string | null> {
  return call<string>('setting_set', { key, value })
}

export function settingAll(): Promise<SettingRow[] | null> {
  return call<SettingRow[]>('setting_all')
}

/* --------------------------------- AI 用量 --------------------------------- */

export function aiUsageRecord(payload: {
  model: string
  kind: 'text' | 'image'
  tokens?: number
  isFreeTrial?: boolean
}): Promise<AiUsage | null> {
  return call<AiUsage>('ai_usage_record', payload)
}

export function aiUsageSummary(): Promise<AiUsageSummary | null> {
  return call<AiUsageSummary>('ai_usage_summary')
}

/* ---------------------------------- 自检 ---------------------------------- */

/** 写一条 notes 再读回来，验证整条链路 */
export function dbSelfTest(): Promise<SelfTestResult | null> {
  return call<SelfTestResult>('db_self_test')
}
