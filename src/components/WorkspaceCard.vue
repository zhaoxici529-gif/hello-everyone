<script setup lang="ts">
import AppIcon from './AppIcon.vue'

/**
 * 工作台卡片外壳：统一 12px 圆角、18px 内边距、标题行与 hover 反馈。
 * 首页所有卡片都基于它，避免每张卡片各写一套容器样式。
 */
withDefaults(
  defineProps<{
    title?: string
    subtitle?: string
    icon?: string
    /** 去掉内边距，交给内容自己控制 */
    flush?: boolean
  }>(),
  { title: '', subtitle: '', icon: '', flush: false },
)
</script>

<template>
  <section class="card" :class="{ 'is-flush': flush }">
    <header v-if="title || $slots.actions" class="card-head">
      <div class="card-heading">
        <span v-if="icon" class="card-icon"><AppIcon :name="icon" :size="16" /></span>
        <div class="card-heading-text">
          <h3 v-if="title" class="card-title">{{ title }}</h3>
          <p v-if="subtitle" class="card-subtitle">{{ subtitle }}</p>
        </div>
      </div>
      <div v-if="$slots.actions" class="card-actions">
        <slot name="actions" />
      </div>
    </header>
    <div class="card-body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-card);
  box-shadow: var(--shadow-card);
  transition: border-color 0.16s ease, background 0.16s ease;
}

.card:hover {
  border-color: var(--border-strong);
}

.card.is-flush {
  padding: 0;
}

.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}

.card-heading {
  display: flex;
  align-items: center;
  gap: 9px;
  min-width: 0;
}

.card-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
  flex: none;
}

.card-heading-text {
  min-width: 0;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
  letter-spacing: 0.2px;
}

.card-subtitle {
  margin-top: 1px;
  font-size: 12px;
  color: var(--text-3);
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}

.card-body {
  flex: 1;
  min-height: 0;
}
</style>
