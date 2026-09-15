<script setup lang="ts">
import MissionCard from '../components/home/MissionCard.vue'
import GreetingCard from '../components/home/GreetingCard.vue'
import CalendarCard from '../components/home/CalendarCard.vue'
import HotspotCard from '../components/home/HotspotCard.vue'
import BenchmarkCard from '../components/home/BenchmarkCard.vue'
import TodoCard from '../components/home/TodoCard.vue'
import PomodoroCard from '../components/home/PomodoroCard.vue'
import AssistantCard from '../components/home/AssistantCard.vue'
import QuickNoteCard from '../components/home/QuickNoteCard.vue'
import InboxCard from '../components/home/InboxCard.vue'
import WorkspaceCard from '../components/WorkspaceCard.vue'
</script>

<template>
  <div class="home-page">
    <div class="home-grid">
      <!-- 第一行：横向铺满 -->
      <MissionCard class="span-12" />

      <!-- 第二行：左大卡（插画 + 时钟 + 问候） / 右日历 -->
      <GreetingCard class="span-7" />
      <CalendarCard class="span-5" />

      <!-- 第三行：AI 热点 / 对标博主动态 -->
      <HotspotCard class="span-6" />
      <BenchmarkCard class="span-6" />

      <!-- 第四行：今日待办 / 番茄钟 -->
      <TodoCard class="span-6" />
      <PomodoroCard class="span-6" />

      <!-- 第五行：常用助手 / 随手记录 -->
      <AssistantCard class="span-6" />
      <QuickNoteCard class="span-6" />

      <!-- 第六行：待处理 / 预留位 -->
      <InboxCard class="span-6" />
      <WorkspaceCard class="span-6 reserve" title="预留位" subtitle="给下一个模块留的位置" icon="layers">
        <div class="reserve-body">
          <p class="reserve-text">
            这里暂时空着。等数据复盘、变现追踪或学员进度看板做好之后，会补进这一格。
          </p>
          <div class="reserve-skeleton">
            <div class="skeleton-line" style="width: 88%" />
            <div class="skeleton-line" style="width: 64%" />
            <div class="skeleton-line" style="width: 76%" />
          </div>
        </div>
      </WorkspaceCard>
    </div>
  </div>
</template>

<style scoped>
.home-page {
  max-width: 1440px;
  margin: 0 auto;
}

.home-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: var(--gap);
  align-items: stretch;
}

.span-12 {
  grid-column: span 12;
}

.span-7 {
  grid-column: span 7;
}

.span-6 {
  grid-column: span 6;
}

.span-5 {
  grid-column: span 5;
}

.reserve-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.reserve-text {
  color: var(--text-3);
  font-size: 13px;
}

.reserve-skeleton {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px dashed var(--border-strong);
  border-radius: 10px;
}

@media (max-width: 1180px) {
  .home-grid > * {
    grid-column: span 12;
  }
}
</style>
