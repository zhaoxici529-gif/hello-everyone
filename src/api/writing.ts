import type {
  ChatTurn,
  GenreInfo,
  ImageSuggestion,
  ReviseResult,
  WriteResult,
  Writing,
  WritingExport,
  WritingGenre,
  WritingStatus,
  WritingSnapshot,
} from '../types'
import { dbDelete, dbInsert, dbList, dbUpdate } from './db'
import { invokeCommand } from './invoke'

export interface WritingPayload {
  title: string
  content: string
  status: WritingStatus
  genre: string
  persona_id: number | null
  images: string
  history: string
  published_at?: string | null
  version_history?: string
  deal_count?: number
  lead_count?: number
  private_count?: number
  consult_count?: number
  stats_updated_at?: string | null
}

export const writingsRepo = {
  list(): Promise<Writing[] | null> {
    return dbList<Writing>('writings', { orderBy: 'updated_at', desc: true })
  },

  create(payload: WritingPayload): Promise<Writing | null> {
    return dbInsert<Writing>('writings', { ...payload })
  },

  update(id: number, payload: Partial<WritingPayload>): Promise<Writing | null> {
    return dbUpdate<Writing>('writings', id, payload)
  },

  remove(id: number): Promise<boolean | null> {
    return dbDelete('writings', id)
  },
}

export function writingGenres(): Promise<GenreInfo[]> {
  return invokeCommand<GenreInfo[]>('writing_genres')
}

/** 一次生成 3 个版本 + 配图建议 */
export function writeDraft(payload: {
  topic: string
  genre?: WritingGenre
  personaId?: number | null
  materialIds?: number[]
  extra?: string | null
}): Promise<WriteResult> {
  return invokeCommand<WriteResult>('ai_write_draft', {
    request: {
      topic: payload.topic,
      genre: payload.genre,
      personaId: payload.personaId ?? null,
      materialIds: payload.materialIds ?? [],
      extra: payload.extra ?? null,
    },
  })
}

/** 对话式修改 */
export function reviseDraft(payload: {
  content: string
  instruction: string
  genre?: WritingGenre
  personaId?: number | null
  history: ChatTurn[]
}): Promise<ReviseResult> {
  return invokeCommand<ReviseResult>('ai_revise_draft', {
    request: {
      content: payload.content,
      instruction: payload.instruction,
      genre: payload.genre,
      personaId: payload.personaId ?? null,
      history: payload.history,
    },
  })
}

/** 配图建议 → 可复制的图片提示词 */
export function suggestionToPrompt(suggestion: ImageSuggestion, title = ''): string {
  return [
    suggestion.keywords,
    suggestion.style ? `${suggestion.style}风格` : '',
    title ? `主题：${title}` : '',
  ]
    .filter(Boolean)
    .join('，')
}

/* ------------------------------ 版本历史与导出 ------------------------------ */

export function parseSnapshots(raw: string | null | undefined): WritingSnapshot[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? (parsed as WritingSnapshot[]) : []
  } catch {
    return []
  }
}

/** 保存前把当前内容压进历史，只保留最近 5 个版本 */
export function pushSnapshot(
  raw: string | null | undefined,
  title: string,
  content: string,
): string {
  const stamp = new Date()
    .toLocaleString('zh-CN', { hour12: false })
    .replaceAll('/', '-')

  const next: WritingSnapshot[] = [
    { title, content, savedAt: stamp },
    ...parseSnapshots(raw),
  ].slice(0, 5)

  return JSON.stringify(next)
}

/** 导出文案：返回文件名与内容，前端落成文件 */
export function exportWriting(id: number, format: 'txt' | 'md'): Promise<WritingExport> {
  return invokeCommand<WritingExport>('writing_export', { id, format })
}

/** 触发浏览器下载 */
export function downloadText(filename: string, content: string): void {
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  document.body.appendChild(anchor)
  anchor.click()
  anchor.remove()
  URL.revokeObjectURL(url)
}

const STATUS_ORDER: WritingStatus[] = ['draft', 'ready', 'published']

/** 一键切换状态：草稿 → 待发 → 已发 → 草稿 */
export function nextStatus(current: WritingStatus): WritingStatus {
  const index = STATUS_ORDER.indexOf(current)
  return STATUS_ORDER[(index + 1) % STATUS_ORDER.length]
}
