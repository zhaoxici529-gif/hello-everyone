<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import PersonaCard from '../components/persona/PersonaCard.vue'
import PersonaEditor from '../components/persona/PersonaEditor.vue'
import { usePersonasStore } from '../stores/personas'
import type { Persona, PersonaPurpose } from '../types'

const personas = usePersonasStore()
const message = useMessage()
const dialog = useDialog()

const showEditor = ref(false)
const editing = ref<Persona | null>(null)

const PURPOSES = [
  { icon: 'pen', label: 'AI 写作', hint: '带上语气、受众与禁忌' },
  { icon: 'video', label: '模拟采访', hint: '带上经历、观点与口头禅' },
  { icon: 'image', label: '图片生成', hint: '带上色调、风格与场景' },
]

onMounted(async () => {
  if (!personas.all.length) await personas.hydrate()
})

function openCreate(): void {
  editing.value = null
  showEditor.value = true
}

function openEdit(persona: Persona): void {
  editing.value = persona
  showEditor.value = true
}

function confirmRemove(persona: Persona): void {
  dialog.warning({
    title: '删除人物档案',
    content: `确定删除「${persona.name}」吗？删除后不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await personas.remove(persona.id)
      message.success('已删除')
    },
  })
}

async function copyPrompt(persona: Persona, purpose: PersonaPurpose): Promise<void> {
  const block = await personas.promptBlock(persona.id, purpose)

  if (!block) {
    message.error(personas.error?.message ?? '取提示词失败')
    return
  }

  try {
    await navigator.clipboard.writeText(block.text)
    message.success(`已复制${block.purposeLabel}提示词，可以直接粘到常用助手里`)
  } catch {
    message.warning('复制失败，请手动选中提示词文本')
  }
}
</script>

<template>
  <div class="personas-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="users" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">人物小传</span>
        <h1>人物小传</h1>
        <p>把「你自己」「你要打动的人」「真实案例」写清楚，AI 才有依据替你说话。</p>
      </div>
      <button class="btn btn-primary" type="button" @click="openCreate">
        <AppIcon name="plus" :size="15" />
        新建人物
      </button>
    </section>

    <div class="purpose-grid">
      <div v-for="item in PURPOSES" :key="item.label" class="purpose">
        <span class="purpose-icon"><AppIcon :name="item.icon" :size="15" /></span>
        <div>
          <strong>{{ item.label }}</strong>
          <small>{{ item.hint }}</small>
        </div>
      </div>
    </div>

    <div class="toolbar">
      <div class="filter-row">
        <button
          class="filter-chip"
          :class="{ 'is-active': personas.activeCategory === 'all' }"
          type="button"
          @click="personas.activeCategory = 'all'"
        >
          全部 <span class="num">{{ personas.counts.all ?? 0 }}</span>
        </button>
        <button
          v-for="item in personas.categories"
          :key="item.id"
          class="filter-chip"
          :class="{ 'is-active': personas.activeCategory === item.id }"
          type="button"
          @click="personas.activeCategory = item.id"
        >
          {{ item.label }} <span class="num">{{ personas.counts[item.id] ?? 0 }}</span>
        </button>
      </div>

      <div class="search">
        <AppIcon name="search" :size="15" />
        <input
          v-model="personas.keyword"
          class="field"
          placeholder="搜索姓名、身份、受众、性格关键词…"
        />
        <button
          v-if="personas.keyword"
          class="btn btn-sm btn-ghost"
          type="button"
          @click="personas.keyword = ''"
        >
          清空
        </button>
      </div>
    </div>

    <p class="result-line">
      共 {{ personas.filtered.length }} 份档案
      <span v-if="personas.keyword">· 匹配关键词「{{ personas.keyword }}」</span>
      <span v-if="!personas.usingDatabase" class="muted">· 浏览器预览模式，数据只存在本地缓存</span>
    </p>

    <div v-if="personas.filtered.length" class="persona-grid">
      <PersonaCard
        v-for="persona in personas.filtered"
        :key="persona.id"
        :persona="persona"
        @edit="openEdit"
        @remove="confirmRemove"
        @copy="copyPrompt"
      />
    </div>

    <div v-else class="empty">
      <AppIcon name="users" :size="26" />
      <p v-if="personas.all.length">没有匹配的档案，换个关键词或切换分类试试。</p>
      <p v-else>还没有人物档案。先建一个「我自己」，写文案时就能带上你的语气。</p>
      <button class="btn btn-primary" type="button" @click="openCreate">
        <AppIcon name="plus" :size="15" />
        新建人物
      </button>
    </div>

    <PersonaEditor v-model:show="showEditor" :persona="editing" />
  </div>
</template>

<style scoped>
.personas-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1440px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: linear-gradient(120deg, rgba(184, 212, 168, 0.1), rgba(27, 32, 27, 0.9) 55%);
  box-shadow: var(--shadow-card);
}

.hero-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 54px;
  height: 54px;
  border-radius: 14px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  flex: none;
}

.hero-text {
  flex: 1;
  min-width: 0;
}

.hero-badge {
  display: inline-block;
  margin-bottom: 6px;
  padding: 2px 9px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11.5px;
}

.hero h1 {
  font-size: 21px;
  font-weight: 600;
}

.hero p {
  margin-top: 3px;
  color: var(--text-3);
  font-size: 13px;
}

.purpose-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 10px;
}

.purpose {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-card);
}

.purpose-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
  flex: none;
}

.purpose strong {
  display: block;
  font-size: 13px;
  color: var(--text-1);
}

.purpose small {
  color: var(--text-4);
  font-size: 11.5px;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-row {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-card);
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
}

.filter-chip:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}

.filter-chip.is-active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.filter-chip .num {
  font-size: 11.5px;
  opacity: 0.75;
}

.search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 260px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-inset);
  color: var(--text-4);
}

.search .field {
  border: none;
  background: transparent;
  padding-left: 0;
}

.search .field:focus {
  box-shadow: none;
}

.result-line {
  color: var(--text-3);
  font-size: 12.5px;
}

.persona-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: var(--gap);
  align-items: start;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 56px 24px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-card);
  color: var(--text-3);
  font-size: 13px;
  text-align: center;
}
</style>
