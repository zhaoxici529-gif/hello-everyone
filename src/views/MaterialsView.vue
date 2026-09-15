<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import MaterialCard from '../components/material/MaterialCard.vue'
import MaterialViewer from '../components/material/MaterialViewer.vue'
import { useMaterialsStore } from '../stores/materials'
import { MATERIAL_TYPES, formatBytes, type Material } from '../types'

const materials = useMaterialsStore()
const message = useMessage()
const dialog = useDialog()

const dragging = ref(false)
const showViewer = ref(false)
const selected = ref<Material | null>(null)
const newCategory = ref('')
const importCategory = ref<'none' | number>('none')
let unlistenDragDrop: (() => void) | null = null

const totalBytes = computed(() =>
  materials.all.reduce((sum, item) => sum + (item.size_bytes ?? 0), 0),
)

onMounted(async () => {
  if (!materials.all.length) await materials.hydrate()

  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return

  // 拖拽导入用的是 Tauri 的窗口级事件，能直接拿到真实路径
  const { getCurrentWebview } = await import('@tauri-apps/api/webview')
  unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      dragging.value = true
      return
    }

    dragging.value = false
    if (event.payload.type !== 'drop') return

    void importPaths(event.payload.paths)
  })
})

onUnmounted(() => unlistenDragDrop?.())

function currentCategory(): number | null {
  return importCategory.value === 'none' ? null : importCategory.value
}

async function importPaths(paths: string[]): Promise<void> {
  const count = await materials.importPaths(paths, currentCategory())

  if (count > 0) message.success(`已导入 ${count} 个素材`)
  if (materials.error) message.warning(`${materials.error.message}：${materials.error.hint}`)
}

async function importViaPicker(): Promise<void> {
  const count = await materials.importViaPicker(currentCategory())

  if (count > 0) message.success(`已导入 ${count} 个素材`)
  else if (materials.error) message.error(materials.error.message)
}

function openViewer(material: Material): void {
  selected.value = material
  showViewer.value = true
}

