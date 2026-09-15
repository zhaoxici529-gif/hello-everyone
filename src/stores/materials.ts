import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  parseMaterialTags,
  type AiErrorPayload,
  type Material,
  type MaterialCategory,
  type MaterialPurpose,
  type MaterialType,
} from '../types'
import {
  createMaterialCategory,
  deleteMaterial,
  deleteMaterialCategory,
  describeMaterial,
  importMaterials,
  materialCategories,
  materialLibraryInfo,
  materialPromptBlock,
  materialsRepo,
  pickMaterialFiles,
  saveMaterialThumbnail,
} from '../api/materials'
import { normalizeAiError } from '../api/invoke'
import { readJson, writeJson } from '../utils/storage'
import { localDateTime } from '../utils/datetime'
import { captureVideoFrame } from '../utils/video'

const STORAGE_KEY = 'materials'

/** 浏览器预览用的示例数据：只看界面，文件本体不存在 */
const LOCAL_SEEDS: Array<Partial<Material>> = [
  {
    name: '清晨窗边的封面图',
    type: 'image',
    tags: '封面,暖色',
    description: '清晨自然光下的书桌，适合做睡眠主题的封面。',
  },
  {
    name: '呼吸训练跟练',
    type: 'video',
    tags: '跟练,呼吸',
    description: '3 分钟呼吸训练示范，可以做视频里的插入素材。',
  },
  {
    name: '背景音乐·静谧',
    type: 'audio',
    tags: '音乐,安静',
    description: '低音量钢琴，适合放在口播开头。',
  },
  {
    name: '第 3 期学员访谈脚本',
    type: 'text',
    tags: '脚本,访谈',
    description: '学员小林的口述整理，含 5 个可用金句。',
  },
]

