<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import RichTextEditor from '../components/RichTextEditor.vue'
import { usePersonasStore } from '../stores/personas'
import {
  downloadText,
  exportWriting,
  nextStatus,
  parseSnapshots,
  pushSnapshot,
  suggestionToPrompt,
  writingsRepo,
} from '../api/writing'
import type { ImageSuggestion, Writing, WritingGenre, WritingStatus } from '../types'

const personas = usePersonasStore()
const message = useMessage()
const dialog = useDialog()

const list = ref<Writing[]>([])
const loading = ref(false)
const saving = ref(false)
const usingDatabase = ref(false)

const statusFilter = ref<'all' | WritingStatus>('all')
const genreFilter = ref<'all' | WritingGenre>('all')
const keyword = ref('')

const selectedId = ref<number | null>(null)
const editTitle = ref('')
const editContent = ref('')
const editGenre = ref<WritingGenre>('xiaohongshu')
const editPersonaId = ref<number | null>(null)
const draftStatus = ref<WritingStatus>('draft')

const GENRE_LABELS: Record<string, string> = {
  xiaohongshu: '小红书',
  wechat: '公众号',
  script: '短视频脚本',
  moments: '朋友圈',
}

const STATUS_LABELS: Record<string, string> = {
  draft: '草稿',
  ready: '待发',
  published: '已发',
}

const STATUS_TABS: Array<{ key: 'all' | WritingStatus; label: string }> = [
  { key: 'all', label: '全部' },
  { key: 'draft', label: '草稿' },
  { key: 'ready', label: '待发' },
  { key: 'published', label: '已发' },
]

const selected = computed(() => list.value.find((item) => item.id === selectedId.value) ?? null)
const snapshots = computed(() => parseSnapshots(selected.value?.version_history))

/** 关联配图：存在 images 列里的配图建议 */
const suggestions = computed<ImageSuggestion[]>(() => {
  try {
    const parsed = JSON.parse(selected.value?.images ?? '[]')
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
})

const counts = computed<Record<string, number>>(() => ({
  all: list.value.length,
  draft: list.value.filter((item) => item.status === 'draft').length,
  ready: list.value.filter((item) => item.status === 'ready').length,
  published: list.value.filter((item) => item.status === 'published').length,
}))

/** 状态 + 文体 + 关键词三重筛选 */
const filtered = computed(() => {
  const text = keyword.value.trim().toLowerCase()

  return list.value.filter((item) => {
    if (statusFilter.value !== 'all' && item.status !== statusFilter.value) return false
    if (genreFilter.value !== 'all' && item.genre !== genreFilter.value) return false
    if (!text) return true

    return [item.title, item.content, item.genre]
      .join(' ')
      .toLowerCase()
      .includes(text)
  })
})

onMounted(async () => {
  if (!personas.all.length) await personas.hydrate()
  await reload()
})

async function reload(): Promise<void> {
  loading.value = true
  const rows = await writingsRepo.list()

  if (rows) {
    usingDatabase.value = true
    list.value = rows
  } else {
    list.value = []
  }

  loading.value = false
  if (!selectedId.value && list.value.length) select(list.value[0])
}

function select(item: Writing): void {
  selectedId.value = item.id
  editTitle.value = item.title
  editContent.value = item.content
  editGenre.value = (item.genre as WritingGenre) || 'xiaohongshu'
  editPersonaId.value = item.persona_id
  draftStatus.value = item.status
}

/** 列表里的正文预览：去掉富文本标签再截断 */
function previewOf(item: Writing): string {
  return item.content.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 60)
}

const dirty = computed(
  () =>
    selected.value !== null &&
    (selected.value.title !== editTitle.value ||
      selected.value.content !== editContent.value ||
      selected.value.genre !== editGenre.value ||
      selected.value.persona_id !== editPersonaId.value),
)

