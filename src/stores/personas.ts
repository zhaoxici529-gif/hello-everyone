import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  emptyPersonaFields,
  parsePersonaFields,
  type Persona,
  type PersonaCategory,
  type PersonaCategoryInfo,
  type PersonaExtraction,
  type PersonaFields,
  type PersonaPromptBlock,
  type PersonaPurpose,
} from '../types'
import {
  aiExtractPersona,
  personaCategories,
  personaPromptBlock,
  personasRepo,
  type PersonaPayload,
} from '../api/personas'
import { normalizeAiError } from '../api/invoke'
import type { AiErrorPayload } from '../types'
import { readJson, writeJson } from '../utils/storage'
import { localDateTime } from '../utils/datetime'

const STORAGE_KEY = 'personas'

const SEEDS: PersonaPayload[] = [
  {
    name: '我自己（心身同调主理人）',
    category: 'self_ip',
    fields: {
      ...emptyPersonaFields(),
      age: '38 岁',
      identity: '心身同调训练法主理人、两个孩子的妈妈',
      experience:
        '做了 12 年中学心理老师，因为自己经历过连续两年的失眠和焦虑，开始系统研究呼吸与身体感受，后来把方法整理成课程。',
      traits: ['温和', '直接', '爱举例', '不说客套话'],
      catchphrases: '你先别急着讲道理',
      expressions: '常用「我举个例子啊」开头，讲完一定落到一个具体动作',
      audience: '30-45 岁、被失眠和情绪反复困扰的女性',
      painPoints: '白天硬撑、晚上睡不着，道理都懂但做不到，越努力越自责',
      taboos: '不聊具体用药、不做诊断、不承诺疗效',
      viewpoints: '先安顿身体，再谈情绪；能坚持的小动作比完美的计划有用',
      visualTone: '暖米色、浅木色、少量雾霾绿',
      visualStyle: '自然光、纪实感、不摆拍',
      visualScene: '家里的书桌旁、清晨的窗边',
      sourceNote: '手工录入',
    },
  },
  {
    name: '学员小林',
    category: 'case',
    fields: {
      ...emptyPersonaFields(),
      age: '34 岁',
      identity: '互联网产品经理、二胎妈妈',
      experience: '长期加班，二胎后睡眠崩了，试过早睡打卡但坚持不过三天，后来靠每天三分钟呼吸练习慢慢稳下来。',
      traits: ['要强', '自嘲', '爱较真'],
      catchphrases: '我这人就是不服输',
      expressions: '喜欢用数据说话，会自己记睡眠时长',
      audience: '和她一样上班带娃两头烧的职场妈妈',
      painPoints: '不敢停下来休息，一休息就有负罪感',
      taboos: '不聊婆媳矛盾、不涉及公司内部信息',
      viewpoints: '恢复不是靠忍，是靠找到能长期做下去的最小动作',
      visualTone: '冷灰 + 一点亮橙',
      visualStyle: '通勤场景纪实',
      visualScene: '地铁上、深夜的办公桌前',
      sourceNote: '来自第 3 期学员访谈',
    },
  },
]

function localRow(payload: PersonaPayload, index: number): Persona {
  return {
    id: -(index + 1),
    name: payload.name,
    category: payload.category,
    fields: JSON.stringify(payload.fields),
    created_at: localDateTime(),
    updated_at: localDateTime(),
  }
}

