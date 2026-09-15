<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useWritingStore } from '../../stores/writing'

type CalendarMode = 'today' | 'week' | 'day' | 'list'

const MODES: Array<{ key: CalendarMode; label: string }> = [
  { key: 'today', label: '今天' },
  { key: 'week', label: '周' },
  { key: 'day', label: '日' },
  { key: 'list', label: '列表' },
]

const WEEK_LABELS = ['一', '二', '三', '四', '五', '六', '日']

const today = new Date()
const mode = ref<CalendarMode>('today')
const cursor = ref(new Date(today.getFullYear(), today.getMonth(), 1))

const monthText = computed(
  () => `${cursor.value.getFullYear()} 年 ${cursor.value.getMonth() + 1} 月`,
)

function toKey(date: Date): string {
  // 补零，好和数据库里的 2026-09-14 对齐
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

const writing = useWritingStore()

const GENRE_LABELS: Record<string, string> = {
  xiaohongshu: '小红书',
  wechat: '公众号',
  script: '短视频脚本',
  moments: '朋友圈',
}

const STATUS_LABELS: Record<string, string> = {
  draft: '草稿',
  ready: '待发',
  published: '已发',
}

interface PlanEvent {
  time: string
  title: string
  kind: string
}

/** 排期来自文案库：填了发布日期算「发布」，其余按更新时间算「编辑」 */
const planEvents = computed<Record<string, PlanEvent[]>>(() => {
  const map: Record<string, PlanEvent[]> = {}

  for (const item of writing.library) {
    const stamp = item.published_at || item.updated_at || ''
    const key = stamp.slice(0, 10)
    if (!key) continue

    const entry: PlanEvent = {
      time: item.published_at ? '发布' : '编辑',
      title: item.title,
      kind: GENRE_LABELS[item.genre] ?? STATUS_LABELS[item.status] ?? '文案',
    }
    map[key] = [...(map[key] ?? []), entry]
  }

  return map
})

onMounted(async () => {
  if (!writing.library.length) await writing.hydrate()
})

interface Cell {
  key: string
  day: number
  inMonth: boolean
  isToday: boolean
  hasEvent: boolean
  weekend: boolean
}

const cells = computed<Cell[]>(() => {
  const year = cursor.value.getFullYear()
  const month = cursor.value.getMonth()
  const first = new Date(year, month, 1)
  // 周一为一周起点
  const offset = (first.getDay() + 6) % 7
  const start = new Date(year, month, 1 - offset)
  const list: Cell[] = []

  for (let index = 0; index < 42; index += 1) {
    const date = new Date(start.getFullYear(), start.getMonth(), start.getDate() + index)
    const key = toKey(date)
    list.push({
      key,
      day: date.getDate(),
      inMonth: date.getMonth() === month,
      isToday: toKey(date) === toKey(today),
      hasEvent: Boolean(planEvents.value[key]),
      weekend: date.getDay() === 0 || date.getDay() === 6,
    })
  }
  return list
})

const todayEvents = computed(() => planEvents.value[toKey(today)] ?? [])

const weekDays = computed(() =>
  Array.from({ length: 7 }, (_, index) => {
    const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() + index)
    return {
      key: toKey(date),
      weekday: WEEK_LABELS[(date.getDay() + 6) % 7],
      day: date.getDate(),
      events: planEvents.value[toKey(date)] ?? [],
    }
  }),
)

function shiftMonth(step: number): void {
  cursor.value = new Date(cursor.value.getFullYear(), cursor.value.getMonth() + step, 1)
}
</script>

