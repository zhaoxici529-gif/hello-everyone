<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import WorkspaceCard from '../components/WorkspaceCard.vue'
import WriteStudio from '../components/writing/WriteStudio.vue'
import { useWritingStore } from '../stores/writing'
import { usePersonasStore } from '../stores/personas'
import { useAiStore } from '../stores/ai'
import { useAssistant } from '../composables/useAssistant'
import { suggestionToPrompt } from '../api/writing'
import type { Writing } from '../types'

const router = useRouter()
const writing = useWritingStore()
const personas = usePersonasStore()
const ai = useAiStore()
const message = useMessage()
const { reply, run, clear } = useAssistant()

const showStudio = ref(false)

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

const BOARD_ACTIONS = [
  { key: 'materials', label: '找资料', icon: 'image', hint: '跳素材库', area: '1 / 1' },
  { key: 'title', label: '起标题', icon: 'bulb', hint: 'AI 生成备选标题', area: '1 / 2' },
  { key: 'cover', label: '做封面', icon: 'image', hint: '用配图关键词出图', area: '1 / 3' },
  { key: 'revision', label: '学习改稿', icon: 'book', hint: 'AI 分析修改建议', area: '2 / 1' },
  { key: 'write', label: '写文案', icon: 'pen', hint: 'AI 写作主功能', area: '2 / 3' },
  { key: 'publish', label: '发布与复盘', icon: 'chart', hint: '跳发布页', area: '3 / 2' },
]

const activePersona = computed(
  () => personas.all.find((item) => item.id === writing.personaId) ?? null,
)

const boardStatus = computed(() => {
  if (writing.hasResult) {
    return `已有 ${writing.versions.length} 版 · 正在看版本 ${writing.activeIndex + 1}`
  }
  if (writing.topic.trim()) return '已定主题，可以开始生成'
  return '还没有开始，先定一个主题'
})

onMounted(async () => {
  await writing.hydrate()
  if (!personas.all.length) await personas.hydrate()
})

function openStudio(): void {
  showStudio.value = true
}

function startNew(): void {
  writing.resetDraft()
  openStudio()
}

function continueWriting(item: Writing): void {
  writing.load(item)
  openStudio()
}

function pickExisting(id: string): void {
  if (id === 'new') {
    writing.resetDraft()
    return
  }
  const target = writing.library.find((item) => String(item.id) === id)
  if (target) writing.load(target)
}

async function boardAction(key: string): Promise<void> {
  if (key === 'materials') {
    await router.push('/materials')
    return
  }
  if (key === 'publish') {
    await router.push('/publish')
    return
  }
  if (key === 'write') {
    openStudio()
    return
  }

  if (key === 'cover') {
    const suggestion = writing.imageKeywords[0]
    if (!suggestion) {
      message.info('先生成文案，系统会一并给出配图关键词')
      return
    }
    try {
      await navigator.clipboard.writeText(
        suggestionToPrompt(suggestion, writing.activeVersion?.title ?? ''),
      )
      message.success('已复制配图提示词，图片生成模块接入后可直接用')
    } catch {
      message.warning('复制失败，请到「写文案」里手动复制')
    }
    return
  }

  if (key === 'title') {
    await run(
      {
        key: 'title',
        label: writing.topic.trim() ? `给「${writing.topic}」起标题` : '起标题',
        icon: 'bulb',
        hint: '一次给 10 个备选标题',
      },
      activePersona.value,
    )
    return
  }

  if (key === 'revision') {
    if (!writing.activeVersion) {
      message.info('先生成或打开一篇文案，再来做改稿分析')
      return
    }
    await run(
      { key: 'revision', label: '学习我的改稿', icon: 'book', hint: '分析这版文案该怎么改' },
      activePersona.value,
    )
  }
}
</script>

