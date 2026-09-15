<script setup lang="ts">
import PlaceholderPage from '../components/PlaceholderPage.vue'

const sections = [
  { code: '01', title: '今天', desc: '今天要推进的一件小事，以及具体的时间段。' },
  { code: '02', title: '本周', desc: '本周要产出几条内容、分别投放在哪个平台。' },
  { code: '03', title: '本月', desc: '本月的更新节奏与重点选题方向。' },
  { code: '04', title: '安排一篇内容', desc: '直接在这里建一条内容排期，自动同步到日历。' },
  { code: '05', title: '本月目标与拆解', desc: '把涨粉、引流人数这类目标拆成每周可执行的动作。' },
]
</script>

<template>
  <PlaceholderPage
    title="计划"
    caption="目标拆到周，动作落到天，节奏稳一点反而更快。"
    icon="calendar"
    :sections="sections"
    note="占位页"
  />
</template>