<template>
  <WorkspaceCard class="calendar">
    <template #actions>
      <div class="mode-switch">
        <button
          v-for="item in MODES"
          :key="item.key"
          class="mode-button"
          :class="{ 'is-active': mode === item.key }"
          type="button"
          @click="mode = item.key"
        >
          {{ item.label }}
        </button>
      </div>
    </template>

    <div class="calendar-head">
      <div class="calendar-title">
        <h3>{{ monthText }}</h3>
        <span class="muted">来自文案库的发布 / 编辑排期</span>
      </div>
      <div class="calendar-nav">
        <button class="btn btn-icon" type="button" title="上个月" @click="shiftMonth(-1)">
          <AppIcon name="chevronLeft" :size="16" />
        </button>
        <button class="btn btn-icon" type="button" title="下个月" @click="shiftMonth(1)">
          <AppIcon name="chevronRight" :size="16" />
        </button>
      </div>
    </div>

    <!-- 今天 / 周：显示月网格 -->
    <div v-if="mode === 'today' || mode === 'week'" class="month-grid">
      <span v-for="label in WEEK_LABELS" :key="label" class="weekday">{{ label }}</span>
      <span
        v-for="cell in cells"
        :key="cell.key"
        class="day-cell"
        :class="{
          'is-out': !cell.inMonth,
          'is-today': cell.isToday,
          'is-weekend': cell.weekend,
        }"
      >
        {{ cell.day }}
        <i v-if="cell.hasEvent" class="event-dot" />
      </span>
    </div>

    <!-- 日视图 -->
    <div v-else-if="mode === 'day'" class="day-view">
      <p class="day-view-date num">
        {{ today.getFullYear() }}-{{ today.getMonth() + 1 }}-{{ today.getDate() }}
      </p>
      <p class="day-view-week">{{ today.toLocaleDateString('zh-CN', { weekday: 'long' }) }}</p>
    </div>

    <div class="agenda">
      <template v-if="mode === 'today'">
        <p class="agenda-caption">今天的安排</p>
        <p v-if="todayEvents.length === 0" class="agenda-empty">今天还没有安排，可以给自己留一段空白。</p>
        <ul v-else class="agenda-list">
          <li v-for="event in todayEvents" :key="event.title">
            <span class="agenda-time num">{{ event.time }}</span>
            <span class="agenda-title">{{ event.title }}</span>
            <span class="chip">{{ event.kind }}</span>
          </li>
        </ul>
      </template>

      <template v-else-if="mode === 'week'">
        <p class="agenda-caption">未来 7 天</p>
        <ul class="agenda-list">
          <li v-for="item in weekDays" :key="item.key">
            <span class="agenda-time num">周{{ item.weekday }} {{ item.day }}</span>
            <span class="agenda-title">
              {{ item.events.length ? item.events[0].title : '暂无安排' }}
            </span>
          </li>
        </ul>
      </template>

      <template v-else-if="mode === 'day'">
        <p class="agenda-caption">当日明细</p>
        <ul v-if="todayEvents.length" class="agenda-list">
          <li v-for="event in todayEvents" :key="event.title">
            <span class="agenda-time num">{{ event.time }}</span>
            <span class="agenda-title">{{ event.title }}</span>
          </li>
        </ul>
        <p v-else class="agenda-empty">这一天还是空的。</p>
      </template>

      <template v-else>
        <p class="agenda-caption">待办清单视图</p>
        <ul class="agenda-list">
          <li v-for="(list, day) in planEvents" :key="day">
            <span class="agenda-time num">{{ day }}</span>
            <span class="agenda-title">{{ list[0]?.title }}</span>
          </li>
        </ul>
      </template>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.calendar {
  height: 100%;
}

.mode-switch {
  display: flex;
  gap: 3px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
}

.mode-button {
  padding: 4px 11px;
  border: none;
  border-radius: var(--radius-pill);
  background: transparent;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.16s ease, color 0.16s ease;
}

.mode-button:hover {
  color: var(--text-1);
}

.mode-button.is-active {
  background: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.calendar-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.calendar-title {
  display: flex;
  flex-direction: column;
}

.calendar-title h3 {
  font-size: 15px;
  font-weight: 600;
}

.calendar-title span {
  font-size: 11.5px;
}

.calendar-nav {
  display: flex;
  gap: 6px;
}

.month-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.weekday {
  padding-bottom: 4px;
  text-align: center;
  font-size: 11.5px;
  color: var(--text-4);
}

.day-cell {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 1 / 1;
  border-radius: 9px;
  font-size: 12.5px;
  color: var(--text-2);
  transition: background 0.14s ease, color 0.14s ease;
}

.day-cell:hover {
  background: var(--bg-card-hover);
}

.day-cell.is-out {
  color: var(--text-4);
  opacity: 0.5;
}

.day-cell.is-weekend {
  color: var(--text-3);
}

.day-cell.is-today {
  background: var(--accent);
  color: var(--accent-ink);
  font-weight: 700;
}

.event-dot {
  position: absolute;
  bottom: 4px;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent);
}

.day-cell.is-today .event-dot {
  background: var(--accent-ink);
}

.day-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 22px 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.day-view-date {
  font-size: 24px;
  font-weight: 600;
  color: var(--accent);
}

.day-view-week {
  font-size: 12.5px;
  color: var(--text-3);
}

.agenda {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px dashed var(--divider);
}

.agenda-caption {
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--text-4);
}

.agenda-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.agenda-list li {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 12.5px;
}

.agenda-time {
  flex: none;
  min-width: 50px;
  color: var(--accent);
  font-size: 12px;
}

.agenda-title {
  flex: 1;
  min-width: 0;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agenda-empty {
  color: var(--text-4);
  font-size: 12.5px;
}
</style>
