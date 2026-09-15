import { ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useAiStore } from '../stores/ai'
import { usePersonasStore } from '../stores/personas'
import type { Assistant } from '../stores/workspace'
import type { Persona } from '../types'
import { ASSISTANT_TASKS, CONTENT_SYSTEM_PROMPT } from '../config/prompts'

export interface AssistantReply {
  label: string
  content: string
}

/**
 * 常用助手的统一执行逻辑。
 * 传入人物档案时，会把档案的提示词块拼进 prompt —— 这就是「人物小传供 AI 写作引用」的落点。
 */
export function useAssistant() {
  const ai = useAiStore()
  const personas = usePersonasStore()
  const message = useMessage()

  const reply = ref<AssistantReply | null>(null)

  async function run(item: Assistant, persona: Persona | null = null): Promise<boolean> {
    const task = ASSISTANT_TASKS[item.key] ?? item.hint
    let prompt = `帮我完成这件事：${item.label}。具体要求：${task}`

    if (persona) {
      const block = await personas.promptBlock(persona.id, 'writing')
      if (block) {
        prompt += `\n\n请严格按下面这份人物档案的语气、立场和禁忌来写：\n${block.text}`
      }
    }

    const response = await ai.run(prompt, { system: CONTENT_SYSTEM_PROMPT })

    if (!response) {
      // 额度用尽由 store 统一弹引导框，这里只处理其它错误
      if (ai.lastError && ai.lastError.code !== 'quota_exhausted') {
        message.error(ai.lastError.message)
      }
      return false
    }

    reply.value = {
      label: persona ? `${item.label} · ${persona.name}` : item.label,
      content: response.content,
    }

    message.success(
      response.usedTrialKey
        ? `已用免费体验额度生成，还剩 ${response.remainingFree} 次`
        : `已用 ${response.providerLabel} · ${response.model} 生成`,
    )
    return true
  }

  function clear(): void {
    reply.value = null
  }

  return { reply, run, clear }
}
