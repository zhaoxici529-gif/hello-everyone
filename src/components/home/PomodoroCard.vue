<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useFocusStore } from '../../stores/focus'

const focus = useFocusStore()

const RADIUS = 54
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

/** 剩余弧长：随着时间流逝逐渐变短 */
const dashOffset = computed(() => CIRCUMFERENCE * focus.progress)

const modeText = computed(() => (focus.mode === 'focus' ? '专注中' : '短休息'))
</script>

<template>
  <WorkspaceCard title="番茄钟" subtitle="25 分钟一个循环，一次只做一件事" icon="clock" class="pomodoro">
    <template #actions>
      <span class="chip">{{ focus.completedRounds }} 个已完成</span>
    </template>

    <div class="timer">
      <svg class="ring" viewBox="0 0 140 140" aria-hidden="true">
        <circle class="ring-track" cx="70" cy="70" :r="RADIUS" />
        <circle
          class="ring-value"
          :class="{ 'is-break': focus.mode === 'break' }"
          cx="70"
          cy="70"
          :r="RADIUS"
          :stroke-dasharray="CIRCUMFERENCE"
          :stroke-dashoffset="dashOffset"
          transform="rotate(-90 70 70)"
        />
      </svg>
      <div class="timer-center">
        <p class="timer-value num">{{ focus.clockText }}</p>
        <p class="timer-mode">
          <i class="timer-dot" :class="{ 'is-running': focus.running }" />
          {{ focus.running ? modeText : '已暂停' }}
        </p>
      </div>
    </div>

    <input v-model="focus.selectedTask" class="field" placeholder="这次专注要完成什么？" />

    <div class="duration-row">
      <span class="duration-label">时长</span>
      <button
        v-for="option in [15, 25, 45, 60]"
        :key="option"
        class="duration-chip"
        :class="{ 'is-active': focus.focusMinutes === option }"
        type="button"
        @click="focus.setMinutes(option)"
      >
        {{ option }} 分
      </button>
      <span class="duration-note">每完成一轮自动记进番茄钟记录</span>
    </div>

    <div class="timer-actions">
      <button class="btn btn-primary" type="button" @click="focus.toggle()">
        <AppIcon :name="focus.running ? 'pause' : 'play'" :size="15" />
        {{ focus.running ? '暂停' : '开始专注' }}
      </button>
      <button class="btn" type="button" @click="focus.reset()">
        <AppIcon name="reset" :size="15" />
        重置
      </button>
    </div>

    <p v-if="focus.lastRecord" class="record-note">
      <AppIcon name="check" :size="13" />
      {{ focus.lastRecord }}
    </p>
  </WorkspaceCard>
</template>

<style scoped>
.pomodoro {
  height: 100%;
}

.timer {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 4px auto 16px;
  width: 172px;
  height: 172px;
}

.ring {
  width: 100%;
  height: 100%;
}

.ring-track {
  fill: none;
  stroke: rgba(255, 255, 255, 0.06);
  stroke-width: 9;
}

.ring-value {
  fill: none;
  stroke: var(--accent);
  stroke-width: 9;
  stroke-linecap: round;
  filter: drop-shadow(0 0 6px rgba(184, 212, 168, 0.35));
  transition: stroke-dashoffset 0.4s linear;
}

.ring-value.is-break {
  stroke: var(--info);
  filter: none;
}

.timer-center {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.timer-value {
  font-size: 34px;
  font-weight: 600;
  letter-spacing: 1px;
  color: var(--text-1);
}

.timer-mode {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-3);
}

.timer-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-4);
}

.timer-dot.is-running {
  background: var(--accent);
  box-shadow: 0 0 8px rgba(184, 212, 168, 0.7);
}

.timer-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.duration-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 10px;
}

.duration-label {
  color: var(--text-4);
  font-size: 11.5px;
}

.duration-chip {
  padding: 4px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-3);
  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
}

.duration-chip.is-active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
  font-weight: 600;
}

.duration-note {
  color: var(--text-4);
  font-size: 11px;
}

.record-note {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  color: var(--accent);
  font-size: 11.5px;
}
</style>
