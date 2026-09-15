<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import WorkspaceCard from '../components/WorkspaceCard.vue'
import { useWritingStore } from '../stores/writing'
import { usePersonasStore } from '../stores/personas'
import { writingsRepo } from '../api/writing'
import type { Writing } from '../types'

const writing = useWritingStore()
const personas = usePersonasStore()
const message = useMessage()
const dialog = useDialog()

const monthFilter = ref('all')
const keyword = ref('')
const loading = ref(false)

const GENRE_LABELS: Record<string, string> = {
  xiaohongshu: '小红书',
  wechat: '公众号',
  script: '短视频脚本',
  moments: '朋友圈',
}

const STATUS_LABELS: Record<string, string> = {
  draft: '草稿',
  ready: '待发布',
  published: '已发布',
}

onMounted(async () => {
  loading.value = true
  if (!personas.all.length) await personas.hydrate()
  await writing.hydrate()
  loading.value = false
})

function monthOf(item: Writing): string {
  const stamp = item.published_at || item.updated_at || ''
  return stamp.slice(0, 7)
}

function personaName(item: Writing): string {
  if (!item.persona_id) return '未关联'
  return personas.all.find((row) => row.id === item.persona_id)?.name ?? '未关联'
}

function topicLabel(item: Writing): string {
  return item.topic_id ? '#' + String(item.topic_id) : '未关联'
}

function genreLabel(item: Writing): string {
  return GENRE_LABELS[item.genre] || item.genre
}

function statusLabel(item: Writing): string {
  return STATUS_LABELS[item.status] || item.status
}

function publishedDate(item: Writing): string {
  return item.published_at || ''
}

function statsDate(item: Writing): string {
  return item.stats_updated_at || ''
}

const publishedTotal = computed(
  () => writing.library.filter((item) => item.status === 'published').length,
)

const months = computed(() => {
  const set = new Set<string>()
  for (const item of writing.library) {
    const month = monthOf(item)
    if (month) set.add(month)
  }
  return [...set].sort().reverse()
})

/** 已发布作品：月份 + 标题/主题搜索 */
const works = computed(() => {
  const text = keyword.value.trim().toLowerCase()

  return writing.library
    .filter((item) => {
      if (monthFilter.value !== 'all' && monthOf(item) !== monthFilter.value) return false
      if (!text) return true
      return [item.title, personaName(item), item.content].join(' ').toLowerCase().includes(text)
    })
    .sort((first, second) => {
      const left = first.published_at || first.updated_at
      const right = second.published_at || second.updated_at
      return right.localeCompare(left)
    })
})

async function saveBusiness(item: Writing, patch: Record<string, unknown>): Promise<void> {
  if (item.id < 0) {
    Object.assign(item, patch)
    return
  }

  if (!item.stats_updated_at && !('stats_updated_at' in patch)) {
    patch.stats_updated_at = new Date().toISOString().slice(0, 10)
  }

  const updated = await writingsRepo.update(item.id, patch)
  if (updated) {
    writing.library = writing.library.map((row) => (row.id === updated.id ? updated : row))
    message.success('业务数据已保存')
  }
}

function onDate(item: Writing, value: string): void {
  void saveBusiness(item, { published_at: value })
}

function onStatsDate(item: Writing, value: string): void {
  void saveBusiness(item, { stats_updated_at: value })
}

function onNumber(item: Writing, field: string, value: string): void {
  void saveBusiness(item, { [field]: Number(value) || 0 })
}

async function markPublished(item: Writing): Promise<void> {
  const today = new Date().toISOString().slice(0, 10)

  if (item.id > 0) {
    const updated = await writingsRepo.update(item.id, {
      status: 'published',
      published_at: item.published_at || today,
    })
    if (updated) {
      writing.library = writing.library.map((row) => (row.id === updated.id ? updated : row))
    }
  } else {
    item.status = 'published'
  }

  message.success(`「${item.title}」已标记为已发布`)
}

