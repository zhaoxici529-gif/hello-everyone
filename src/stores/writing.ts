import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type {
  AiErrorPayload,
  ChatTurn,
  GenreInfo,
  ImageSuggestion,
  Writing,
  WritingGenre,
  WritingStatus,
} from '../types'
import { reviseDraft, writeDraft, writingGenres, writingsRepo } from '../api/writing'
import { normalizeAiError } from '../api/invoke'
import { readJson, writeJson } from '../utils/storage'
import { localDateTime } from '../utils/datetime'

const STORAGE_KEY = 'writings'

const DEFAULT_GENRES: GenreInfo[] = [
  { id: 'xiaohongshu', label: '小红书' },
  { id: 'wechat', label: '公众号' },
  { id: 'script', label: '短视频脚本' },
  { id: 'moments', label: '朋友圈' },
]

/** 浏览器预览用的示例文案 */
const LOCAL_SEEDS: Writing[] = [
  {
    id: -1,
    title: '总是睡不好？先看看你的肩膀',
    content: '很多人以为自己睡不着是脑子太兴奋，其实先紧起来的是身体……',
    status: 'ready',
    persona_id: null,
    topic_id: null,
    images: '[]',
    genre: 'xiaohongshu',
    history: '[]',
    created_at: localDateTime(),
    updated_at: localDateTime(),
    published_at: null,
    version_history: '[]',
    deal_count: 0,
    lead_count: 0,
    private_count: 0,
    consult_count: 0,
    stats_updated_at: null,
  },
  {
    id: -2,
    title: '辅导作业前，先给自己 30 秒',
    content: '你一开口吼，孩子的脑子就关机了。这不是他不听话，是身体先自保……',
    status: 'draft',
    persona_id: null,
    topic_id: null,
    images: '[]',
    genre: 'wechat',
    history: '[]',
    created_at: localDateTime(),
    updated_at: localDateTime(),
    published_at: null,
    version_history: '[]',
    deal_count: 0,
    lead_count: 0,
    private_count: 0,
    consult_count: 0,
    stats_updated_at: null,
  },
]

function parseSuggestions(raw: string | null | undefined): ImageSuggestion[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? (parsed as ImageSuggestion[]) : []
  } catch {
    return []
  }
}

function parseHistory(raw: string | null | undefined): ChatTurn[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? (parsed as ChatTurn[]) : []
  } catch {
    return []
  }
}

