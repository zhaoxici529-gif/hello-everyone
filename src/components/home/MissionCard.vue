<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useInboxStore } from '../../stores/inbox'
import { useTasksStore } from '../../stores/tasks'
import { onMounted } from 'vue'
import { useWritingStore } from '../../stores/writing'

const tasks = useTasksStore()
const inbox = useInboxStore()
const writing = useWritingStore()

/** 今日主线 = 最近编辑的一篇文案 */
const mainline = computed(() => writing.recent[0] ?? null)

const GENRE_LABELS: Record<string, string> = {
  xiaohongshu: '小红书',
  wechat: '公众号',
  script: '短视频脚本',
  moments: '朋友圈',
}

onMounted(async () => {
  if (!writing.library.length) await writing.hydrate()
})

const todayLabel = new Date().toLocaleDateString('zh-CN', {
  month: 'long',
  day: 'numeric',
  weekday: 'long',
})

/** 「去处理」把页面滚动到对应卡片，避免离开首页上下文。 */
function focusCard(id: string): void {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}
</script>

<template>
  <WorkspaceCard
    title="今天先做什么"
    subtitle="先看主线，再清待办，最后处理收件箱"
    icon="compass"
    class="mission"
  >
    <template #actions>
      <span class="chip">{{ todayLabel }}</span>
    </template>

    <div class="mission-grid">
      <!-- 今日主线 -->
      <article class="block block-mainline">
        <header class="block-head">
          <AppIcon name="pen" :size="15" />
          <span>今日主线</span>
          <span class="chip block-stage">
            {{ mainline ? GENRE_LABELS[mainline.genre] ?? '文案' : '待创建' }}
          </span>
        </header>
        <p class="block-title">
          {{ mainline ? mainline.title : '还没有正在写的文案，去创作页开一篇' }}
        </p>
        <p class="block-meta">
          上次编辑 · {{ mainline ? mainline.updated_at : '—' }}
        </p>
        <RouterLink to="/create" class="btn btn-primary block-action">
          继续创作
          <AppIcon name="arrowRight" :size="15" />
        </RouterLink>
      </article>

      <!-- 今日待办 -->
      <article class="block">
        <header class="block-head">
          <AppIcon name="check" :size="15" />
          <span>今日待办</span>
        </header>
        <p class="block-number num">
          {{ tasks.openCount }}<small>/ {{ tasks.total }}</small>
        </p>
        <p class="block-meta">已完成 {{ tasks.doneCount }} 项 · 进度 {{ tasks.progress }}%</p>
        <button class="btn block-action" type="button" @click="focusCard('home-todo')">
          去处理
          <AppIcon name="arrowRight" :size="15" />
        </button>
      </article>

      <!-- 待处理 -->
      <article class="block">
        <header class="block-head">
          <AppIcon name="inbox" :size="15" />
          <span>待处理</span>
        </header>
        <p class="block-number num">
          {{ inbox.total }}<small>条</small>
        </p>
        <p class="block-meta">灵感 / 视频 / 文章 / 文字 待归类</p>
        <button class="btn block-action" type="button" @click="focusCard('home-inbox')">
          去处理
          <AppIcon name="arrowRight" :size="15" />
        </button>
      </article>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.mission-grid {
  display: grid;
  grid-template-columns: 1.4fr 1fr 1fr;
  gap: 14px;
}

.block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.block-mainline {
  background: linear-gradient(135deg, var(--accent-soft), rgba(18, 21, 18, 0.9) 60%);
  border-color: var(--accent-border);
}

.block-head {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-3);
  font-size: 12.5px;
}

.block-stage {
  margin-left: auto;
}

.block-title {
  color: var(--text-1);
  font-size: 15px;
  font-weight: 600;
  line-height: 1.5;
}

.block-number {
  color: var(--text-1);
  font-size: 30px;
  font-weight: 600;
  line-height: 1.15;
}

.block-number small {
  margin-left: 4px;
  font-size: 13px;
  color: var(--text-3);
  font-weight: 400;
}

.block-meta {
  color: var(--text-4);
  font-size: 12px;
}

.block-action {
  align-self: flex-start;
  margin-top: auto;
}

@media (max-width: 1180px) {
  .mission-grid {
    grid-template-columns: 1fr;
  }
}
</style>
