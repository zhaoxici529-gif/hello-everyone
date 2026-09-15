<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useInboxStore } from '../../stores/inbox'
import { useNotesStore } from '../../stores/notes'

const inbox = useInboxStore()
const notes = useNotesStore()

/** 最近 4 条待处理，点分类标签能过滤 */
const recent = computed(() =>
  inbox.items.filter((item) => item.type === inbox.activeCategory).slice(0, 4),
)
</script>

<template>
  <WorkspaceCard id="home-inbox" title="待处理" subtitle="先收进来，再决定它值不值得做" icon="inbox" class="inbox-card">
    <template #actions>
      <span class="chip">共 {{ inbox.total }} 条</span>
    </template>

    <div class="stat-grid">
      <button
        v-for="item in inbox.stats"
        :key="item.key"
        class="stat-item"
        :class="{ 'is-active': inbox.activeCategory === item.key }"
        type="button"
        @click="inbox.activeCategory = item.key"
      >
        <span class="stat-icon"><AppIcon :name="item.icon" :size="15" /></span>
        <span class="stat-label">{{ item.label }}</span>
        <span class="stat-value num">{{ item.count }}</span>
      </button>
    </div>

    <ul class="inbox-list">
      <li v-for="item in recent" :key="item.id">
        <AppIcon :name="inbox.stats.find((s) => s.key === item.type)?.icon ?? 'text'" :size="14" />
        <span class="inbox-title">{{ item.title }}</span>
        <button class="inbox-remove" type="button" title="删除" @click="inbox.remove(item.id)">
          <AppIcon name="close" :size="13" />
        </button>
      </li>
      <li v-if="recent.length === 0" class="inbox-empty">这个分类还是空的。</li>
    </ul>

    <div class="inbox-foot">
      <span class="muted">随手记录 {{ notes.total }} 条</span>
      <RouterLink to="/plan" class="btn btn-sm btn-ghost">
        <AppIcon name="arrowRight" :size="14" />
        查看全部
      </RouterLink>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.inbox-card {
  height: 100%;
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 9px;
  margin-bottom: 14px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 5px;
  padding: 11px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
  font-family: inherit;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.stat-item:hover {
  border-color: var(--border-strong);
}

.stat-item.is-active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.stat-icon {
  color: var(--accent);
}

.stat-label {
  font-size: 11.5px;
  color: var(--text-3);
}

.stat-value {
  font-size: 19px;
  font-weight: 600;
  color: var(--text-1);
}

.inbox-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.inbox-list li {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
  color: var(--text-3);
}

.inbox-title {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inbox-remove {
  border: none;
  background: transparent;
  color: var(--text-4);
  cursor: pointer;
  padding: 2px;
  opacity: 0;
  transition: opacity 0.16s ease, color 0.16s ease;
}

.inbox-list li:hover .inbox-remove {
  opacity: 1;
}

.inbox-remove:hover {
  color: var(--danger);
}

.inbox-empty {
  justify-content: center;
  color: var(--text-4);
  font-size: 12.5px;
}

.inbox-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px dashed var(--divider);
  font-size: 12px;
}

@media (max-width: 900px) {
  .stat-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
