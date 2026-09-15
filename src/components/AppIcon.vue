<script setup lang="ts">
import { computed } from 'vue'

/**
 * 零依赖图标组件：内联 SVG（stroke 风格，1.7px 线宽，24 网格）。
 * 当前构建环境离线，无法安装 lucide-vue-next / @vicons/ionicons5，
 * 因此用内联图标代替，视觉上与线性图标库保持一致。
 */
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 })

const ICONS: Record<string, string> = {
  // 主导航
  flow: '<circle cx="5" cy="6" r="2.2"/><circle cx="19" cy="6" r="2.2"/><circle cx="12" cy="18.5" r="2.2"/><path d="M7.2 6h9.6M6.3 8 10.8 16.5M17.7 8 13.2 16.5"/>',
  home: '<path d="M4 10.6 12 4l8 6.6V20a1 1 0 0 1-1 1h-4.4v-6.2H9.4V21H5a1 1 0 0 1-1-1z"/>',
  flame:
    '<path d="M12 2.8c.6 2.7 2.3 3.8 3.7 5.4A6.5 6.5 0 0 1 17.6 13a5.6 5.6 0 0 1-11.2 0c0-1.6.6-2.8 1.6-3.9 1.4-1.6 3.4-2.6 4-6.3z"/><path d="M12 20.4a2.7 2.7 0 0 0 2.7-2.7c0-1.7-1.2-2.4-2.7-4.5-1.5 2.1-2.7 2.8-2.7 4.5A2.7 2.7 0 0 0 12 20.4z"/>',
  bulb:
    '<path d="M12 3a6 6 0 0 0-3.6 10.8c.7.5 1.1 1.3 1.1 2.2h5c0-.9.4-1.7 1.1-2.2A6 6 0 0 0 12 3z"/><path d="M9.8 19h4.4M10.6 21.4h2.8"/>',
  pen: '<path d="M16.4 3.6a2.2 2.2 0 0 1 3.1 3.1L7.2 19l-4.1 1 1-4.1z"/><path d="m14.4 5.6 3.1 3.1"/>',
  chart:
    '<path d="M3 20.2h18"/><rect x="5" y="12.5" width="3.4" height="5.6" rx="1"/><rect x="10.3" y="8" width="3.4" height="10.1" rx="1"/><rect x="15.6" y="4.2" width="3.4" height="13.9" rx="1"/>',
  calendar:
    '<rect x="3.2" y="5" width="17.6" height="16" rx="2.4"/><path d="M8 3v4M16 3v4M3.2 10.4h17.6"/>',
  gear: '<circle cx="12" cy="12" r="3.1"/><path d="M19.5 12c0-.5 0-1-.1-1.4l1.7-1.3-1.8-3.1-2 .8a7.6 7.6 0 0 0-2.4-1.4L14.6 3.4h-3.6l-.3 2.2c-.9.3-1.7.8-2.4 1.4l-2-.8L4.5 9.3l1.7 1.3a7.7 7.7 0 0 0 0 2.8L4.5 14.7l1.8 3.1 2-.8c.7.6 1.5 1.1 2.4 1.4l.3 2.2h3.6l.3-2.2c.9-.3 1.7-.8 2.4-1.4l2 .8 1.8-3.1-1.7-1.3c.1-.4.1-.9.1-1.4z"/>',
  // 操作
  play: '<path d="M7 4.8 19 12 7 19.2z"/>',
  pause: '<rect x="7" y="5" width="3.6" height="14" rx="1.2"/><rect x="13.4" y="5" width="3.6" height="14" rx="1.2"/>',
  reset: '<path d="M20 12a8 8 0 1 1-2.6-5.9"/><path d="M20 4v4.4h-4.4"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  check: '<path d="m4.5 12.6 5 5L19.5 6.8"/>',
  close: '<path d="M6 6l12 12M18 6 6 18"/>',
  trash:
    '<path d="M4.5 6.8h15M9.4 6.8V4.6h5.2v2.2M6.6 6.8l.9 12.9h9l.9-12.9"/><path d="M10.4 10.4v6M13.6 10.4v6"/>',
  search: '<circle cx="11" cy="11" r="6.4"/><path d="m16 16 4.4 4.4"/>',
  chevronLeft: '<path d="m14.4 6-6 6 6 6"/>',
  chevronRight: '<path d="m9.6 6 6 6-6 6"/>',
  arrowRight: '<path d="M4.5 12h15M13.5 6l6 6-6 6"/>',
  // 卡片语义
  clock: '<circle cx="12" cy="12" r="8.6"/><path d="M12 7.4V12l3.2 2"/>',
  sparkles:
    '<path d="M12 3.6l1.7 4.7 4.7 1.7-4.7 1.7L12 16.4l-1.7-4.7L5.6 10l4.7-1.7z"/><path d="M18.6 15.4l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z"/>',
  image:
    '<rect x="3.4" y="5" width="17.2" height="14" rx="2.4"/><circle cx="8.8" cy="10" r="1.7"/><path d="m4.6 17.4 4.6-4.4 3.4 3.2 2.6-2.4 4.2 3.8"/>',
  book:
    '<path d="M4.4 5.4A1.8 1.8 0 0 1 6.2 3.6h12.4v16.8H6.2a1.8 1.8 0 0 1-1.8-1.8z"/><path d="M8 3.6v16.8"/>',
  layers:
    '<path d="m12 3.6 8.4 4.4-8.4 4.4-8.4-4.4z"/><path d="m4.4 12.6 7.6 4 7.6-4"/><path d="m4.4 16.6 7.6 4 7.6-4"/>',
  inbox:
    '<path d="M3.6 12.6 6 5.4A1.8 1.8 0 0 1 7.7 4.2h8.6A1.8 1.8 0 0 1 18 5.4l2.4 7.2v6a1.8 1.8 0 0 1-1.8 1.8H5.4a1.8 1.8 0 0 1-1.8-1.8z"/><path d="M3.6 12.6h4.8l1.2 2.6h4.8l1.2-2.6h4.8"/>',
  video:
    '<rect x="3" y="6.4" width="12.6" height="11.2" rx="2.4"/><path d="m15.6 11 5.4-3.4v8.8L15.6 13z"/>',
  link: '<path d="M10.4 13.6a3.8 3.8 0 0 0 5.4 0l2.6-2.6a3.8 3.8 0 0 0-5.4-5.4L11.6 7"/><path d="M13.6 10.4a3.8 3.8 0 0 0-5.4 0l-2.6 2.6a3.8 3.8 0 0 0 5.4 5.4l1.4-1.4"/>',
  text: '<path d="M5 5.4h14M9.4 5.4v13.2M5 18.6h14"/><path d="M12 12h7"/>',
  target:
    '<circle cx="12" cy="12" r="8.4"/><circle cx="12" cy="12" r="4.6"/><circle cx="12" cy="12" r="1.1"/>',
  trend:
    '<path d="M3.4 16.8 9 11l3.4 3.4 7.4-7.4"/><path d="M15.6 7h4.2v4.2"/>',
  users:
    '<circle cx="9.4" cy="8.6" r="3.4"/><path d="M3.4 19.6a6 6 0 0 1 12 0"/><path d="M16.4 5.6a3.4 3.4 0 0 1 0 6.4M17.6 19.6a6 6 0 0 0-1.6-4.1"/>',
  list: '<path d="M8.4 6.6h12M8.4 12h12M8.4 17.4h12"/><path d="M3.6 6.6h.01M3.6 12h.01M3.6 17.4h.01"/>',
  compass:
    '<circle cx="12" cy="12" r="8.6"/><path d="m15.4 8.6-2 4.8-4.8 2 2-4.8z"/>',
  coffee:
    '<path d="M4.6 8.4h12v5.4a4.4 4.4 0 0 1-4.4 4.4H9a4.4 4.4 0 0 1-4.4-4.4z"/><path d="M16.6 9.6h1.6a2.4 2.4 0 0 1 0 4.8h-1.6"/><path d="M4.6 21h12"/>',
  edit: '<path d="M4 20h4l10.4-10.4-4-4L4 16z"/><path d="m13.6 6.4 4 4"/>',
}

const path = computed(() => ICONS[props.name] ?? ICONS.text)
</script>

<template>
  <svg
    class="app-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.7"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="path"
  />
</template>