function confirmRemove(item: Writing): void {
  dialog.warning({
    title: '删除作品',
    content: `确定删除「${item.title}」吗？删除后不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await writing.remove(item.id)
      message.success('已删除')
    },
  })
}
</script>

<template>
  <div class="publish-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="chart" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">发布与经营</span>
        <h1>发布与经营</h1>
        <p>已发布 {{ publishedTotal }} 篇 · 共 {{ writing.library.length }} 篇文案</p>
      </div>
      <RouterLink to="/manuscripts" class="btn">
        <AppIcon name="layers" :size="15" />
        文案管理
      </RouterLink>
      <RouterLink to="/create" class="btn">
        <AppIcon name="pen" :size="15" />
        去创作
      </RouterLink>
    </section>

    <WorkspaceCard title="01 已发布作品" subtitle="业务数据直接在表格里改，改完自动保存" icon="layers">
      <template #actions>
        <select v-model="monthFilter" class="field month-select">
          <option value="all">全部月份</option>
          <option v-for="month in months" :key="month" :value="month">{{ month }}</option>
        </select>
        <input v-model="keyword" class="field table-search" placeholder="搜索作品标题或主题" />
      </template>

      <div v-if="works.length" class="table-wrap">
        <table class="work-table">
          <thead>
            <tr>
              <th>作品标题</th>
              <th>主题 / 角色</th>
              <th>发布日期</th>
              <th>成交人数</th>
              <th>客资量</th>
              <th>引流私域人数</th>
              <th>咨询人数</th>
              <th>业务数据截至</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in works" :key="item.id">
              <td>
                <RouterLink to="/manuscripts" class="work-title">{{ item.title }}</RouterLink>
                <span class="work-sub">{{ genreLabel(item) }} · {{ statusLabel(item) }}</span>
              </td>
              <td>
                <span class="work-sub">角色：{{ personaName(item) }}</span>
                <span class="work-sub">主题：{{ topicLabel(item) }}</span>
              </td>
              <td>
                <input
                  class="field cell-field"
                  type="date"
                  :value="publishedDate(item)"
                  @change="onDate(item, ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td>
                <input
                  class="field cell-field num"
                  type="number"
                  min="0"
                  :value="item.deal_count"
                  @change="onNumber(item, 'deal_count', ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td>
                <input
                  class="field cell-field num"
                  type="number"
                  min="0"
                  :value="item.lead_count"
                  @change="onNumber(item, 'lead_count', ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td>
                <input
                  class="field cell-field num"
                  type="number"
                  min="0"
                  :value="item.private_count"
                  @change="onNumber(item, 'private_count', ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td>
                <input
                  class="field cell-field num"
                  type="number"
                  min="0"
                  :value="item.consult_count"
                  @change="onNumber(item, 'consult_count', ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td>
                <input
                  class="field cell-field"
                  type="date"
                  :value="statsDate(item)"
                  @change="onStatsDate(item, ($event.target as HTMLInputElement).value)"
                />
              </td>
              <td class="cell-actions">
                <button
                  v-if="item.status !== 'published'"
                  class="btn btn-sm"
                  type="button"
                  @click="markPublished(item)"
                >
                  标记已发布
                </button>
                <button class="btn btn-sm btn-ghost delete" type="button" @click="confirmRemove(item)">
                  删除
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <p v-else class="muted empty-line">
        这个月份还没有作品。去「创作」保存一篇，回到这里就能登记业务数据。
      </p>
    </WorkspaceCard>

    <WorkspaceCard title="02 内容数据复盘" subtitle="接入平台数据后的对比与结论" icon="trend">
      <p class="muted empty-line">
        这块还在建设中：后续会把每篇作品的播放、完播、互动和涨粉填进来，
        再对比出哪一类结构值得复用。
      </p>
    </WorkspaceCard>
  </div>
</template>

<style scoped>
.publish-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1520px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
  gap: 14px;
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

.month-select {
  width: 140px;
}

.table-search {
  width: 200px;
}

.table-wrap {
  overflow-x: auto;
}

.work-table {
  width: 100%;
  min-width: 1120px;
  border-collapse: collapse;
  font-size: 12.5px;
}

.work-table th {
  padding: 9px 10px;
  border-bottom: 1px solid var(--border);
  color: var(--text-4);
  font-weight: 400;
  text-align: left;
  white-space: nowrap;
}

.work-table td {
  padding: 9px 10px;
  border-bottom: 1px dashed var(--divider);
  color: var(--text-2);
  vertical-align: middle;
}

.work-table tr:hover td {
  background: var(--accent-softer);
}

.work-title {
  display: block;
  max-width: 240px;
  overflow: hidden;
  color: var(--text-1);
  font-size: 13px;
  text-decoration: none;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.work-title:hover {
  color: var(--accent);
}

.work-sub {
  display: block;
  margin-top: 3px;
  color: var(--text-4);
  font-size: 11.5px;
}

.cell-field {
  width: 118px;
  padding: 5px 8px;
  font-size: 12px;
}

.cell-field.num {
  width: 76px;
  font-family: var(--font-mono);
}

.cell-actions {
  display: flex;
  gap: 6px;
  white-space: nowrap;
}

.delete {
  color: var(--danger);
}

.empty-line {
  font-size: 12.5px;
  line-height: 1.8;
}
</style>
