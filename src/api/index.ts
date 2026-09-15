import { dbStatus } from './db'
import type { DbStatus } from '../types'

export * from './db'
export * from './repositories'
export * from './personas'

/**
 * 应用启动时调用：探测 Tauri 侧的 SQLite 是否就绪。
 * 返回 null 表示当前跑在浏览器预览里（`pnpm dev`），没有 Rust 运行时。
 */
export function initDatabase(): Promise<DbStatus | null> {
  return dbStatus()
}
