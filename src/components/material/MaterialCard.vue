<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '../AppIcon.vue'
import { fileUrl } from '../../utils/asset'
import {
  MATERIAL_TYPES,
  formatBytes,
  parseMaterialTags,
  type Material,
} from '../../types'

const props = defineProps<{ material: Material; view: 'grid' | 'list' }>()

const emit = defineEmits<{
  open: [Material]
  describe: [Material]
  remove: [Material]
}>()

const typeMeta = computed(
  () => MATERIAL_TYPES.find((item) => item.key === props.material.type) ?? MATERIAL_TYPES[3],
)
const tags = computed(() => parseMaterialTags(props.material.tags))

/** 图片用缩略图，视频用抓到的首帧 */
const previewUrl = computed(() => fileUrl(props.material.thumb_path ?? props.material.file_path))
const canPreview = computed(
  () => Boolean(previewUrl.value) && (props.material.type === 'image' || props.material.type === 'video'),
)
</script>

<template>
  <article class="material-card" :class="{ 'is-list': view === 'list' }">
    <button class="material-preview" type="button" :title="'查看 ' + material.name" @click="emit('open', material)">
      <img v-if="canPreview" :src="previewUrl" :alt="material.name" loading="lazy" />
      <span v-else class="material-placeholder">
        <AppIcon :name="typeMeta.icon" :size="26" />
      </span>

      <span class="material-type">{{ typeMeta.label }}</span>
      <span v-if="material.type === 'video' || material.type === 'audio'" class="material-play">
        <AppIcon name="play" :size="16" />
      </span>
    </button>

    <div class="material-body">
      <h3 class="material-name" :title="material.name">{{ material.name }}</h3>

      <p v-if="material.description" class="material-desc">{{ material.description }}</p>

      <div v-if="tags.length" class="material-tags">
        <span v-for="tag in tags.slice(0, 4)" :key="tag" class="tag">{{ tag }}</span>
        <span v-if="tags.length > 4" class="tag is-more">+{{ tags.length - 4 }}</span>
      </div>

      <div class="material-meta">
        <span class="num">{{ formatBytes(material.size_bytes) }}</span>
        <span v-if="material.ai_description_at" class="chip">AI 已描述</span>
      </div>
    </div>

    <footer class="material-actions">
      <button class="btn btn-sm" type="button" @click="emit('open', material)">
        <AppIcon name="search" :size="14" />
        查看
      </button>
      <button class="btn btn-sm btn-ghost" type="button" @click="emit('describe', material)">
        <AppIcon name="sparkles" :size="14" />
        AI 描述
      </button>
      <button class="btn btn-sm btn-ghost material-delete" type="button" @click="emit('remove', material)">
        <AppIcon name="trash" :size="14" />
      </button>
    </footer>
  </article>
</template>

<style scoped>
.material-card {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-card);
  box-shadow: var(--shadow-card);
  overflow: hidden;
  transition: border-color 0.16s ease, transform 0.16s ease;
}

.material-card:hover {
  border-color: var(--accent-border);
  transform: translateY(-1px);
}

.material-preview {
  position: relative;
  display: block;
  width: 100%;
  aspect-ratio: 16 / 10;
  padding: 0;
  border: none;
  background: var(--bg-inset);
  cursor: pointer;
  overflow: hidden;
}

.material-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.material-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: var(--text-4);
}

.material-type {
  position: absolute;
  top: 9px;
  left: 9px;
  padding: 2px 9px;
  border-radius: var(--radius-pill);
  background: rgba(15, 18, 16, 0.78);
  color: var(--accent);
  font-size: 11px;
}

.material-play {
  position: absolute;
  right: 9px;
  bottom: 9px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: rgba(184, 212, 168, 0.9);
  color: var(--accent-ink);
}

.material-body {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 13px 14px;
  flex: 1;
}

.material-name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.material-desc {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  color: var(--text-3);
  font-size: 12px;
  line-height: 1.6;
}

.material-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.tag {
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
}

.tag.is-more {
  background: var(--bg-inset);
  color: var(--text-4);
}

.material-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-4);
  font-size: 11.5px;
}

.material-actions {
  display: flex;
  gap: 7px;
  padding: 10px 14px;
  border-top: 1px dashed var(--divider);
}

.material-delete {
  margin-left: auto;
  color: var(--danger);
}

/* 列表视图：缩略图在左，信息在右 */
.material-card.is-list {
  flex-direction: row;
  align-items: stretch;
}

.material-card.is-list .material-preview {
  width: 176px;
  aspect-ratio: auto;
  flex: none;
}

.material-card.is-list .material-body {
  padding: 12px 14px;
}

.material-card.is-list .material-actions {
  flex-direction: column;
  border-top: none;
  border-left: 1px dashed var(--divider);
  padding: 12px;
}

.material-card.is-list .material-delete {
  margin-left: 0;
}
</style>