async function save(): Promise<void> {
  const current = selected.value
  if (!current) return

  saving.value = true
  const history = pushSnapshot(current.version_history, current.title, current.content)

  const updated = await writingsRepo.update(current.id, {
    title: editTitle.value.trim() || '未命名文案',
    content: editContent.value,
    genre: editGenre.value,
    persona_id: editPersonaId.value,
    version_history: history,
  })

  if (updated) {
    list.value = list.value.map((item) => (item.id === updated.id ? updated : item))
    message.success('已保存，上一版进了版本历史（保留最近 5 版）')
  } else {
    message.error('保存失败')
  }
  saving.value = false
}

async function switchStatus(target?: WritingStatus): Promise<void> {
  const current = selected.value
  if (!current) return

  const next = target ?? nextStatus(current.status)
  const patch: Record<string, unknown> = { status: next }

  // 第一次变成已发布时补上发布日期
  if (next === 'published' && !current.published_at) {
    patch.published_at = new Date().toISOString().slice(0, 10)
  }

  const updated = await writingsRepo.update(current.id, patch)
  if (updated) {
    list.value = list.value.map((item) => (item.id === updated.id ? updated : item))
    draftStatus.value = updated.status
    message.success(`状态已切换为「${STATUS_LABELS[next]}」`)
  }
}

function restoreSnapshot(index: number): void {
  const snapshot = snapshots.value[index]
  if (!snapshot) return

  editTitle.value = snapshot.title
  editContent.value = snapshot.content
  message.info(`已载入 ${snapshot.savedAt} 的版本，点保存才会生效`)
}

async function copyContent(): Promise<void> {
  const current = selected.value
  if (!current) return

  try {
    await navigator.clipboard.writeText(editContent.value.replace(/<[^>]+>/g, ''))
    message.success('文案已复制')
  } catch {
    message.warning('复制失败，请手动选中')
  }
}

async function exportFile(format: 'txt' | 'md'): Promise<void> {
  const current = selected.value
  if (!current) return

  try {
    const result = await exportWriting(current.id, format)
    downloadText(result.filename, result.content)
    message.success(`已导出 ${result.filename}`)
  } catch {
    message.error('导出失败：请用桌面应用打开')
  }
}