export const useMaterialsStore = defineStore('materials', () => {
  const all = ref<Material[]>([])
  const categories = ref<MaterialCategory[]>([])
  const libraryDirectory = ref('')
  const keyword = ref('')
  const typeFilter = ref<'all' | MaterialType>('all')
  const categoryFilter = ref<'all' | number>('all')
  const tagFilter = ref<string | null>(null)
  const view = ref<'grid' | 'list'>('grid')

  const loading = ref(false)
  const importing = ref(false)
  const describingId = ref<number | null>(null)
  const usingDatabase = ref(false)
  const error = ref<AiErrorPayload | null>(null)

  /* --------------------------------- 派生数据 -------------------------------- */

  const typeCounts = computed(() => {
    const result: Record<string, number> = { all: all.value.length }
    for (const type of ['image', 'video', 'audio', 'text'] as MaterialType[]) {
      result[type] = all.value.filter((item) => item.type === type).length
    }
    return result
  })

  /** 所有出现过的标签，按出现次数排序 */
  const allTags = computed(() => {
    const counter = new Map<string, number>()
    for (const item of all.value) {
      for (const tag of parseMaterialTags(item.tags)) {
        counter.set(tag, (counter.get(tag) ?? 0) + 1)
      }
    }
    return [...counter.entries()]
      .sort((first, second) => second[1] - first[1])
      .map(([tag, count]) => ({ tag, count }))
  })

  /** 分类筛选 + 标签筛选 + 关键词搜索 */
  const filtered = computed(() => {
    const text = keyword.value.trim().toLowerCase()

    return all.value.filter((item) => {
      if (typeFilter.value !== 'all' && item.type !== typeFilter.value) return false
      if (categoryFilter.value !== 'all' && item.category_id !== categoryFilter.value) return false

      const tags = parseMaterialTags(item.tags)
      if (tagFilter.value && !tags.includes(tagFilter.value)) return false

      if (!text) return true
      const haystack = [item.name, item.tags, item.type, item.description ?? '']
        .join(' ')
        .toLowerCase()
      return haystack.includes(text)
    })
  })

  /* ---------------------------------- 加载 ---------------------------------- */

  async function hydrate(): Promise<void> {
    loading.value = true

    try {
      categories.value = await materialCategories()
      libraryDirectory.value = (await materialLibraryInfo()).directory
    } catch {
      // 浏览器预览没有运行时，用本地兜底
    }

    const rows = await materialsRepo.list()

    if (rows) {
      usingDatabase.value = true
      all.value = rows
    } else {
      const cached = readJson<Material[]>(STORAGE_KEY, [])
      all.value = cached.length
        ? cached
        : LOCAL_SEEDS.map((seed, index) => localRow(seed, index))
    }

    loading.value = false
  }

  function localRow(seed: Partial<Material>, index: number): Material {
    return {
      id: -(index + 1),
      type: seed.type ?? 'text',
      name: seed.name ?? `素材 ${index + 1}`,
      file_path: null,
      tags: seed.tags ?? '',
      description: seed.description ?? null,
      created_at: localDateTime(),
      category_id: null,
      thumb_path: null,
      size_bytes: 0,
      source_path: null,
      ai_description_at: null,
    }
  }

  function persistLocal(): void {
    if (!usingDatabase.value) writeJson(STORAGE_KEY, all.value)
  }

  /* ---------------------------------- 导入 ---------------------------------- */

  /** 拖拽进来的路径 */
  async function importPaths(paths: string[], categoryId?: number | null): Promise<number> {
    if (!paths.length) return 0

    importing.value = true
    error.value = null

    try {
      const outcome = await importMaterials(paths, categoryId ?? null)
      all.value = [...outcome.imported, ...all.value]

      if (outcome.skipped.length) {
        error.value = {
          code: 'import_skipped',
          message: `${outcome.skipped.length} 个文件没能导入`,
          hint: outcome.skipped.map((item) => item.reason).join('；'),
        }
      }

      // 视频：抓首帧补缩略图
      await fillVideoThumbnails(outcome.imported)
      return outcome.imported.length
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return 0
    } finally {
      importing.value = false
    }
  }

  /** 点击「上传素材」：打开系统选择框再导入 */
  async function importViaPicker(categoryId?: number | null): Promise<number> {
    try {
      const paths = await pickMaterialFiles()
      if (!paths.length) return 0
      return await importPaths(paths, categoryId)
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return 0
    }
  }

  async function fillVideoThumbnails(items: Material[]): Promise<void> {
    for (const item of items) {
      if (item.type !== 'video' || item.thumb_path || !item.file_path) continue

      const dataUrl = await captureVideoFrame(item.file_path).catch(() => null)
      if (!dataUrl) continue

      const saved = await saveMaterialThumbnail(item.id, dataUrl).catch(() => null)
      if (saved) all.value = all.value.map((row) => (row.id === saved.id ? saved : row))
    }
  }

  /* ---------------------------------- 编辑 ---------------------------------- */

  async function updateMeta(
    id: number,
    patch: { tags?: string; description?: string; category_id?: number | null },
  ): Promise<void> {
    const saved = await materialsRepo.updateMeta(id, patch)

    if (saved) {
      all.value = all.value.map((row) => (row.id === id ? saved : row))
    } else {
      all.value = all.value.map((row) => (row.id === id ? { ...row, ...patch } : row))
    }
    persistLocal()
  }

  async function setTags(id: number, tags: string[]): Promise<void> {
    await updateMeta(id, { tags: tags.join(',') })
  }

  async function remove(id: number): Promise<void> {
    all.value = all.value.filter((row) => row.id !== id)
    persistLocal()
    if (usingDatabase.value && id > 0) await deleteMaterial(id, true)
  }

  /* ---------------------------------- 分类 ---------------------------------- */

  async function addCategory(name: string): Promise<MaterialCategory | null> {
    try {
      const created = await createMaterialCategory(name)
      categories.value = [...categories.value, created]
      return created
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    }
  }

  async function removeCategory(id: number): Promise<void> {
    categories.value = categories.value.filter((item) => item.id !== id)
    all.value = all.value.map((row) =>
      row.category_id === id ? { ...row, category_id: null } : row,
    )
    if (usingDatabase.value) await deleteMaterialCategory(id)
  }

  /* ------------------------------ 给别的模块用 ------------------------------ */

  async function promptBlock(id: number, purpose: MaterialPurpose = 'writing') {
    try {
      return await materialPromptBlock(id, purpose)
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    }
  }

  /** AI 生成描述 + 标签，会自动写回 */
  async function describe(id: number) {
    describingId.value = id
    error.value = null

    try {
      const result = await describeMaterial(id)
      all.value = all.value.map((row) => (row.id === id ? result.material : row))
      return result
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    } finally {
      describingId.value = null
    }
  }

  return {
    all,
    categories,
    libraryDirectory,
    keyword,
    typeFilter,
    categoryFilter,
    tagFilter,
    view,
    loading,
    importing,
    describingId,
    usingDatabase,
    error,
    typeCounts,
    allTags,
    filtered,
    hydrate,
    importPaths,
    importViaPicker,
    updateMeta,
    setTags,
    remove,
    addCategory,
    removeCategory,
    promptBlock,
    describe,
  }
})
