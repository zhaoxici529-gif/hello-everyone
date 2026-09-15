<script setup lang="ts">
import WorkspaceCard from '../WorkspaceCard.vue'
import AppIcon from '../AppIcon.vue'

/** 占位：后续从「对标博主」配置里读取，展示更新动态 */
const accounts = [
  { name: '对标账号 A', platform: '视频号', initial: 'A' },
  { name: '对标账号 B', platform: '小红书', initial: 'B' },
  { name: '对标账号 C', platform: '抖音', initial: 'C' },
]
</script>

<template>
  <WorkspaceCard title="对标博主动态" subtitle="盯着同类账号的更新，找可借鉴的结构" icon="users">
    <template #actions>
      <span class="chip">占位</span>
      <button class="btn btn-icon" type="button" title="刷新动态">
        <AppIcon name="reset" :size="16" />
      </button>
    </template>

    <ul class="account-list">
      <li v-for="account in accounts" :key="account.name">
        <span class="avatar">{{ account.initial }}</span>
        <div class="account-body">
          <p class="account-name">
            {{ account.name }}
            <span class="account-platform">{{ account.platform }}</span>
          </p>
          <div class="skeleton-line" style="width: 72%" />
        </div>
      </li>
    </ul>

    <p class="account-note">配置对标账号后，这里会自动汇总他们的最新作品与数据表现。</p>
  </WorkspaceCard>
</template>

<style scoped>
.account-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.account-list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  font-size: 12.5px;
  font-weight: 600;
  flex: none;
}

.account-body {
  display: flex;
  flex-direction: column;
  gap: 7px;
  flex: 1;
  min-width: 0;
}

.account-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-1);
}

.account-platform {
  color: var(--text-4);
  font-size: 11.5px;
}

.account-note {
  margin-top: 12px;
  color: var(--text-4);
  font-size: 12px;
}
</style>
