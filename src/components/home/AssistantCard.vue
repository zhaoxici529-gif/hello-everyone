<script setup lang="ts">
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useAiStore } from '../../stores/ai'
import { useMessage } from 'naive-ui'
import { useRouter } from 'vue-router'
import { useWorkspaceStore, type Assistant } from '../../stores/workspace'
import { useAssistant } from '../../composables/useAssistant'

const workspace = useWorkspaceStore()
const ai = useAiStore()
const router = useRouter()
const message = useMessage()

// 统一走 useAssistant，避免和创作页各写一套调用逻辑
const { reply, run, clear } = useAssistant()

async function handle(item: Assistant): Promise<void> {
  // 知识库维护管家进的是本地记忆库，不是跑一次 AI
  // 跳转类的不跑 AI，生成类和改稿类才调模型
  if (item.key === 'knowledge') return void (await router.push('/memory'))
  if (item.key === 'copy') return void (await router.push('/create'))
  if (item.key === 'cover') {
    message.info('图片生成模块还没接入，先在创作页生成文案拿配图关键词')
    return
  }
  await run(item)
}
</script>

<template>
  <WorkspaceCard
    title="常用助手"
    subtitle="把重复劳动交给 AI，你只做判断"
    icon="sparkles"
    class="assistant"
  >
    <template #actions>
      <span class="chip">剩余体验 {{ ai.remaining }} 次</span>
    </template>

    <div class="assistant-grid">
      <button
        v-for="item in workspace.assistants"
        :key="item.key"
        class="assistant-item"
        type="button"
        :disabled="ai.busy"
        :title="item.hint"
        @click="handle(item)"
      >
        <span class="assistant-icon"><AppIcon :name="item.icon" :size="18" /></span>
        <span class="assistant-text">
          <strong>{{ item.label }}</strong>
          <small>{{ item.hint }}</small>
        </span>
        <AppIcon class="assistant-arrow" name="arrowRight" :size="15" />
      </button>
    </div>

    <div v-if="ai.busy" class="assistant-status">
      <span class="pulse" />
      AI 正在生成…
    </div>

    <div v-else-if="reply" class="assistant-reply">
      <div class="assistant-reply-head">
        <AppIcon name="sparkles" :size="14" />
        <strong>{{ reply.label }}</strong>
        <button class="btn btn-sm btn-ghost" type="button" @click="clear()">收起</button>
      </div>
      <pre class="assistant-reply-body">{{ reply.content }}</pre>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.assistant {
  height: 100%;
}

.assistant-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
}

.assistant-item {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 12px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
  color: inherit;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease, transform 0.16s ease;
}

.assistant-item:hover:not(:disabled) {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  transform: translateY(-1px);
}

.assistant-item:disabled {
  opacity: 0.55;
  cursor: progress;
}

.assistant-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
  flex: none;
}

.assistant-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.assistant-text strong {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-1);
}

.assistant-text small {
  font-size: 11.5px;
  color: var(--text-4);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.assistant-arrow {
  color: var(--text-4);
  flex: none;
}

.assistant-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  color: var(--text-3);
  font-size: 12.5px;
}

.pulse {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
  animation: pulse 1.1s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.35;
    transform: scale(0.85);
  }
  50% {
    opacity: 1;
    transform: scale(1.15);
  }
}

.assistant-reply {
  margin-top: 12px;
  border: 1px solid var(--accent-border);
  border-radius: 10px;
  background: var(--accent-softer);
  overflow: hidden;
}

.assistant-reply-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
  color: var(--accent);
  font-size: 12.5px;
}

.assistant-reply-head .btn {
  margin-left: auto;
}

.assistant-reply-body {
  margin: 0;
  padding: 12px;
  max-height: 200px;
  overflow-y: auto;
  color: var(--text-1);
  font-family: var(--font-ui);
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
