<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import WorkspaceCard from '../components/WorkspaceCard.vue'
import { useMemoryStore } from '../stores/memory'
import { downloadText } from '../api/writing'
import type { MemorySource } from '../types'

const memory = useMemoryStore()
const message = useMessage()
const dialog = useDialog()

const fileInput = ref<HTMLInputElement | null>(null)

const SOURCE_TABS: Array<{ key: 'all' | MemorySource; label: string }> = [
  { key: 'all', label: '全部' },
  { key: 'persona', label: '人物小传' },
  { key: 'material', label: '素材' },
  { key: 'writing', label: '文案' },
  { key: 'topic', label: '选题' },
]

const total = computed(() => memory.stats?.total ?? memory.rows.length)

onMounted(async () => {
  await memory.load()
})

async function search(): Promise<void> {
  await memory.load()
}

function pickSource(key: 'all' | MemorySource): void {
  memory.sourceFilter = key
  void memory.load()
}

async function rebuild(): Promise<void> {
  dialog.warning({
    title: '重建记忆索引',
    content: '会把人物、素材、文案、选题重新摘要一遍，然后覆盖现有索引。继续吗？',
    positiveText: '重建',
    negativeText: '取消',
    onPositiveClick: async () => {
      await memory.reindex()
      message.success(`重建完成，共 ${memory.stats?.total ?? 0} 条记忆`)
    },
  })
}

function confirmRemove(id: number): void {
  dialog.warning({
    title: '删除这条记忆',
    content: '只删索引，不影响原始数据。确定删除吗？',
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await memory.remove(id)
      message.success('已删除')
    },
  })
}

async function exportAll(): Promise<void> {
  const payload = await memory.exportAll()
  if (!payload) {
    message.error(memory.error?.message ?? '导出失败')
    return
  }

  const stamp = new Date().toISOString().slice(0, 10)
  downloadText(`自媒体AI工作台-数据备份-${stamp}.json`, payload)
  message.success('已导出 JSON 备份')
}

function chooseFile(): void {
  fileInput.value?.click()
}

async function onFile(event: Event): Promise<void> {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  const text = await file.text()

  dialog.warning({
    title: '导入数据',
    content: '导入会用备份里的内容覆盖同名数据表，当前数据会被替换。建议先导出一份备份。继续吗？',
    positiveText: '导入',
    negativeText: '取消',
    onPositiveClick: async () => {
      const result = await memory.importAll(text)
      if (result) {
        message.success(`导入完成：${result.tables} 张表、${result.rows} 行数据`)
      } else {
        message.error(memory.error?.message ?? '导入失败')
      }
    },
  })

  ;(event.target as HTMLInputElement).value = ''
}
</script>

