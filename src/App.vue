<script setup lang="ts">
import { NConfigProvider, NDialogProvider, NMessageProvider, darkTheme } from 'naive-ui'
import { themeOverrides } from './styles/theme'
import AiQuotaGuide from './components/AiQuotaGuide.vue'
</script>

<template>
  <!-- 全局注入 Naive UI 深色主题 + 工作台配色覆盖 -->
  <NConfigProvider :theme="darkTheme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <NDialogProvider>
        <RouterView />
        <!-- 体验额度用完时的全局引导弹窗 -->
        <AiQuotaGuide />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
