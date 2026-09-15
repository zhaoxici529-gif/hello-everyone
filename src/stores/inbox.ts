import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { INBOX_TYPES, type InboxType, type Todo } from '../types'
import { todosRepo } from '../api'
import { readJson, writeJson } from '../utils/storage'
import { localDateTime } from '../utils/datetime'

const STORAGE_KEY = 'inbox-items'

const SEEDS: Array<{ title: string; type: InboxType }> = [
  { title: '学员说「一紧张肩膀就发紧」，可以做成一条科普', type: 'idea' },
  { title: '抖音收藏：3 分钟呼吸训练跟练，做视频时可参考结构', type: 'video' },
  { title: '公众号文章《自律神经与情绪》：待精读并摘录观点', type: 'article' },
  { title: '课程群答疑里这段回应很打动人，已摘录原话', type: 'text' },
]

function localRow(title: string, type: InboxType, index: number): Todo {
  return {
    id: -(index + 1),
    title,
    type,
    status: 'pending',
    plan_date: null,
    done_at: null,
    created_at: localDateTime(),
  }
}

/** 待处理：todos 表里 idea / video / article / text 四类 */
export const useInboxStore = defineStore('inbox', () => {
  const items = ref<Todo[]>([])
  const activeCategory = ref<InboxType>('idea')
  const draft = ref('')
  const usingDatabase = ref(false)

  const pending = computed(() => items.value.filter((item) => item.status === 'pending'))
  const total = computed(() => pending.value.length)

  const stats = computed(() =>
    INBOX_TYPES.map((type) => ({
      ...type,
      count: pending.value.filter((item) => item.type === type.key).length,
    })),
  )

  async function hydrate(): Promise<void> {
    const rows = await todosRepo.listInbox()

    if (rows) {
      usingDatabase.value = true
      if (rows.length > 0) {
        items.value = rows
      } else {
        const created: Todo[] = []
        for (const seed of SEEDS) {
          const row = await todosRepo.create(seed.title, seed.type)
          if (row) created.push(row)
        }
        items.value = created
      }
      return
    }

    const cached = readJson<Array<{ title: string; type: InboxType }>>(
      STORAGE_KEY,
      SEEDS.map((seed) => ({ title: seed.title, type: seed.type })),
    )
    items.value = cached.map((item, index) => localRow(item.title, item.type, index))
  }

  /** 新增一条待处理，返回落库后的记录（浏览器预览下返回本地行） */
  async function add(
    title: string,
    type: InboxType = activeCategory.value,
  ): Promise<Todo | null> {
    const value = title.trim()
    if (!value) return null

    const created =
      (await todosRepo.create(value, type)) ?? localRow(value, type, items.value.length)
    items.value.unshift(created)
    draft.value = ''

    writeJson(
      STORAGE_KEY,
      items.value.map(({ title: text, type: kind }) => ({ title: text, type: kind })),
    )
    return created
  }

  async function remove(id: number): Promise<void> {
    items.value = items.value.filter((item) => item.id !== id)
    if (usingDatabase.value && id > 0) await todosRepo.remove(id)
  }

  return {
    items,
    stats,
    total,
    pending,
    activeCategory,
    draft,
    usingDatabase,
    hydrate,
    add,
    remove,
  }
})