function confirmRemove(material: Material): void {
  dialog.warning({
    title: '删除素材',
    content: `确定删除「${material.name}」吗？素材库里的文件也会一起删掉，不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await materials.remove(material.id)
      if (selected.value?.id === material.id) showViewer.value = false
      message.success('已删除')
    },
  })
}

async function runDescribe(material: Material): Promise<void> {
  const result = await materials.describe(material.id)

  if (!result) {
    message.error(materials.error?.message ?? '生成失败')
    return
  }

  message.success(result.sentImage ? '已根据图片生成描述' : '已生成描述')
}

async function addCategory(): Promise<void> {
  const name = newCategory.value.trim()
  if (!name) return

  const created = await materials.addCategory(name)
  if (!created) {
    message.error(materials.error?.message ?? '新建分类失败')
    return
  }

  newCategory.value = ''
  message.success(`已新建分类「${created.name}」`)
}

async function removeCategory(id: number): Promise<void> {
  await materials.removeCategory(id)
  if (materials.categoryFilter === id) materials.categoryFilter = 'all'
  message.success('分类已删除，素材回到「未分类」')
}
</script>

<template>
  <div class="materials-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="image" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">素材库</span>
        <h1>素材管理</h1>
        <p>
          共 {{ materials.all.length }} 个素材 · {{ formatBytes(totalBytes) }}
          <template v-if="materials.libraryDirectory">· 存于 {{ materials.libraryDirectory }}</template>
        </p>
      </div>
      <select v-model="importCategory" class="field hero-select">
        <option value="none">导入到：未分类</option>
        <option v-for="item in materials.categories" :key="item.id" :value="item.id">
          导入到：{{ item.name }}
        </option>
      </select>
      <button class="btn btn-primary" type="button" :disabled="materials.importing" @click="importViaPicker">
        <AppIcon name="plus" :size="15" />
        {{ materials.importing ? '导入中…' : '上传素材' }}
      </button>
    </section>

    <!-- 拖拽上传 -->
    <section
      class="dropzone"
      :class="{ 'is-dragging': dragging, 'is-busy': materials.importing }"
      @click="importViaPicker"
    >
      <AppIcon name="inbox" :size="22" />
      <div>
        <strong>{{ dragging ? '松手就开始导入' : '把文件拖到这里，或点击选择' }}</strong>
        <p>
          <span v-for="item in MATERIAL_TYPES" :key="item.key" class="dropzone-type">
            {{ item.label }} <code>{{ item.extensions }}</code>
          </span>
        </p>
      </div>
    </section>

    <p v-if="!materials.usingDatabase" class="hint is-error">
      当前是浏览器预览模式：可以看界面，但上传/播放需要桌面应用。请双击「自媒体AI工作台.exe」。
    </p>

    <!-- 分类管理 -->
    <div class="category-bar">
      <span class="bar-label">分类</span>
      <button
        class="filter-chip"
        :class="{ 'is-active': materials.categoryFilter === 'all' }"
        type="button"
        @click="materials.categoryFilter = 'all'"
      >
        全部
      </button>
      <button
        v-for="item in materials.categories"
        :key="item.id"
        class="filter-chip"
        :class="{ 'is-active': materials.categoryFilter === item.id }"
        type="button"
        @click="materials.categoryFilter = item.id"
      >
        {{ item.name }}
        <span
          class="chip-remove"
          title="删除分类"
          @click.stop="removeCategory(item.id)"
        >
          ×
        </span>
      </button>

      <div class="category-add">
        <input
          v-model="newCategory"
          class="field"
          placeholder="新建分类名"
          maxlength="20"
          @keydown.enter="addCategory"
        />
        <button class="btn btn-sm" type="button" @click="addCategory">新建</button>
      </div>
    </div>

    <!-- 类型 + 搜索 + 视图 -->
    <div class="toolbar">
      <div class="filter-row">
        <button
          class="filter-chip"
          :class="{ 'is-active': materials.typeFilter === 'all' }"
          type="button"
          @click="materials.typeFilter = 'all'"
        >
          全部 <span class="num">{{ materials.typeCounts.all ?? 0 }}</span>
        </button>
        <button
          v-for="item in MATERIAL_TYPES"
          :key="item.key"
          class="filter-chip"
          :class="{ 'is-active': materials.typeFilter === item.key }"
          type="button"
          @click="materials.typeFilter = item.key"
        >
          {{ item.label }} <span class="num">{{ materials.typeCounts[item.key] ?? 0 }}</span>
        </button>
      </div>

      <div class="search">
        <AppIcon name="search" :size="15" />
        <input v-model="materials.keyword" class="field" placeholder="搜索名称、标签、类型、描述…" />
      </div>

      <div class="view-switch">
        <button
          class="view-button"
          :class="{ 'is-active': materials.view === 'grid' }"
          type="button"
          title="网格视图"
          @click="materials.view = 'grid'"
        >
          网格
        </button>
        <button
          class="view-button"
          :class="{ 'is-active': materials.view === 'list' }"
          type="button"
          title="列表视图"
          @click="materials.view = 'list'"
        >
          列表
        </button>
      </div>
    </div>

    <!-- 标签筛选 -->
    <div v-if="materials.allTags.length" class="tag-bar">
      <span class="bar-label">标签</span>
      <button
        v-if="materials.tagFilter"
        class="filter-chip is-active"
        type="button"
        @click="materials.tagFilter = null"
      >
        {{ materials.tagFilter }} ×
      </button>
      <button
        v-for="item in materials.allTags"
        :key="item.tag"
        class="filter-chip"
        :class="{ 'is-active': materials.tagFilter === item.tag }"
        type="button"
        @click="materials.tagFilter = materials.tagFilter === item.tag ? null : item.tag"
      >
        {{ item.tag }} <span class="num">{{ item.count }}</span>
      </button>
    </div>

    <p class="result-line">
      显示 {{ materials.filtered.length }} / {{ materials.all.length }} 个素材
      <span v-if="materials.keyword">· 关键词「{{ materials.keyword }}」</span>
    </p>

    <div v-if="materials.filtered.length" class="material-grid" :class="{ 'is-list': materials.view === 'list' }">
      <MaterialCard
        v-for="item in materials.filtered"
        :key="item.id"
        :material="item"
        :view="materials.view"
        @open="openViewer"
        @describe="runDescribe"
        @remove="confirmRemove"
      />
    </div>

    <div v-else class="empty">
      <AppIcon name="image" :size="26" />
      <p v-if="materials.all.length">没有匹配的素材，换个关键词或清掉筛选试试。</p>
      <p v-else>素材库还是空的。把图片、视频、音频或文本拖进来，或者点上方「上传素材」。</p>
    </div>

    <MaterialViewer v-model:show="showViewer" :material="selected" />
  </div>
</template>

<style scoped>
.materials-page {
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
  word-break: break-all;
}

.hero-select {
  width: 180px;
  flex: none;
}

.dropzone {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-card);
  background: var(--bg-card);
  color: var(--text-3);
  cursor: pointer;
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.dropzone:hover,
.dropzone.is-dragging {
  border-color: var(--accent);
  background: var(--accent-softer);
  color: var(--accent);
}

.dropzone.is-busy {
  opacity: 0.7;
  cursor: progress;
}

.dropzone strong {
  display: block;
  font-size: 13.5px;
  color: var(--text-1);
}

.dropzone.is-dragging strong {
  color: var(--accent);
}

.dropzone p {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 5px;
}

.dropzone-type {
  color: var(--text-4);
  font-size: 11.5px;
}

.dropzone-type code {
  color: var(--text-3);
  font-family: var(--font-mono);
  font-size: 11px;
}

.category-bar,
.toolbar,
.tag-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.bar-label {
  color: var(--text-4);
  font-size: 12px;
  flex: none;
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
  font-size: 11px;
  opacity: 0.75;
}

.chip-remove {
  padding: 0 3px;
  opacity: 0.6;
  font-size: 13px;
}

.chip-remove:hover {
  opacity: 1;
  color: var(--danger);
}

.category-add {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

.category-add .field {
  width: 150px;
}

.search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 240px;
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

.view-switch {
  display: flex;
  gap: 3px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
}

.view-button {
  padding: 4px 12px;
  border: none;
  border-radius: var(--radius-pill);
  background: transparent;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.view-button.is-active {
  background: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.result-line {
  color: var(--text-3);
  font-size: 12.5px;
}

.material-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--gap);
  align-items: start;
}

.material-grid.is-list {
  grid-template-columns: 1fr;
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

.hint {
  color: var(--text-4);
  font-size: 12px;
}

.hint.is-error {
  color: var(--warn);
}
</style>
