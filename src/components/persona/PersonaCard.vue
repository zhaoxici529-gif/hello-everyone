<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '../AppIcon.vue'
import { parsePersonaFields, type Persona, type PersonaPurpose } from '../../types'

const props = defineProps<{ persona: Persona }>()

const emit = defineEmits<{
  edit: [Persona]
  remove: [Persona]
  copy: [Persona, PersonaPurpose]
}>()

const CATEGORY_LABELS: Record<string, string> = {
  self_ip: '自我 IP',
  target_customer: '目标客户',
  case: '案例人物',
}

const fields = computed(() => parsePersonaFields(props.persona.fields))
const categoryLabel = computed(() => CATEGORY_LABELS[props.persona.category] ?? '未分类')
const initial = computed(() => props.persona.name.trim().charAt(0) || '人')
const shownTraits = computed(() => fields.value.traits.slice(0, 4))
const restTraits = computed(() => Math.max(0, fields.value.traits.length - 4))
const visual = computed(() =>
  [fields.value.visualTone, fields.value.visualStyle, fields.value.visualScene]
    .filter((item) => item.trim())
    .join(' · '),
)
</script>

<template>
  <article class="persona-card">
    <header class="persona-head">
      <span class="persona-avatar">{{ initial }}</span>
      <div class="persona-title">
        <h3>{{ persona.name }}</h3>
        <span class="persona-category">{{ categoryLabel }}</span>
      </div>
      <span v-if="fields.age" class="chip">{{ fields.age }}</span>
    </header>

    <p v-if="fields.identity" class="persona-identity">{{ fields.identity }}</p>

    <div v-if="shownTraits.length" class="persona-traits">
      <span v-for="trait in shownTraits" :key="trait" class="trait">{{ trait }}</span>
      <span v-if="restTraits" class="trait is-more">+{{ restTraits }}</span>
    </div>

    <dl class="persona-meta">
      <div v-if="fields.audience">
        <dt>目标受众</dt>
        <dd>{{ fields.audience }}</dd>
      </div>
      <div v-if="fields.painPoints">
        <dt>痛点</dt>
        <dd>{{ fields.painPoints }}</dd>
      </div>
      <div v-if="fields.catchphrases">
        <dt>口头禅</dt>
        <dd>{{ fields.catchphrases }}</dd>
      </div>
      <div v-if="visual">
        <dt>视觉偏好</dt>
        <dd>{{ visual }}</dd>
      </div>
    </dl>

    <p v-if="fields.experience" class="persona-experience">{{ fields.experience }}</p>

    <footer class="persona-actions">
      <button class="btn btn-sm" type="button" @click="emit('edit', persona)">
        <AppIcon name="edit" :size="14" />
        编辑
      </button>
      <button class="btn btn-sm btn-ghost" type="button" @click="emit('copy', persona, 'writing')">
        <AppIcon name="pen" :size="14" />
        写作提示词
      </button>
      <button class="btn btn-sm btn-ghost" type="button" @click="emit('copy', persona, 'image')">
        <AppIcon name="image" :size="14" />
        图片提示词
      </button>
      <button class="btn btn-sm btn-ghost persona-delete" type="button" @click="emit('remove', persona)">
        <AppIcon name="trash" :size="14" />
        删除
      </button>
    </footer>
  </article>
</template>

<style scoped>
.persona-card {
  display: flex;
  flex-direction: column;
  gap: 11px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-card);
  box-shadow: var(--shadow-card);
  transition: border-color 0.16s ease, transform 0.16s ease;
}

.persona-card:hover {
  border-color: var(--accent-border);
  transform: translateY(-1px);
}

.persona-head {
  display: flex;
  align-items: center;
  gap: 11px;
}

.persona-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 12px;
  background: linear-gradient(145deg, var(--accent), var(--accent-strong));
  color: var(--accent-ink);
  font-size: 17px;
  font-weight: 700;
  flex: none;
}

.persona-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.persona-title h3 {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.persona-category {
  color: var(--text-4);
  font-size: 11.5px;
}

.persona-identity {
  color: var(--accent);
  font-size: 12.5px;
}

.persona-traits {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.trait {
  padding: 3px 9px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11.5px;
}

.trait.is-more {
  background: var(--bg-inset);
  color: var(--text-4);
}

.persona-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
}

.persona-meta > div {
  display: flex;
  gap: 8px;
  align-items: baseline;
}

.persona-meta dt {
  flex: none;
  width: 56px;
  color: var(--text-4);
  font-size: 11.5px;
}

.persona-meta dd {
  margin: 0;
  color: var(--text-2);
  font-size: 12.5px;
  line-height: 1.6;
}

.persona-experience {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  color: var(--text-3);
  font-size: 12.5px;
  line-height: 1.7;
}

.persona-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  margin-top: auto;
  padding-top: 11px;
  border-top: 1px dashed var(--divider);
}

.persona-delete {
  margin-left: auto;
  color: var(--danger);
}

.persona-delete:hover {
  background: rgba(215, 154, 146, 0.1);
  border-color: rgba(215, 154, 146, 0.4);
  color: var(--danger);
}
</style>
