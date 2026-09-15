<script lang="ts">
export interface PlaceholderSection {
  code: string
  title: string
  desc: string
}
</script>

<script setup lang="ts">
import AppIcon from './AppIcon.vue'
import WorkspaceCard from './WorkspaceCard.vue'

withDefaults(
  defineProps<{
    title: string
    caption: string
    icon: string
    /** 预留的子导航 / 结构区块 */
    sections?: PlaceholderSection[]
    /** 子导航标签 */
    tabs?: string[]
    activeTab?: string
    note?: string
  }>(),
  { sections: () => [], tabs: () => [], activeTab: '', note: '' },
)
</script>

<template>
  <div class="placeholder-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon :name="icon" :size="26" /></span>
      <div class="hero-text">
        <span class="hero-badge">占位页 · 结构已预留</span>
        <h1>{{ title }}</h1>
        <p>{{ caption }}</p>
      </div>
      <span v-if="note" class="hero-note">{{ note }}</span>
    </section>

    <nav v-if="tabs.length" class="sub-tabs">
      <button v-for="tab in tabs" :key="tab" class="sub-tab" :class="{ 'is-active': tab === activeTab }" type="button">
        {{ tab }}
      </button>
    </nav>

    <div v-if="sections.length" class="section-grid">
      <WorkspaceCard v-for="section in sections" :key="section.code">
        <div class="section-head">
          <span class="section-code num">{{ section.code }}</span>
          <h3>{{ section.title }}</h3>
        </div>
        <p class="section-desc">{{ section.desc }}</p>
        <div class="section-body">
          <slot :name="section.code" />
        </div>
      </WorkspaceCard>
    </div>

    <WorkspaceCard v-else title="功能开发中" icon="layers">
      <p class="empty-text">
        这个页面的骨架已经就位。后续接入选题库、热点抓取与创作白板后，内容会显示在这里。
      </p>
    </WorkspaceCard>
  </div>
</template>

<style scoped>
.placeholder-page {
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

.hero-text h1 {
  font-size: 21px;
  font-weight: 600;
  letter-spacing: 0.3px;
}

.hero-text p {
  margin-top: 3px;
  color: var(--text-3);
  font-size: 13px;
}

.hero-note {
  margin-left: auto;
  color: var(--text-4);
  font-size: 12px;
  white-space: nowrap;
}

.sub-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.sub-tab {
  padding: 7px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-card);
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.16s ease, color 0.16s ease, border-color 0.16s ease;
}

.sub-tab:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}

.sub-tab.is-active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.section-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: var(--gap);
}

.section-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.section-code {
  font-size: 13px;
  color: var(--accent);
  letter-spacing: 1px;
}

.section-head h3 {
  font-size: 15px;
  font-weight: 600;
}

.section-desc {
  margin-top: 8px;
  color: var(--text-3);
  font-size: 13px;
}

.section-body:not(:empty) {
  margin-top: 14px;
}

.empty-text {
  color: var(--text-3);
  font-size: 13px;
}
</style>
