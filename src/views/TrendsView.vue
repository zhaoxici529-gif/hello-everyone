<script setup lang="ts">
import PlaceholderPage from '../components/PlaceholderPage.vue'

const sections = [
  { code: '01', title: '今日热点榜', desc: '按心身同调、亲子关系、学习动力等方向聚合热点，支持一键转选题。' },
  { code: '02', title: '热点来源', desc: '管理热点抓取渠道：平台热榜、行业公众号、评论区高频提问。' },
  { code: '03', title: '我的关注词', desc: '维护关键词列表，决定每天给你推荐哪些热点。' },
]
</script>

<template>
  <PlaceholderPage
    title="今日热点"
    caption="每天自动汇总的行业热点，先看别人在关心什么，再决定你写什么。"
    icon="flame"
    :sections="sections"
    note="占位页"
  />
</template>
