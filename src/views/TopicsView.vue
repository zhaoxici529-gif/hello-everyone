<script setup lang="ts">
import { ref } from 'vue'
import PlaceholderPage from '../components/PlaceholderPage.vue'

const tabs = [
  '从我的课程找',
  '从评论区找',
  '找近期热点',
  '看对标博主',
  '从已发布内容找',
]

const activeTab = ref(tabs[0])
</script>

<template>
  <PlaceholderPage
    title="选题"
    caption="五个来源，帮你把「不知道写什么」变成一张可执行的选题清单。"
    icon="bulb"
    :tabs="tabs"
    :active-tab="activeTab"
    note="占位页"
  />
</template>
