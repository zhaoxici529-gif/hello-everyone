<script setup lang="ts">
import WorkspaceCard from '../WorkspaceCard.vue'
import AppIcon from '../AppIcon.vue'

/** 占位数据：接入热点抓取后替换为真实条目 */
const placeholders = [
  { index: '01', tag: '睡眠', width: '78%' },
  { index: '02', tag: '情绪', width: '64%' },
  { index: '03', tag: '亲子', width: '86%' },
]
</script>

<template>
  <WorkspaceCard title="今日 AI 热点" subtitle="每天自动汇总心身同调相关的行业热点" icon="flame">
    <template #actions>
      <span class="chip">占位</span>
      <button class="btn btn-icon" type="button" title="刷新热点">
        <AppIcon name="reset" :size="16" />
      </button>
    </template>

    <ul class="hotspot-list">
      <li v-for="item in placeholders" :key="item.index">
        <span class="hotspot-index num">{{ item.index }}</span>
        <div class="hotspot-body">
          <div class="skeleton-line" :style="{ width: item.width }" />
          <div class="skeleton-line is-short" style="width: 42%" />
        </div>
        <span class="chip">{{ item.tag }}</span>
      </li>
    </ul>

    <p class="hotspot-note">
      这里会显示抓取到的热点标题与来源链接，可一键转为选题。
    </p>
  </WorkspaceCard>
</template>

<style scoped>
.hotspot-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.hotspot-list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.hotspot-index {
  flex: none;
  font-size: 12.5px;
  color: var(--accent);
}

.hotspot-body {
  display: flex;
  flex-direction: column;
  gap: 7px;
  flex: 1;
  min-width: 0;
}

.hotspot-note {
  margin-top: 12px;
  color: var(--text-4);
  font-size: 12px;
}
</style>
