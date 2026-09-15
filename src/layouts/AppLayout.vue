<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { NSpin, NTooltip } from 'naive-ui'
import TopNav from '../components/TopNav.vue'
import AppIcon from '../components/AppIcon.vue'
import OnboardingGuide from '../components/OnboardingGuide.vue'
import { useAppStore } from '../stores/app'
import { useAiStore } from '../stores/ai'
import { useInboxStore } from '../stores/inbox'
import { useNotesStore } from '../stores/notes'
import { useTasksStore } from '../stores/tasks'
import { initDatabase, settingGet } from '../api'

const route = useRoute()
const app = useAppStore()
const ai = useAiStore()
const tasks = useTasksStore()
const inbox = useInboxStore()
const notes = useNotesStore()
const booting = ref(true)
/** 首次启动引导 */
const showOnboarding = ref(false)

onMounted(async () => {
  // 1. 探测本地 SQLite（浏览器预览下返回 null）
  try {
    const status = await initDatabase()
    if (status) app.markReady(status)
    else app.markBrowser()
  } catch (error) {
    app.markError(error instanceof Error ? error.message : String(error))
  }

  // 2. 用 SQLite / localStorage 填充 store
  await Promise.all([tasks.hydrate(), inbox.hydrate(), notes.hydrate()])

  // 3. 读一次 AI 配置（模型选择、Key 状态、剩余体验次数、昵称）
  if (app.isDatabaseReady) await ai.load()

  // 4. 没走过引导就弹一次（settings.onboarded 为空表示首次启动）
  if (app.isDatabaseReady) {
    const onboarded = await settingGet('onboarded')
    showOnboarding.value = !onboarded
  }

  booting.value = false
})
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <div class="header-side header-left">
        <div class="brand">
          <span class="brand-mark">心</span>
          <span class="brand-text">
            <strong>自媒体 AI 工作台</strong>
            <small>心身同调 · 内容创作与获客</small>
          </span>
        </div>
      </div>

      <!-- 顶部主导航：横向居中，两侧留白 -->
      <TopNav class="header-center" />

      <div class="header-side header-right">
        <NTooltip trigger="hover">
          <template #trigger>
            <span class="db-pill" :class="'is-' + app.databaseStatus">
              <i class="db-dot" />
              {{ app.databaseText }}
            </span>
          </template>
          {{ app.databasePath || '本地 SQLite 尚未连接' }}
        </NTooltip>
        <RouterLink to="/settings" class="icon-button" title="设置">
          <AppIcon name="gear" :size="18" />
        </RouterLink>
      </div>
    </header>

    <main class="app-main">
      <div v-if="booting" class="boot-mask">
        <NSpin size="small" />
        <span>正在准备工作台…</span>
      </div>
      <RouterView v-else v-slot="{ Component }">
        <Transition name="fade" mode="out-in">
          <component :is="Component" :key="route.path" />
        </Transition>
      </RouterView>
    </main>

    <OnboardingGuide v-model:show="showOnboarding" />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.app-header {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(180deg, rgba(26, 29, 26, 0.92), rgba(15, 18, 16, 0.72));
  backdrop-filter: blur(14px);
  flex: none;
}

.header-side {
  display: flex;
  align-items: center;
  min-width: 0;
}

.header-left {
  justify-content: flex-start;
}

.header-right {
  justify-content: flex-end;
  gap: 10px;
}

.header-center {
  justify-self: center;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.brand-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  background: linear-gradient(145deg, var(--accent), var(--accent-strong));
  color: var(--accent-ink);
  font-size: 15px;
  font-weight: 700;
  flex: none;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
  min-width: 0;
}

.brand-text strong {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.brand-text small {
  font-size: 11.5px;
  color: var(--text-4);
}

.db-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
}

.db-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-4);
}

.db-pill.is-ready {
  color: var(--accent);
  border-color: var(--accent-border);
  background: var(--accent-softer);
}

.db-pill.is-ready .db-dot {
  background: var(--accent);
  box-shadow: 0 0 8px rgba(184, 212, 168, 0.7);
}

.db-pill.is-error {
  color: var(--danger);
}

.db-pill.is-error .db-dot {
  background: var(--danger);
}

.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-card);
  color: var(--text-2);
  text-decoration: none;
  transition: background 0.16s ease, color 0.16s ease;
}

.icon-button:hover {
  background: var(--accent-soft);
  color: var(--accent);
}

.app-main {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 20px;
}

.boot-mask {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 60vh;
  color: var(--text-3);
  font-size: 13px;
}

/* 空间不够时先收起品牌副标题和状态胶囊，给导航文字让位 */
@media (max-width: 1280px) {
  .brand-text {
    display: none;
  }

  .db-pill {
    display: none;
  }
}
</style>
