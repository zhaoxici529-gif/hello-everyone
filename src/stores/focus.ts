import { computed, onScopeDispose, ref } from 'vue'
import { defineStore } from 'pinia'
import { localDateTime } from '../utils/datetime'

const FOCUS_SECONDS = 25 * 60
const BREAK_SECONDS = 5 * 60

export const useFocusStore = defineStore('focus', () => {
  const mode = ref<'focus' | 'break'>('focus')
  const running = ref(false)
  const remaining = ref(FOCUS_SECONDS)
  const selectedTask = ref('')
  const completedRounds = ref(0)
  /** 可配置的专注时长（分钟） */
  const focusMinutes = ref(25)
  /** 本轮开始时间，用于写 pomodoro 记录 */
  const startedAt = ref('')
  /** 最近一次记录的反馈 */
  const lastRecord = ref('')
  let timer: ReturnType<typeof setInterval> | null = null

  const duration = computed(() =>
    mode.value === 'focus' ? focusMinutes.value * 60 : BREAK_SECONDS,
  )
  const progress = computed(() => 1 - remaining.value / duration.value)

  const clockText = computed(() => {
    const minutes = Math.floor(remaining.value / 60)
    const seconds = remaining.value % 60
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
  })

  function stopTimer(): void {
    if (timer !== null) {
      clearInterval(timer)
      timer = null
    }
  }

  function tick(): void {
    if (remaining.value > 0) {
      remaining.value -= 1
      return
    }
    // 一轮结束：专注结束进入休息，休息结束回到专注。
    if (mode.value === 'focus') {
      completedRounds.value += 1
      void recordSession()
    }
    mode.value = mode.value === 'focus' ? 'break' : 'focus'
    remaining.value = duration.value
    running.value = false
    stopTimer()
  }

  function start(): void {
    if (running.value) return
    if (!startedAt.value) startedAt.value = localDateTime()
    running.value = true
    stopTimer()
    timer = setInterval(tick, 1000)
  }

  function pause(): void {
    running.value = false
    stopTimer()
  }

  function reset(): void {
    pause()
    startedAt.value = ''
    remaining.value = duration.value
  }

  function setMinutes(minutes: number): void {
    focusMinutes.value = Math.min(120, Math.max(5, Math.round(minutes)))
    pause()
    startedAt.value = ''
    remaining.value = duration.value
  }

  /** 一轮专注结束 → 记进 pomodoro 表 */
  async function recordSession(): Promise<void> {
    const started = startedAt.value || localDateTime()
    startedAt.value = ''

    const { dbInsert } = await import('../api/db')
    const saved = await dbInsert('pomodoro', {
      task: selectedTask.value.trim() || '未命名专注',
      duration: focusMinutes.value * 60,
      started_at: started,
      finished_at: localDateTime(),
    })

    lastRecord.value = saved ? `已记录：${selectedTask.value.trim() || '未命名专注'}` : '记录失败（浏览器预览模式）'
  }

  function toggle(): void {
    running.value ? pause() : start()
  }

  onScopeDispose(stopTimer)

  return {
    mode,
    running,
    remaining,
    selectedTask,
    completedRounds,
    focusMinutes,
    lastRecord,
    duration,
    progress,
    clockText,
    start,
    pause,
    reset,
    toggle,
    setMinutes,
    recordSession,
  }
})
