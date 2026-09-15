<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import AppIcon from './AppIcon.vue'
import { navItems, type NavItem } from '../router'

const route = useRoute()

function isActive(item: NavItem): boolean {
  return item.path === '/' ? route.path === '/' : route.path.startsWith(item.path)
}
</script>

<template>
  <nav class="top-nav" aria-label="主导航">
    <RouterLink v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :class="{ 'is-active': isActive(item) }" :title="item.caption">
      <AppIcon :name="item.icon" :size="17" />
      <span class="nav-label">{{ item.title }}</span>
    </RouterLink>
  </nav>
</template>

<style scoped>
.top-nav {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: rgba(15, 18, 16, 0.72);
  backdrop-filter: blur(12px);
}

.nav-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  /* 未选中：深色背景 + 浅色文字 */
  background: var(--bg-card);
  color: var(--text-2);
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
  white-space: nowrap;
  transition: background 0.16s ease, color 0.16s ease, transform 0.16s ease;
}

.nav-item:hover {
  background: var(--bg-card-hover);
  color: var(--text-1);
}

.nav-item:active {
  transform: translateY(1px);
}

/* 当前选中：浅绿色背景 + 深色文字 */
.nav-item.is-active {
  background: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
  box-shadow: 0 6px 16px rgba(143, 188, 143, 0.22);
}

.nav-item.is-active:hover {
  background: #c6dcb8;
  color: var(--accent-ink);
}

/* 窗口变窄时只收紧间距，文字始终保留 */
@media (max-width: 1280px) {
  .nav-item {
    gap: 5px;
    padding: 7px 10px;
    font-size: 12.5px;
  }
}

@media (max-width: 980px) {
  .top-nav {
    gap: 4px;
    padding: 5px;
  }

  .nav-item {
    padding: 6px 8px;
    font-size: 12px;
  }
}
</style>
