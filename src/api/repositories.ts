import { dbCount, dbDelete, dbInsert, dbList, dbUpdate, settingAll, settingGet, settingSet } from './db'
import type { Filter, InboxType, Note, Todo, TodoType } from '../types'
import { localDate, localDateTime } from '../utils/datetime'

/**
 * 领域仓储：把「表 + 常用条件」固化成语义化方法，
 * 内部全部走通用 CRUD command，不重复写 SQL。
 */

function typeFilter(type: TodoType): Filter {
  return { column: 'type', op: 'eq', value: type }
}

export const todosRepo = {
  /** 今日待办 = type 为 task 的待办 */
  listTasks(): Promise<Todo[] | null> {
    return dbList<Todo>('todos', {
      filters: [typeFilter('task')],
      orderBy: 'id',
      desc: false,
    })
  },

  /** 待处理 = idea / video / article / text 四类 */
  listInbox(): Promise<Todo[] | null> {
    return dbList<Todo>('todos', {
      filters: [{ column: 'type', op: 'in', value: ['idea', 'video', 'article', 'text'] }],
      orderBy: 'id',
      desc: true,
    })
  },

  create(title: string, type: TodoType = 'task', planDate: string | null = null): Promise<Todo | null> {
    return dbInsert<Todo>('todos', {
      title,
      type,
      status: 'pending',
      plan_date: planDate ?? (type === 'task' ? localDate() : null),
    })
  },

  setStatus(id: number, status: 'pending' | 'done'): Promise<Todo | null> {
    return dbUpdate<Todo>('todos', id, {
      status,
      done_at: status === 'done' ? localDateTime() : null,
    })
  },

  remove(id: number): Promise<boolean | null> {
    return dbDelete('todos', id)
  },

  count(type?: TodoType, status?: 'pending' | 'done'): Promise<number | null> {
    const filters: Filter[] = []
    if (type) filters.push(typeFilter(type))
    if (status) filters.push({ column: 'status', op: 'eq', value: status })
    return dbCount('todos', filters)
  },
}

export const notesRepo = {
  list(limit = 20): Promise<Note[] | null> {
    return dbList<Note>('notes', { orderBy: 'id', desc: true, limit })
  },

  create(content: string, convertedTo: string | null = null): Promise<Note | null> {
    return dbInsert<Note>('notes', { content, converted_to: convertedTo })
  },

  markConverted(id: number, convertedTo: string): Promise<Note | null> {
    return dbUpdate<Note>('notes', id, { converted_to: convertedTo })
  },

  remove(id: number): Promise<boolean | null> {
    return dbDelete('notes', id)
  },
}

export const settingsRepo = {
  get: settingGet,
  set: settingSet,
  all: settingAll,
}

/** 待处理四类的数量统计 */
export async function inboxCounts(types: InboxType[]): Promise<Record<string, number> | null> {
  const result: Record<string, number> = {}
  let available = false

  for (const type of types) {
    const value = await dbCount('todos', [typeFilter(type), { column: 'status', op: 'eq', value: 'pending' }])
    if (value === null) return null
    available = true
    result[type] = value
  }

  return available ? result : null
}