function confirmRemove(): void {
  const current = selected.value
  if (!current) return

  dialog.warning({
    title: '删除文案',
    content: `确定删除「${current.title}」吗？删除后不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await writingsRepo.remove(current.id)
      list.value = list.value.filter((item) => item.id !== current.id)
      selectedId.value = null
      if (list.value.length) select(list.value[0])
      message.success('已删除')
    },
  })
}

async function copyImagePrompt(suggestion: ImageSuggestion): Promise<void> {
  try {
    await navigator.clipboard.writeText(
      suggestionToPrompt(suggestion, selected.value?.title ?? ''),
    )
    message.success('配图提示词已复制')
  } catch {
    message.warning('复制失败')
  }
}
</script>

<template>
  <div class="manuscripts-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="layers" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">文案管理</span>
        <h1>文案管理</h1>
        <p>共 {{ list.length }} 篇 · 草稿 {{ counts.draft }} · 待发 {{ counts.ready }} · 已发 {{ counts.published }}</p>
      </div>
      <RouterLink to="/create" class="btn">
        <AppIcon name="pen" :size="15" />
        去创作
      </RouterLink>
    </section>

    <!-- 筛选条 -->
    <div class="toolbar">
      <div class="filter-row">
        <button
          v-for="tab in STATUS_TABS"
          :key="tab.key"
          class="filter-chip"
          :class="{ 'is-active': statusFilter === tab.key }"
          type="button"
          @click="statusFilter = tab.key"
        >
          {{ tab.label }} <span class="num">{{ counts[tab.key] }}</span>
        </button>
      </div>

      <select v-model="genreFilter" class="field genre-select">
        <option value="all">全部文体</option>
        <option v-for="(label, key) in GENRE_LABELS" :key="key" :value="key">{{ label }}</option>
      </select>

      <div class="search">
        <AppIcon name="search" :size="15" />
        <input v-model="keyword" class="field" placeholder="搜索标题、正文、标签…" />
      </div>
    </div>

    <p v-if="!usingDatabase" class="hint is-error">
      浏览器预览模式没有 Rust 运行时，草稿箱读不到数据。请用桌面应用打开。
    </p>

    <div class="split">
      <!-- 列表 -->
      <ul class="manuscript-list">
        <li v-for="item in filtered" :key="item.id" :class="{ 'is-active': selectedId === item.id }">
          <button type="button" @click="select(item)">
            <div class="item-head">
              <strong>{{ item.title }}</strong>
              <span class="status-chip" :class="'is-' + item.status">
                {{ STATUS_LABELS[item.status] }}
              </span>
            </div>
            <span class="item-meta">
              {{ GENRE_LABELS[item.genre] ?? item.genre }} · {{ item.updated_at }}
            </span>
            <span class="item-preview">{{ previewOf(item) }}</span>
          </button>
        </li>
        <li v-if="!filtered.length" class="empty">
          <AppIcon name="layers" :size="22" />
          <p>{{ list.length ? '没有匹配的文案' : '草稿箱还是空的。去「创作」写一篇，保存后就会出现在这里。' }}</p>
        </li>
      </ul>

      <!-- 详情 -->
      <section v-if="selected" class="detail">
        <div class="detail-head">
          <input v-model="editTitle" class="field detail-title" placeholder="标题" />
          <span v-if="dirty" class="chip">有未保存的修改</span>
        </div>

        <div class="detail-row">
          <label class="inline">
            <span>文体</span>
            <select v-model="editGenre" class="field">
              <option v-for="(label, key) in GENRE_LABELS" :key="key" :value="key">{{ label }}</option>
            </select>
          </label>

          <label class="inline">
            <span>人物小传</span>
            <select v-model="editPersonaId" class="field">
              <option :value="null">未关联</option>
              <option v-for="item in personas.all" :key="item.id" :value="item.id">
                {{ item.name }}
              </option>
            </select>
          </label>

          <span class="inline static">
            <span>选题</span>
            <b>{{ selected.topic_id ? `#${selected.topic_id}` : '未关联（选题模块接入后可选）' }}</b>
          </span>
        </div>

        <div class="status-row">
          <span class="row-label">状态</span>
          <button
            v-for="key in (['draft', 'ready', 'published'] as WritingStatus[])"
            :key="key"
            class="filter-chip"
            :class="{ 'is-active': draftStatus === key }"
            type="button"
            @click="switchStatus(key)"
          >
            {{ STATUS_LABELS[key] }}
          </button>
          <button class="btn btn-sm" type="button" @click="switchStatus()">
            <AppIcon name="arrowRight" :size="14" />
            一键切换
          </button>
        </div>

        <RichTextEditor v-model="editContent" :min-height="280" />

        <div class="detail-actions">
          <button class="btn btn-primary" type="button" :disabled="saving" @click="save">
            <AppIcon name="check" :size="15" />
            {{ saving ? '保存中…' : '保存' }}
          </button>
          <button class="btn" type="button" @click="copyContent">
            <AppIcon name="edit" :size="15" />
            复制文案
          </button>
          <button class="btn" type="button" @click="exportFile('txt')">导出 txt</button>
          <button class="btn" type="button" @click="exportFile('md')">导出 markdown</button>
          <button class="btn btn-ghost delete" type="button" @click="confirmRemove">
            <AppIcon name="trash" :size="15" />
            删除
          </button>
        </div>

        <!-- 关联配图 -->
        <div class="section">
          <div class="section-head">
            <AppIcon name="image" :size="15" />
            <strong>关联配图</strong>
            <span class="muted">{{ suggestions.length }} 条配图建议</span>
          </div>
          <ul v-if="suggestions.length" class="suggestion-list">
            <li v-for="(item, index) in suggestions" :key="index">
              <span class="chip">{{ item.position }}</span>
              <span class="suggestion-keywords">{{ item.keywords }}</span>
              <span class="muted">{{ item.style }} · {{ item.ratio }}</span>
              <button class="btn btn-sm btn-ghost" type="button" @click="copyImagePrompt(item)">
                复制提示词
              </button>
            </li>
          </ul>
          <p v-else class="muted empty-line">这篇还没有配图建议，在「写文案」里生成就会带上。</p>
        </div>

        <!-- 版本历史 -->
        <div class="section">
          <div class="section-head">
            <AppIcon name="clock" :size="15" />
            <strong>版本历史</strong>
            <span class="muted">保留最近 5 个版本</span>
          </div>
          <ul v-if="snapshots.length" class="version-list">
            <li v-for="(item, index) in snapshots" :key="index">
              <span class="num version-time">{{ item.savedAt }}</span>
              <span class="version-title">{{ item.title }}</span>
              <button class="btn btn-sm btn-ghost" type="button" @click="restoreSnapshot(index)">
                载入这一版
              </button>
            </li>
          </ul>
          <p v-else class="muted empty-line">还没有历史版本，改一次并保存就会记下来。</p>
        </div>
      </section>

      <section v-else class="detail empty-detail">
        <AppIcon name="layers" :size="26" />
        <p>左边选一篇文案，这里就能编辑、切状态、导出。</p>
      </section>
    </div>
  </div>
</template>

<style scoped>
.manuscripts-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1520px;
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

.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.filter-row {
  display: flex;
  gap: 6px;
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

.genre-select {
  width: 150px;
}

.search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 220px;
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

.split {
  display: grid;
  grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
  gap: var(--gap);
  align-items: start;
}

.manuscript-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
  max-height: 720px;
  overflow-y: auto;
}

.manuscript-list li > button {
  display: flex;
  flex-direction: column;
  gap: 5px;
  width: 100%;
  padding: 12px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-card);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
}

.manuscript-list li.is-active > button {
  border-color: var(--accent-border);
  background: var(--accent-softer);
}

.item-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-head strong {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-chip {
  flex: none;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-4);
  font-size: 11px;
}

.status-chip.is-published {
  background: var(--accent-soft);
  color: var(--accent);
}

.status-chip.is-ready {
  background: rgba(216, 192, 138, 0.14);
  color: var(--warn);
}

.item-meta,
.item-preview {
  color: var(--text-4);
  font-size: 11.5px;
}

.item-preview {
  color: var(--text-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail {
  display: flex;
  flex-direction: column;
  gap: 13px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-card);
  box-shadow: var(--shadow-card);
}

.empty-detail {
  align-items: center;
  justify-content: center;
  min-height: 320px;
  color: var(--text-4);
  font-size: 12.5px;
  text-align: center;
}

.detail-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.detail-title {
  font-size: 15px;
  font-weight: 600;
}

.detail-row,
.status-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.inline {
  display: flex;
  align-items: center;
  gap: 7px;
}

.inline > span {
  color: var(--text-4);
  font-size: 12px;
  flex: none;
}

.inline .field {
  width: 150px;
}

.inline.static b {
  color: var(--text-3);
  font-size: 12px;
  font-weight: 400;
}

.row-label {
  color: var(--text-4);
  font-size: 12px;
}

.detail-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px dashed var(--divider);
}

.delete {
  margin-left: auto;
  color: var(--danger);
}

.section {
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding-top: 12px;
  border-top: 1px dashed var(--divider);
}

.section-head {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent);
  font-size: 12.5px;
}

.section-head .muted {
  margin-left: auto;
  font-size: 11.5px;
}

.suggestion-list,
.version-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.suggestion-list li,
.version-list li {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 11px;
  border-radius: 9px;
  background: var(--bg-inset);
  font-size: 12px;
}

.suggestion-keywords {
  flex: 1;
  min-width: 0;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.version-time {
  flex: none;
  color: var(--text-4);
  font-size: 11.5px;
}

.version-title {
  flex: 1;
  min-width: 0;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px 18px;
  border: 1px dashed var(--border-strong);
  border-radius: 10px;
  color: var(--text-4);
  font-size: 12.5px;
  text-align: center;
}

.empty-line {
  font-size: 12px;
}

.hint {
  font-size: 12px;
}

.hint.is-error {
  color: var(--warn);
}

@media (max-width: 1000px) {
  .split {
    grid-template-columns: 1fr;
  }
}
</style>
