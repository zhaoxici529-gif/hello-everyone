<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useAiStore } from '../../stores/ai'

const ai = useAiStore()

const now = ref(new Date())
const timer = setInterval(() => {
  now.value = new Date()
}, 1000)

onUnmounted(() => clearInterval(timer))

const greeting = computed(() => {
  const hour = now.value.getHours()
  if (hour < 6) return '凌晨好'
  if (hour < 11) return '上午好'
  if (hour < 13) return '中午好'
  if (hour < 18) return '下午好'
  return '晚上好'
})

const timeText = computed(() =>
  now.value.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }),
)

const dateText = computed(() =>
  now.value.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  }),
)

const tip = computed(() => {
  const hour = now.value.getHours()
  if (hour < 11) return '早上是写初稿最好的时间，先动笔再优化。'
  if (hour < 18) return '下午适合改稿和回评论，别让灵感停在草稿箱。'
  return '晚上适合整理素材，把今天的收获放进待处理。'
})

/** 学员昵称（设置页可改） */
const welcome = computed(() =>
  ai.nickname ? `${greeting.value}，${ai.nickname}` : `${greeting.value}，欢迎回到工作台`,
)
</script>

<template>
  <WorkspaceCard flush class="greeting">
    <div class="greeting-inner">
      <div class="greeting-text">
        <p class="greeting-line">{{ welcome }}</p>
        <p class="greeting-clock num">{{ timeText }}</p>
        <p class="greeting-date">{{ dateText }}</p>
        <p class="greeting-tip">{{ tip }}</p>
      </div>

      <!-- 插画：夜色山峦 + 静坐的人，纯内联 SVG，无外部资源 -->
      <svg class="scene" viewBox="0 0 260 180" aria-hidden="true">
        <defs>
          <linearGradient id="sceneSky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#26331f" />
            <stop offset="100%" stop-color="#12160f" />
          </linearGradient>
          <linearGradient id="sceneHillFar" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#43592f" />
            <stop offset="100%" stop-color="#28361f" />
          </linearGradient>
          <linearGradient id="sceneHillNear" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#547042" />
            <stop offset="100%" stop-color="#1c2417" />
          </linearGradient>
        </defs>

        <rect width="260" height="180" rx="16" fill="url(#sceneSky)" />
        <circle cx="198" cy="46" r="24" fill="#b8d4a8" opacity="0.14" />
        <circle cx="198" cy="46" r="13" fill="#dbeacd" opacity="0.9" />

        <path d="M0 124c34-24 62-32 96-19 30 11 50 6 80-9 26-12 52-9 84 8v76H0z" fill="url(#sceneHillFar)" />
        <path d="M0 148c40-19 72-17 104-3 30 13 60 10 92-3 24-9 44-7 64 3v35H0z" fill="url(#sceneHillNear)" />

        <!-- 静坐的人 -->
        <circle cx="86" cy="106" r="7.5" fill="#e8ece6" opacity="0.92" />
        <path d="M72 133c0-8 6.3-15 14-15s14 7 14 15z" fill="#e8ece6" opacity="0.82" />
        <path d="M64 133h44" stroke="#b8d4a8" stroke-width="2.4" stroke-linecap="round" opacity="0.55" />

        <!-- 装饰叶片 -->
        <path d="M28 168c0-12 8-21 18-24-1 12-7 20-18 24z" fill="#8fbc8f" opacity="0.5" />
        <path d="M228 170c-2-10-8-17-16-20 0 10 6 17 16 20z" fill="#8fbc8f" opacity="0.35" />
      </svg>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.greeting {
  height: 100%;
}

.greeting-inner {
  display: flex;
  align-items: center;
  gap: 20px;
  height: 100%;
  padding: 18px;
}

.greeting-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.greeting-line {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.greeting-clock {
  font-size: 44px;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: 1px;
  color: var(--accent);
}

.greeting-date {
  color: var(--text-2);
  font-size: 13px;
}

.greeting-tip {
  margin-top: 6px;
  color: var(--text-4);
  font-size: 12.5px;
}

.scene {
  width: 260px;
  max-width: 42%;
  height: auto;
  flex: none;
  border-radius: 16px;
}

@media (max-width: 900px) {
  .scene {
    display: none;
  }
}
</style>