export const useWritingStore = defineStore('writing', () => {
  const genres = ref<GenreInfo[]>(DEFAULT_GENRES)
  const library = ref<Writing[]>([])
  const usingDatabase = ref(false)

  /* ------------------------------- 白板输入区 ------------------------------- */

  const topic = ref('')
  const genre = ref<WritingGenre>('xiaohongshu')
  const personaId = ref<number | null>(null)
  const materialIds = ref<number[]>([])
  const extra = ref('')

  /* ------------------------------- 生成结果区 ------------------------------- */

  const generating = ref(false)
  const revising = ref(false)
  const saving = ref(false)
  const error = ref<AiErrorPayload | null>(null)

  const versions = ref<Array<{ angle: string; title: string; content: string }>>([])
  const activeIndex = ref(0)
  const history = ref<ChatTurn[]>([])
  const imageKeywords = ref<ImageSuggestion[]>([])
  const meta = ref<{ providerLabel: string; model: string; usedTrialKey: boolean } | null>(null)
  /** 已保存的文案 id，再次保存就是更新 */
  const savedId = ref<number | null>(null)

  const activeVersion = computed(() => versions.value[activeIndex.value] ?? null)
  const hasResult = computed(() => versions.value.length > 0)

  const recent = computed(() =>
    [...library.value]
      .sort((first, second) => second.updated_at.localeCompare(first.updated_at))
      .slice(0, 3),
  )

  /* ---------------------------------- 加载 ---------------------------------- */

  async function hydrate(): Promise<void> {
    try {
      genres.value = await writingGenres()
    } catch {
      // 浏览器预览用默认文体
    }

    const rows = await writingsRepo.list()
    if (rows) {
      usingDatabase.value = true
      library.value = rows
    } else {
      library.value = readJson<Writing[]>(STORAGE_KEY, LOCAL_SEEDS)
    }
  }

  function persistLocal(): void {
    if (!usingDatabase.value) writeJson(STORAGE_KEY, library.value)
  }

  /* ---------------------------------- 生成 ---------------------------------- */

  async function generate(): Promise<boolean> {
    if (!topic.value.trim()) {
      error.value = { code: 'config_error', message: '先写一个主题', hint: '例如「失眠调理」' }
      return false
    }

    generating.value = true
    error.value = null

    try {
      const result = await writeDraft({
        topic: topic.value,
        genre: genre.value,
        personaId: personaId.value,
        materialIds: materialIds.value,
        extra: extra.value || null,
      })

      versions.value = result.versions
      imageKeywords.value = result.imageKeywords
      activeIndex.value = 0
      history.value = []
      savedId.value = null
      meta.value = {
        providerLabel: result.providerLabel,
        model: result.model,
        usedTrialKey: result.usedTrialKey,
      }
      return true
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return false
    } finally {
      generating.value = false
    }
  }

  /* ------------------------------- 对话式修改 ------------------------------- */

  async function revise(instruction: string): Promise<boolean> {
    const current = activeVersion.value
    if (!current) {
      error.value = { code: 'config_error', message: '先生成一版文案', hint: '' }
      return false
    }

    const value = instruction.trim()
    if (!value) return false

    revising.value = true
    error.value = null

    const previous = [...history.value]

    try {
      const result = await reviseDraft({
        content: current.content,
        instruction: value,
        genre: genre.value,
        personaId: personaId.value,
        history: previous,
      })

      // 保留修改历史：一问一答都记下来，下次追问时一起发给模型
      history.value = [
        ...previous,
        { role: 'user', content: value },
        { role: 'assistant', content: result.content },
      ]

      versions.value = versions.value.map((item, index) =>
        index === activeIndex.value ? { ...item, content: result.content } : item,
      )
      meta.value = {
        providerLabel: result.providerLabel,
        model: result.model,
        usedTrialKey: result.usedTrialKey,
      }
      return true
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return false
    } finally {
      revising.value = false
    }
  }

  /* ---------------------------------- 保存 ---------------------------------- */

  async function save(status: WritingStatus = 'ready'): Promise<Writing | null> {
    const current = activeVersion.value
    if (!current) return null

    saving.value = true
    error.value = null

    const payload = {
      title: current.title.trim() || topic.value.trim() || '未命名文案',
      content: current.content,
      status,
      genre: genre.value,
      persona_id: personaId.value,
      images: JSON.stringify(imageKeywords.value),
      history: JSON.stringify(history.value),
    }

    try {
      const saved = savedId.value
        ? await writingsRepo.update(savedId.value, payload)
        : await writingsRepo.create(payload)

      if (!saved) throw new Error('保存失败')

      savedId.value = saved.id
      library.value = library.value.some((item) => item.id === saved.id)
        ? library.value.map((item) => (item.id === saved.id ? saved : item))
        : [saved, ...library.value]
      persistLocal()
      return saved
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    } finally {
      saving.value = false
    }
  }

  async function remove(id: number): Promise<void> {
    library.value = library.value.filter((item) => item.id !== id)
    persistLocal()
    if (usingDatabase.value && id > 0) await writingsRepo.remove(id)
  }

  /* -------------------------------- 载入已有 -------------------------------- */

  /** 继续编辑一篇已有文案 */
  function load(writing: Writing): void {
    topic.value = writing.title
    genre.value = (writing.genre as WritingGenre) || 'xiaohongshu'
    personaId.value = writing.persona_id
    savedId.value = writing.id > 0 ? writing.id : null
    extra.value = ''
    imageKeywords.value = parseSuggestions(writing.images)
    history.value = parseHistory(writing.history)
    versions.value = [{ angle: '已有文案', title: writing.title, content: writing.content }]
    activeIndex.value = 0
    meta.value = null
    error.value = null
  }

  function resetDraft(): void {
    topic.value = ''
    extra.value = ''
    materialIds.value = []
    versions.value = []
    imageKeywords.value = []
    history.value = []
    activeIndex.value = 0
    savedId.value = null
    meta.value = null
    error.value = null
  }

  return {
    genres,
    library,
    usingDatabase,
    topic,
    genre,
    personaId,
    materialIds,
    extra,
    generating,
    revising,
    saving,
    error,
    versions,
    activeIndex,
    history,
    imageKeywords,
    meta,
    savedId,
    activeVersion,
    hasResult,
    recent,
    hydrate,
    generate,
    revise,
    save,
    remove,
    load,
    resetDraft,
  }
})