<template>
  <div class="memory-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="layers" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">本地工作记忆库</span>
        <h1>工作记忆库</h1>
        <p>共 {{ total }} 条记忆 · 来自人物小传、素材、文案和选题，AI 写作时会自动引用</p>
      </div>
      <button class="btn" type="button" :disabled="memory.busy" @click="rebuild">
        <AppIcon name="reset" :size="15" />
        重建索引
      </button>
      <button class="btn" type="button" :disabled="memory.busy" @click="exportAll">
        <AppIcon name="layers" :size="15" />
        导出备份
      </button>
      <button class="btn btn-primary" type="button" :disabled="memory.busy" @click="chooseFile">
        <AppIcon name="inbox" :size="15" />
        导入恢复
      </button>
      <input ref="fileInput" class="hidden-input" type="file" accept=".json,application/json" @change="onFile" />
    </section>

    <div v-if="memory.stats" class="stats">
      <button
        v-for="tab in SOURCE_TABS"
        :key="tab.key"
        class="filter-chip"
        :class="{ 'is-active': memory.sourceFilter === tab.key }"
        type="button"
        @click="pickSource(tab.key)"
      >
        {{ tab.label }}
        <span class="num">
          {{ tab.key === 'all' ? memory.stats.total : memory.stats.bySource.find((item) => item.sourceType === tab.key)?.count ?? 0 }}
        </span>
      </button>
    </div>

    <div class="search-row">
      <input
        v-model="memory.keyword"
        class="field"
        placeholder="搜索记忆内容，例如「失眠」「辅导作业」"
        @keydown.enter="search"
      />
      <button class="btn btn-primary" type="button" @click="search">
        <AppIcon name="search" :size="15" />
        搜索
      </button>
      <button
        v-if="memory.keyword"
        class="btn btn-ghost"
        type="button"
        @click="memory.keyword = ''; search()"
      >
        清空
      </button>
    </div>

    <p v-if="!memory.usingDatabase" class="hint is-error">
      浏览器预览模式没有 Rust 运行时，记忆库读不到数据。请用桌面应用打开。
    </p>

    <WorkspaceCard title="记忆索引" :subtitle="`显示 ${memory.rows.length} 条`" icon="layers">
      <ul v-if="memory.rows.length" class="memory-list">
        <li v-for="item in memory.rows" :key="item.id">
          <div class="memory-head">
            <span class="source-chip">{{ item.sourceLabel }}</span>
            <span class="memory-source num">#{{ item.sourceId }}</span>
            <span v-if="item.score > 0" class="score num">相关度 {{ item.score.toFixed(2) }}</span>
            <span class="memory-time num">{{ item.createdAt }}</span>
            <button class="btn btn-sm btn-ghost delete" type="button" @click="confirmRemove(item.id)">
              <AppIcon name="trash" :size="14" />
            </button>
          </div>
          <p class="memory-content">{{ item.content }}</p>
        </li>
      </ul>

      <p v-else class="muted empty-line">
        {{ memory.loading ? '正在读取…' : '还没有匹配的记忆。去人物小传、素材库或创作页保存一条数据，这里就会自动出现。' }}
      </p>
    </WorkspaceCard>

    <WorkspaceCard title="给其它模块用" subtitle="AI 写作、模拟采访可以按查询取相关片段" icon="sparkles">
      <p class="muted empty-line">
        Rust 侧提供 <code>memory_search(query, source_types, limit)</code> 和
        <code>memory_prompt_block(query, limit)</code>：前者返回相关记忆片段（含相关度），
        后者直接拼成可塞进 prompt 的「【工作记忆】」段落。
        检索排序用的是关键词命中 + 字符二元组相似度；embedding 列已预留，
        接入向量模型后替换打分函数即可。
      </p>
    </WorkspaceCard>
  </div>
</template>

<style scoped>
.memory-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1440px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: linear-gradient(120deg, rgba(184, 212, 168, 0.1), rgba(27, 32, 27, 0.9) 55%);
  box-shadow: var(--shadow-card);
  flex-wrap: wrap;
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
  min-width: 220px;
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

.hidden-input {
  display: none;
}

.stats {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 13px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-card);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
}

.filter-chip.is-active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.filter-chip .num {
  opacity: 0.75;
}

.search-row {
  display: flex;
  gap: 8px;
}

.memory-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.memory-list li {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 11px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.memory-head {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 11.5px;
}

.source-chip {
  padding: 2px 9px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
}

.memory-source,
.memory-time {
  color: var(--text-4);
}

.score {
  color: var(--info);
}

.memory-time {
  margin-left: auto;
}

.delete {
  color: var(--danger);
}

.memory-content {
  color: var(--text-2);
  font-size: 12.5px;
  line-height: 1.75;
  word-break: break-word;
}

.empty-line {
  font-size: 12.5px;
  line-height: 1.8;
}

.empty-line code {
  padding: 1px 6px;
  border-radius: 5px;
  background: var(--accent-soft);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.hint {
  font-size: 12px;
}

.hint.is-error {
  color: var(--warn);
}
</style>
