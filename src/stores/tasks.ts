import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { Todo } from '../types'
import { todosRepo } from '../api'
import { readJson, writeJson } from '../utils/storage'
import { localDate, localDateTime } from '../utils/datetime'

/** 浏览器预览（无 Tauri）时用的本地兜底缓存 */
const STORAGE_KEY = 'today-tasks'

const SEED_TITLES = [
  '整理「心身同调」7 天入门课的 3 个学员故事',
  '把上周直播切片改成 5 个备选标题',
  '回复评论区 12 条关于「总是失眠」的提问',
]

function localRow(title: string, done: boolean, index: number): Todo {
  return {
    id: -(index + 1),
    title,
    type: 'task',
    status: done ? 'done' : 'pending',
    plan_date: localDate(),
    done_at: done ? localDateTime() : null,
    created_at: localDateTime(),
  }
}

/** 今日待办：todos 表里 type = 'task' 的记录 */
export const useTasksStore = defineStore('tasks', () => {
  const tasks = ref<Todo[]>([])
  const draft = ref('')
  const hydrated = ref(false)
  const usingDatabase = ref(false)

  const total = computed(() => tasks.value.length)
  const doneCount = computed(() => tasks.value.filter((item) => item.status === 'done').length)
  const openCount = computed(() => total.value - doneCount.value)
  const progress = computed(() =>
    total.value === 0 ? 0 : Math.round((doneCount.value / total.value) * 100),
  )

  function persistLocal(): void {
    writeJson(
      STORAGE_KEY,
      tasks.value.map(({ title, status }) => ({ title, done: status === 'done' })),
    )
  }

  /** 首次进入：读 todos(type=task)；表是空的就把种子数据写进去 */
  async function hydrate(): Promise<void> {
    const rows = await todosRepo.listTasks()

    if (rows) {
      usingDatabase.value = true
      if (rows.length > 0) {
        tasks.value = rows
      } else {
        const created: Todo[] = []
        for (const title of SEED_TITLES) {
          const row = await todosRepo.create(title, 'task')
          if (row) created.push(row)
        }
        tasks.value = created
      }
      hydrated.value = true
      return
    }

    const cached = readJson<Array<{ title: string; done: boolean }>>(
      STORAGE_KEY,
      SEED_TITLES.map((title) => ({ title, done: false })),
    )
    tasks.value = cached.map((item, index) => localRow(item.title, item.done, index))
    hydrated.value = true
  }

  async function add(title?: string): Promise<void> {
    const value = (title ?? draft.value).trim()
    if (!value) return
    draft.value = ''

    const created = await todosRepo.create(value, 'task')
    tasks.value.push(created ?? localRow(value, false, tasks.value.length))
    persistLocal()
  }

  async function toggle(id: number): Promise<void> {
    const target = tasks.value.find((item) => item.id === id)
    if (!target) return

    const next = target.status === 'done' ? 'pending' : 'done'
    target.status = next
    target.done_at = next === 'done' ? localDateTime() : null

    if (usingDatabase.value && id > 0) await todosRepo.setStatus(id, next)
    persistLocal()
  }

  async function remove(id: number): Promise<void> {
    tasks.value = tasks.value.filter((item) => item.id !== id)
    if (usingDatabase.value && id > 0) await todosRepo.remove(id)
    persistLocal()
  }

  function setDraft(value: string): void {
    draft.value = value
  }

  return {
    tasks,
    draft,
    hydrated,
    usingDatabase,
    total,
    doneCount,
    openCount,
    progress,
    hydrate,
    add,
    toggle,
    remove,
    setDraft,
  }
})
