import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { Note } from '../types'
import { notesRepo } from '../api'
import { localDateTime } from '../utils/datetime'

/** 随手记录：notes 表 */
export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([])
  const usingDatabase = ref(false)

  const total = computed(() => notes.value.length)

  async function hydrate(): Promise<void> {
    const rows = await notesRepo.list(20)
    if (rows) {
      usingDatabase.value = true
      notes.value = rows
    }
  }

  /**
   * 记一条随手记录。
   * convertedTo 用于记录它后来变成了什么，例如 'todo:12'。
   */
  async function record(content: string, convertedTo: string | null = null): Promise<Note | null> {
    const value = content.trim()
    if (!value) return null

    const created =
      (await notesRepo.create(value, convertedTo)) ??
      ({
        id: -(Date.now() % 100000),
        content: value,
        converted_to: convertedTo,
        created_at: localDateTime(),
      } satisfies Note)

    notes.value.unshift(created)
    return created
  }

  async function remove(id: number): Promise<void> {
    notes.value = notes.value.filter((item) => item.id !== id)
    if (usingDatabase.value && id > 0) await notesRepo.remove(id)
  }

  return { notes, total, usingDatabase, hydrate, record, remove }
})
