<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'

/**
 * 轻量富文本编辑器：contenteditable + 工具栏。
 * 存的是 HTML，导出时由 Rust 侧转成 txt / markdown。
 */
const props = withDefaults(defineProps<{ modelValue: string; minHeight?: number }>(), {
  minHeight: 260,
})

const emit = defineEmits<{ 'update:modelValue': [string] }>()

const editor = ref<HTMLDivElement | null>(null)

onMounted(() => {
  if (editor.value) editor.value.innerHTML = props.modelValue || ''
})

watch(
  () => props.modelValue,
  (value) => {
    // 只在外部值与编辑器内容不一致时同步，避免输入过程中光标跳动
    if (editor.value && editor.value.innerHTML !== (value || '')) {
      editor.value.innerHTML = value || ''
    }
  },
)

function emitValue(): void {
  emit('update:modelValue', editor.value?.innerHTML ?? '')
}

function exec(command: string, argument?: string): void {
  editor.value?.focus()
  document.execCommand(command, false, argument)
  emitValue()
}

const TOOLS = [
  { label: '粗', command: 'bold', title: '加粗' },
  { label: '斜', command: 'italic', title: '斜体' },
  { label: '小标题', command: 'formatBlock', argument: '<h3>', title: '小标题' },
  { label: '列表', command: 'insertUnorderedList', title: '无序列表' },
  { label: '引用', command: 'formatBlock', argument: '<blockquote>', title: '引用' },
  { label: '清格式', command: 'removeFormat', title: '清除格式' },
]
</script>

<template>
  <div class="rte">
    <div class="rte-toolbar">
      <button
        v-for="tool in TOOLS"
        :key="tool.label"
        type="button"
        :title="tool.title"
        @click="exec(tool.command, tool.argument)"
      >
        {{ tool.label }}
      </button>
    </div>
    <div
      ref="editor"
      class="rte-body"
      :style="{ minHeight: minHeight + 'px' }"
      contenteditable="true"
      @input="emitValue"
      @blur="emitValue"
    />
  </div>
</template>

<style scoped>
.rte {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-inset);
  overflow: hidden;
}

.rte-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 7px 8px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-card);
}

.rte-toolbar button {
  padding: 4px 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.rte-toolbar button:hover {
  background: var(--accent-soft);
  color: var(--accent);
}

.rte-body {
  padding: 12px 14px;
  color: var(--text-1);
  font-size: 13.5px;
  line-height: 1.85;
  outline: none;
  overflow-y: auto;
}

.rte-body :deep(h3) {
  margin: 12px 0 6px;
  font-size: 15px;
  color: var(--accent);
}

.rte-body :deep(blockquote) {
  margin: 8px 0;
  padding-left: 12px;
  border-left: 3px solid var(--accent-border);
  color: var(--text-2);
}

.rte-body :deep(ul) {
  padding-left: 20px;
}
</style>
