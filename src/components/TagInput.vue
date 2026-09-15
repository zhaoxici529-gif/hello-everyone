<script setup lang="ts">
import { ref } from 'vue'
import AppIcon from './AppIcon.vue'

/** 通用标签输入：回车添加，点 × 删除，数量上限可配 */
const props = withDefaults(
  defineProps<{
    modelValue: string[]
    max?: number
    placeholder?: string
    /** 建议数量，用于提示文案 */
    recommend?: string
  }>(),
  { max: 5, placeholder: '输入一个关键词，回车添加', recommend: '' },
)

const emit = defineEmits<{ 'update:modelValue': [string[]] }>()

const draft = ref('')

function add(): void {
  const value = draft.value.trim()
  if (!value) return
  if (props.modelValue.includes(value) || props.modelValue.length >= props.max) {
    draft.value = ''
    return
  }
  emit('update:modelValue', [...props.modelValue, value])
  draft.value = ''
}

function remove(index: number): void {
  emit(
    'update:modelValue',
    props.modelValue.filter((_, position) => position !== index),
  )
}
</script>

<template>
  <div class="trait-input">
    <div v-if="modelValue.length" class="trait-list">
      <span v-for="(tag, index) in modelValue" :key="tag" class="trait">
        {{ tag }}
        <button type="button" :title="'删除 ' + tag" @click="remove(index)">
          <AppIcon name="close" :size="11" />
        </button>
      </span>
    </div>

    <input
      v-model="draft"
      class="field"
      :placeholder="modelValue.length >= max ? `最多 ${max} 个` : placeholder"
      :disabled="modelValue.length >= max"
      @keydown.enter.prevent="add"
    />

    <p class="trait-hint">
      已添加 {{ modelValue.length }} / {{ max }} 个{{ recommend ? ` · ${recommend}` : '' }}
    </p>
  </div>
</template>

<style scoped>
.trait-input {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.trait-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.trait {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px 3px 10px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
}

.trait button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 15px;
  height: 15px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.7;
}

.trait button:hover {
  opacity: 1;
  background: rgba(184, 212, 168, 0.2);
}

.trait-hint {
  color: var(--text-4);
  font-size: 11.5px;
}
</style>
