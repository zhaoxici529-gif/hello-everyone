import type {
  MemoryImportResult,
  MemoryRow,
  MemorySource,
  MemoryStats,
} from '../types'
import { invokeCommand } from './invoke'

export function memoryList(options: {
  sourceType?: MemorySource | null
  keyword?: string
  limit?: number
} = {}): Promise<MemoryRow[]> {
  return invokeCommand<MemoryRow[]>('memory_list', {
    sourceType: options.sourceType ?? null,
    keyword: options.keyword ?? null,
    limit: options.limit ?? 200,
  })
}

/** 给其它模块用：按查询取相关记忆片段 */
export function memorySearch(
  query: string,
  sourceTypes: MemorySource[] = [],
  limit = 5,
): Promise<MemoryRow[]> {
  return invokeCommand<MemoryRow[]>('memory_search', { query, sourceTypes, limit })
}

export function memoryStats(): Promise<MemoryStats> {
  return invokeCommand<MemoryStats>('memory_stats')
}

export function memoryDelete(id: number): Promise<boolean> {
  return invokeCommand<boolean>('memory_delete', { id })
}

export function memoryReindex(): Promise<MemoryStats> {
  return invokeCommand<MemoryStats>('memory_reindex')
}

export function memoryExport(): Promise<string> {
  return invokeCommand<string>('memory_export')
}

export function memoryImport(payload: string): Promise<MemoryImportResult> {
  return invokeCommand<MemoryImportResult>('memory_import', { payload })
}
