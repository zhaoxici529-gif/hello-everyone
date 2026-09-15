import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { ContentDraft } from '../types'

export interface Assistant {
  key: string
  label: string
  icon: string
  hint: string
}

/** 常用助手按钮网格 */
export const assistants: Assistant[] = [
  { key: 'copy', label: '写文案', icon: 'pen', hint: '把想法扩写成口播稿' },
  { key: 'title', label: '起标题', icon: 'sparkles', hint: '一次给 10 个备选标题' },
  { key: 'cover', label: '做封面', icon: 'image', hint: '按你的模板出封面图' },
  { key: 'revision', label: '学习我的改稿', icon: 'book', hint: '记住你的删改偏好' },
  { key: 'knowledge', label: '知识库维护管家', icon: 'layers', hint: '整理课程与素材库' },
]

export const useWorkspaceStore = defineStore('workspace', () => {
  // 今日主线：当前正在推进的那一条内容。
  const current = ref<ContentDraft>({
    id: 1,
    title: '《总是睡不好？先看看你的肩膀》—— 口播稿第 2 稿',
    stage: '改稿',
    updatedAt: new Date().toISOString(),
  })

  function setCurrent(draft: ContentDraft): void {
    current.value = draft
  }

  return { current, assistants, setCurrent }
})
