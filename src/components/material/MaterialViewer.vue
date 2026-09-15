<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NModal, useMessage } from 'naive-ui'
import AppIcon from '../AppIcon.vue'
import TagInput from '../TagInput.vue'
import { fileUrl } from '../../utils/asset'
import { readMaterialText } from '../../api/materials'
import { useMaterialsStore } from '../../stores/materials'
import {
  MATERIAL_TYPES,
  formatBytes,
  parseMaterialTags,
  type Material,
  type MaterialPurpose,
} from '../../types'

const props = defineProps<{ show: boolean; material: Material | null }>()
const emit = defineEmits<{ 'update:show': [boolean]; changed: [] }>()

const materials = useMaterialsStore()
const message = useMessage()

const tags = ref<string[]>([])
const description = ref('')
const textContent = ref('')
const saving = ref(false)
const describing = ref(false)

const typeMeta = computed(
  () => MATERIAL_TYPES.find((item) => item.key === props.material?.type) ?? MATERIAL_TYPES[3],
)
const previewUrl = computed(() => fileUrl(props.material?.thumb_path ?? props.material?.file_path))
const playUrl = computed(() => fileUrl(props.material?.file_path))
const categoryName = computed(() => {
  const id = props.material?.category_id
  return materials.categories.find((item) => item.id === id)?.name ?? '未分类'
})

watch(
  () => [props.show, props.material?.id] as const,
  async ([show]) => {
    if (!show || !props.material) return
    tags.value = parseMaterialTags(props.material.tags)
    description.value = props.material.description ?? ''
    textContent.value = ''

    if (props.material.type === 'text') {
      textContent.value = await readMaterialText(props.material.id).catch(
        () => '（当前环境读不到文件内容）',
      )
    }
  },
  { immediate: true },
)

async function save(): Promise<void> {
  if (!props.material) return
  saving.value = true
  await materials.updateMeta(props.material.id, {
    tags: tags.value.join(','),
    description: description.value,
  })
  saving.value = false
  message.success('已保存')
  emit('changed')
}

async function runDescribe(): Promise<void> {
  if (!props.material) return
  describing.value = true
  const result = await materials.describe(props.material.id)
  describing.value = false

  if (!result) {
    message.error(materials.error?.message ?? '生成失败')
    return
  }

  description.value = result.description
  tags.value = parseMaterialTags(result.material.tags)
  emit('changed')

  message.success(
    result.sentImage
      ? `已根据${typeMeta.value.label}生成描述`
      : '已生成描述（这次没附带图片，换支持看图的模型效果更好）',
  )
}

async function copyPrompt(purpose: MaterialPurpose): Promise<void> {
  if (!props.material) return
  const block = await materials.promptBlock(props.material.id, purpose)
  if (!block) {
    message.error(materials.error?.message ?? '取提示词失败')
    return
  }

  try {
    await navigator.clipboard.writeText(block.text)
    message.success(`已复制${block.purposeLabel}提示词`)
  } catch {
    message.warning('复制失败，请手动选中提示词文本')
  }
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :style="{ width: 'min(980px, 94vw)' }"
    :bordered="false"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <template #header>
      <div class="viewer-head">
        <span class="viewer-icon"><AppIcon :name="typeMeta.icon" :size="17" /></span>
        <div>
          <h2>{{ material?.name }}</h2>
          <p>
            {{ typeMeta.label }} · {{ formatBytes(material?.size_bytes ?? 0) }} · {{ categoryName }}
          </p>
        </div>
      </div>
    </template>

    <div v-if="material" class="viewer">
      <div class="viewer-stage">
        <img
          v-if="material.type === 'image' && previewUrl"
          :src="previewUrl"
          :alt="material.name"
          class="stage-image"
        />
        <video v-else-if="material.type === 'video' && playUrl" :src="playUrl" class="stage-video" controls />
        <div v-else-if="material.type === 'audio' && playUrl" class="stage-audio">
          <AppIcon name="clock" :size="34" />
          <audio :src="playUrl" controls />
        </div>
        <pre v-else-if="material.type === 'text'" class="stage-text">{{ textContent }}</pre>
        <div v-else class="stage-fallback">
          <AppIcon :name="typeMeta.icon" :size="34" />
          <p>当前环境无法预览这个文件，素材本身已经保存在素材库里。</p>
        </div>
      </div>

      <div class="viewer-side">
        <div class="field-block">
          <span class="field-label">标签</span>
          <TagInput v-model="tags" :max="12" placeholder="输入标签后回车，例如：封面" />
        </div>

        <div class="field-block">
          <span class="field-label">描述</span>
          <textarea
            v-model="description"
            class="field"
            rows="4"
            placeholder="这条素材是什么、适合用在哪里"
          />
        </div>

        <div class="viewer-actions">
          <button class="btn btn-primary btn-sm" type="button" :disabled="saving" @click="save">
            <AppIcon name="check" :size="14" />
            保存
          </button>
          <button class="btn btn-sm" type="button" :disabled="describing" @click="runDescribe">
            <AppIcon name="sparkles" :size="14" />
            {{ describing ? '生成中…' : 'AI 生成描述' }}
          </button>
        </div>

        <div class="viewer-prompts">
          <span class="field-label">给其它模块用</span>
          <div class="prompt-row">
            <button class="btn btn-sm btn-ghost" type="button" @click="copyPrompt('writing')">
              <AppIcon name="pen" :size="14" />
              写作提示词
            </button>
            <button class="btn btn-sm btn-ghost" type="button" @click="copyPrompt('image')">
              <AppIcon name="image" :size="14" />
              图片提示词
            </button>
            <button class="btn btn-sm btn-ghost" type="button" @click="copyPrompt('video')">
              <AppIcon name="video" :size="14" />
              剪辑提示词
            </button>
          </div>
        </div>

        <p class="viewer-path">{{ material.file_path }}</p>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.viewer-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.viewer-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 11px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  flex: none;
}

.viewer-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
}

.viewer-head p {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-3);
}

.viewer {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(280px, 1fr);
  gap: 16px;
}

.viewer-stage {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 320px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
  overflow: hidden;
}

.stage-image,
.stage-video {
  max-width: 100%;
  max-height: 460px;
  border-radius: 8px;
}

.stage-audio {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--accent);
}

.stage-text {
  align-self: stretch;
  width: 100%;
  max-height: 460px;
  margin: 0;
  overflow: auto;
  color: var(--text-2);
  font-family: var(--font-ui);
  font-size: 12.5px;
  line-height: 1.8;
  white-space: pre-wrap;
  word-break: break-word;
}

.stage-fallback {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-4);
  font-size: 12.5px;
  text-align: center;
}

.viewer-side {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.field-label {
  color: var(--text-3);
  font-size: 12.5px;
}

.viewer-actions,
.prompt-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.viewer-prompts {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding-top: 12px;
  border-top: 1px dashed var(--divider);
}

.viewer-path {
  color: var(--text-4);
  font-family: var(--font-mono);
  font-size: 10.5px;
  word-break: break-all;
}

@media (max-width: 860px) {
  .viewer {
    grid-template-columns: 1fr;
  }
}
</style>
