<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMessage } from 'naive-ui'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useInboxStore } from '../../stores/inbox'
import { useNotesStore } from '../../stores/notes'

type NoteMode = 'idea' | 'inbox'

const inbox = useInboxStore()
const notes = useNotesStore()
const message = useMessage()

const mode = ref<NoteMode>('idea')
const text = ref('')

const submitLabel = computed(() => (mode.value === 'idea' ? '记下灵感' : '放入待处理'))

async function submit(): Promise<void> {
  const value = text.value.trim()
  if (!value) {
    message.warning('先写点什么吧')
    return
  }

  // 两条写入：notes 留一份记录，todos 让它进入待处理队列
  const created = await inbox.add(value, mode.value === 'idea' ? 'idea' : 'text')
  await notes.record(value, created && created.id > 0 ? `todo:${created.id}` : null)

  text.value = ''
  message.success(mode.value === 'idea' ? '已记到灵感里' : '已放入待处理')
}
</script>

<template>
  <WorkspaceCard title="随手记录" subtitle="想法先落地，分类可以晚一点再做" icon="edit" class="quick-note">
    <div class="note-modes">
      <button
        class="note-mode"
        :class="{ 'is-active': mode === 'idea' }"
        type="button"
        @click="mode = 'idea'"
      >
        <AppIcon name="bulb" :size="15" />
        记灵感
      </button>
      <button
        class="note-mode"
        :class="{ 'is-active': mode === 'inbox' }"
        type="button"
        @click="mode = 'inbox'"
      >
        <AppIcon name="inbox" :size="15" />
        放入待处理
      </button>
    </div>

    <textarea
      v-model="text"
      class="field note-input"
      rows="5"
      placeholder="看到什么、想到什么，随手写下来。写下就算数，不用润色。"
    />

    <div class="note-footer">
      <span class="muted num">{{ text.length }} 字</span>
      <button class="btn btn-primary" type="button" @click="submit">
        <AppIcon name="plus" :size="15" />
        {{ submitLabel }}
      </button>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.quick-note {
  height: 100%;
}

.note-modes {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.note-mode {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 13px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-3);
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
}

.note-mode:hover {
  color: var(--text-1);
  border-color: var(--border-strong);
}

.note-mode.is-active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
  font-weight: 600;
}

.note-input {
  min-height: 108px;
}

.note-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
  font-size: 12px;
}
</style>