<template>
  <div class="create-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="pen" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">创作台</span>
        <h1>创作</h1>
        <p>先定这次写什么，再在白板上一步步把它写出来。</p>
      </div>
      <RouterLink to="/personas" class="btn">
        <AppIcon name="users" :size="15" />
        人物小传
      </RouterLink>
      <RouterLink to="/materials" class="btn">
        <AppIcon name="image" :size="15" />
        找资料
      </RouterLink>
    </section>

    <div class="top-grid">
      <WorkspaceCard title="最近正在做" subtitle="接着上一篇继续" icon="clock">
        <template #actions>
          <RouterLink to="/manuscripts" class="btn btn-sm btn-ghost">
            <AppIcon name="layers" :size="14" />
            文案管理
          </RouterLink>
        </template>
        <ul v-if="writing.recent.length" class="recent-list">
          <li v-for="item in writing.recent" :key="item.id">
            <div class="recent-text">
              <strong>{{ item.title }}</strong>
              <span>
                {{ GENRE_LABELS[item.genre] ?? item.genre }} ·
                {{ STATUS_LABELS[item.status] ?? item.status }} ·
                {{ item.updated_at }}
              </span>
            </div>
            <button class="btn btn-sm" type="button" @click="continueWriting(item)">
              <AppIcon name="arrowRight" :size="14" />
              继续这篇
            </button>
          </li>
        </ul>
        <p v-else class="muted empty-line">还没有文案，从右边开一篇新的吧。</p>
      </WorkspaceCard>

      <WorkspaceCard title="这次创作哪篇内容" subtitle="新起一篇，或者接着改旧的" icon="compass">
        <div class="pick">
          <label class="field-block">
            <span class="field-label">从哪开始</span>
            <select class="field" @change="pickExisting(($event.target as HTMLSelectElement).value)">
              <option value="new">从新想法开始</option>
              <option v-for="item in writing.library" :key="item.id" :value="String(item.id)">
                已有文案：{{ item.title }}
              </option>
            </select>
          </label>

          <label class="field-block">
            <span class="field-label">补充你的想法</span>
            <textarea
              v-model="writing.extra"
              class="field"
              rows="3"
              placeholder="这次想讲什么、一定要提到什么"
            />
          </label>

          <label class="field-block">
            <span class="field-label">主题</span>
            <input v-model="writing.topic" class="field" placeholder="例如：失眠调理" />
          </label>

          <button class="btn btn-primary" type="button" @click="startNew">
            <AppIcon name="plus" :size="15" />
            进入写作台
          </button>
        </div>
      </WorkspaceCard>
    </div>

    <WorkspaceCard title="创作白板" subtitle="中心是这次的内容，周围是要用到的动作" icon="layers">
      <template #actions>
        <span class="chip">剩余体验 {{ ai.remaining }} 次</span>
      </template>

      <div class="board">
        <button
          v-for="action in BOARD_ACTIONS"
          :key="action.key"
          class="board-action"
          :class="{ 'is-primary': action.key === 'write' }"
          :style="{ gridArea: action.area }"
          type="button"
          :title="action.hint"
          @click="boardAction(action.key)"
        >
          <AppIcon :name="action.icon" :size="20" />
          <strong>{{ action.label }}</strong>
          <small>{{ action.hint }}</small>
        </button>

        <div class="board-center">
          <span class="chip">当前内容</span>
          <h3>{{ writing.topic.trim() || '还没有主题' }}</h3>
          <p>{{ boardStatus }}</p>
          <p class="board-genre">
            {{ GENRE_LABELS[writing.genre] ?? writing.genre }}
            <template v-if="activePersona"> · {{ activePersona.name }}</template>
          </p>
          <button class="btn btn-primary btn-sm" type="button" @click="openStudio">
            <AppIcon name="pen" :size="14" />
            {{ writing.hasResult ? '继续编辑' : '开始写作' }}
          </button>
        </div>
      </div>

      <div v-if="reply" class="reply">
        <div class="reply-head">
          <AppIcon name="sparkles" :size="14" />
          <strong>{{ reply.label }}</strong>
          <button class="btn btn-sm btn-ghost" type="button" @click="clear()">收起</button>
        </div>
        <pre class="reply-body">{{ reply.content }}</pre>
      </div>
    </WorkspaceCard>

    <WriteStudio v-model:show="showStudio" @saved="writing.hydrate()" />
  </div>
</template>

<style scoped>
.create-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1440px;
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

.top-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(380px, 1fr));
  gap: var(--gap);
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.recent-list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
}

.recent-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
  min-width: 0;
}

.recent-text strong {
  font-size: 13px;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-text span {
  color: var(--text-4);
  font-size: 11.5px;
}

.empty-line {
  font-size: 12.5px;
}

.pick {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  color: var(--text-3);
  font-size: 12.5px;
}

.pick .btn {
  align-self: flex-start;
}

.board {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  grid-template-rows: repeat(3, minmax(118px, auto));
  gap: 12px;
}

.board-action {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 5px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-inset);
  color: var(--text-3);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease, transform 0.16s ease;
}

.board-action:hover {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
  transform: translateY(-1px);
}

.board-action strong {
  font-size: 14px;
  color: var(--text-1);
}

.board-action small {
  color: var(--text-4);
  font-size: 11.5px;
}

.board-action.is-primary {
  border-color: var(--accent-border);
  background: var(--accent-softer);
}

.board-action.is-primary strong {
  color: var(--accent);
}

.board-center {
  grid-area: 2 / 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 18px;
  border: 1px solid var(--accent-border);
  border-radius: var(--radius-card);
  background: linear-gradient(160deg, rgba(184, 212, 168, 0.12), rgba(18, 21, 18, 0.9));
  text-align: center;
}

.board-center h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
  word-break: break-all;
}

.board-center p {
  color: var(--text-3);
  font-size: 12px;
}

.board-genre {
  color: var(--text-4) !important;
  font-size: 11.5px !important;
}

.reply {
  margin-top: 14px;
  border: 1px solid var(--accent-border);
  border-radius: 10px;
  background: var(--accent-softer);
  overflow: hidden;
}

.reply-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
  color: var(--accent);
  font-size: 12.5px;
}

.reply-head .btn {
  margin-left: auto;
}

.reply-body {
  margin: 0;
  padding: 12px;
  max-height: 260px;
  overflow-y: auto;
  color: var(--text-1);
  font-family: var(--font-ui);
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 900px) {
  .board {
    grid-template-columns: 1fr;
    grid-template-rows: auto;
  }

  .board-action,
  .board-center {
    grid-area: auto !important;
  }
}
</style>