export const usePersonasStore = defineStore('personas', () => {
  const all = ref<Persona[]>([])
  const categories = ref<PersonaCategoryInfo[]>([
    { id: 'self_ip', label: '自我 IP' },
    { id: 'target_customer', label: '目标客户' },
    { id: 'case', label: '案例人物' },
  ])
  const keyword = ref('')
  const activeCategory = ref<'all' | PersonaCategory>('all')
  const loading = ref(false)
  const usingDatabase = ref(false)
  const extracting = ref(false)
  const lastExtraction = ref<PersonaExtraction | null>(null)
  const error = ref<AiErrorPayload | null>(null)

  /* --------------------------------- 派生数据 -------------------------------- */

  const counts = computed(() => {
    const result: Record<string, number> = { all: all.value.length }
    for (const category of categories.value) {
      result[category.id] = all.value.filter((item) => item.category === category.id).length
    }
    return result
  })

  /** 分类筛选 + 关键词搜索 */
  const filtered = computed(() => {
    const text = keyword.value.trim().toLowerCase()

    return all.value.filter((item) => {
      if (activeCategory.value !== 'all' && item.category !== activeCategory.value) return false
      if (!text) return true

      const fields = parsePersonaFields(item.fields)
      const haystack = [
        item.name,
        fields.identity,
        fields.experience,
        fields.audience,
        fields.painPoints,
        fields.viewpoints,
        fields.catchphrases,
        ...fields.traits,
      ]
        .join(' ')
        .toLowerCase()

      return haystack.includes(text)
    })
  })

  /* ---------------------------------- 加载 ---------------------------------- */

  async function hydrate(): Promise<void> {
    loading.value = true

    try {
      categories.value = await personaCategories()
    } catch {
      // 浏览器预览下拿不到，用本地默认分类
    }

    const rows = await personasRepo.list()

    if (rows) {
      usingDatabase.value = true
      if (rows.length > 0) {
        all.value = rows
      } else {
        const created: Persona[] = []
        for (const seed of SEEDS) {
          const row = await personasRepo.create(seed)
          if (row) created.push(row)
        }
        all.value = created
      }
    } else {
      const cached = readJson<Persona[]>(STORAGE_KEY, [])
      all.value = cached.length
        ? cached
        : SEEDS.map((seed, index) => localRow(seed, index))
    }

    loading.value = false
  }

  function persistLocal(): void {
    if (!usingDatabase.value) writeJson(STORAGE_KEY, all.value)
  }

  /* --------------------------------- 增删改查 -------------------------------- */

  async function create(payload: PersonaPayload): Promise<Persona | null> {
    const row = (await personasRepo.create(payload)) ?? localRow(payload, all.value.length)
    all.value = [row, ...all.value]
    persistLocal()
    return row
  }

  async function update(id: number, payload: PersonaPayload): Promise<Persona | null> {
    const saved = await personasRepo.update(id, payload)
    const row: Persona =
      saved ?? {
        id,
        name: payload.name,
        category: payload.category,
        fields: JSON.stringify(payload.fields),
        created_at: localDateTime(),
        updated_at: localDateTime(),
      }

    all.value = all.value.map((item) => (item.id === id ? row : item))
    persistLocal()
    return row
  }

  async function remove(id: number): Promise<void> {
    all.value = all.value.filter((item) => item.id !== id)
    if (usingDatabase.value && id > 0) await personasRepo.remove(id)
    persistLocal()
  }

  /* -------------------------------- 给别的模块用 ------------------------------- */

  function fieldsOf(id: number): PersonaFields {
    const row = all.value.find((item) => item.id === id)
    return row ? parsePersonaFields(row.fields) : emptyPersonaFields()
  }

  function byId(id: number): Persona | undefined {
    return all.value.find((item) => item.id === id)
  }

  /** 取某个人物的提示词块（writing / interview / image） */
  async function promptBlock(
    id: number,
    purpose: PersonaPurpose = 'writing',
  ): Promise<PersonaPromptBlock | null> {
    try {
      return await personaPromptBlock(id, purpose)
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    }
  }

  /** AI 从已有文案提取特征 */
  async function extract(sourceText: string, name?: string): Promise<PersonaFields | null> {
    extracting.value = true
    error.value = null

    try {
      const result = await aiExtractPersona(sourceText, name)
      lastExtraction.value = result
      return result.fields
    } catch (cause) {
      error.value = normalizeAiError(cause)
      return null
    } finally {
      extracting.value = false
    }
  }

  return {
    all,
    categories,
    keyword,
    activeCategory,
    loading,
    usingDatabase,
    extracting,
    lastExtraction,
    error,
    counts,
    filtered,
    hydrate,
    create,
    update,
    remove,
    fieldsOf,
    byId,
    promptBlock,
    extract,
  }
})
