<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NModal, useMessage } from 'naive-ui'
import AppIcon from '../AppIcon.vue'
import { useWritingStore } from '../../stores/writing'
import { usePersonasStore } from '../../stores/personas'
import { useMaterialsStore } from '../../stores/materials'
import { useAiStore } from '../../stores/ai'
import { suggestionToPrompt } from '../../api/writing'
import type { ImageSuggestion } from '../../types'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ 'update:show': [boolean]; saved: [] }>()

const writing = useWritingStore()
const personas = usePersonasStore()
const materials = useMaterialsStore()
const ai = useAiStore()
const message = useMessage()

const instruction = ref('')
const pickedMaterialIds = ref<number[]>([])
const showHistory = ref(false)

const pickingMaterials = computed(() => materials.all.slice(0, 8))
const activePersona = computed(
  () => personas.all.find((item) => item.id === writing.personaId) ?? null,
)

watch(
  () => props.show,
  async (show) => {
    if (!show) return
    if (!personas.all.length) await personas.hydrate()
    if (!materials.all.length) await materials.hydrate()
    pickedMaterialIds.value = [...writing.materialIds]
  },
)

function toggleMaterial(id: number): void {
  pickedMaterialIds.value = pickedMaterialIds.value.includes(id)
    ? pickedMaterialIds.value.filter((item) => item !== id)
    : [...pickedMaterialIds.value, id]
}

function editContent(value: string): void {
  writing.versions = writing.versions.map((item, index) =>
    index === writing.activeIndex ? { ...item, content: value } : item,
  )
}

function reportError(): void {
  // 额度用完由全局弹窗处理，这里只提示其它错误
  if (writing.error && writing.error.code !== 'quota_exhausted') {
    message.error(writing.error.message)
  }
}

async function generate(): Promise<void> {
  writing.materialIds = pickedMaterialIds.value
  const ok = await writing.generate()

  if (ok) message.success(`已生成 ${writing.versions.length} 版，挑一版接着改`)
  else reportError()
}

async function revise(): Promise<void> {
  const text = instruction.value.trim()
  if (!text) {
    message.warning('说一句你想怎么改，比如「再口语化一点」')
    return
  }

  const ok = await writing.revise(text)
  instruction.value = ''

  if (ok) message.success('已按你的要求改好')
  else reportError()
}

async function save(): Promise<void> {
  const saved = await writing.save('ready')
  if (saved) {
    message.success('已保存到文案库，可以在「发布与经营」里找到')
    emit('saved')
  } else {
    message.error(writing.error?.message ?? '保存失败')
  }
}

async function copyImagePrompt(suggestion: ImageSuggestion): Promise<void> {
  const text = suggestionToPrompt(suggestion, writing.activeVersion?.title ?? '')
  try {
    await navigator.clipboard.writeText(text)
    message.success('配图提示词已复制')
  } catch {
    message.warning('复制失败，请手动选中提示词')
  }
}

function generateImage(suggestion: ImageSuggestion): void {
  message.info('图片生成模块还没接入，先把提示词复制给你')
  void copyImagePrompt(suggestion)
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :style="{ width: 'min(1200px, 96vw)' }"
    :bordered="false"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <template #header>
      <div class="studio-head">
        <span class="studio-icon"><AppIcon name="pen" :size="17" /></span>
        <div>
          <h2>AI 写作</h2>
          <p>左边定方向，右边改到你满意，然后存进文案库。</p>
        </div>
        <span class="chip">剩余体验 {{ ai.remaining }} 次</span>
      </div>
    </template>

    <div class="studio">
      <!-- 左侧输入区 -->
      <section class="studio-input">
        <label class="field-block">
          <span class="field-label">主题</span>
          <input v-model="writing.topic" class="field" placeholder="例如：失眠调理" />
        </label>

        <div class="field-block">
          <span class="field-label">文体</span>
          <div class="genre-row">
            <button
              v-for="item in writing.genres"
              :key="item.id"
              class="genre-chip"
              :class="{ 'is-active': writing.genre === item.id }"
              type="button"
              @click="writing.genre = item.id"
            >
              {{ item.label }}
            </button>
          </div>
        </div>

        <label class="field-block">
          <span class="field-label">人物小传</span>
          <select v-model="writing.personaId" class="field">
            <option :value="null">不使用（通用口吻）</option>
            <option v-for="item in personas.all" :key="item.id" :value="item.id">
              {{ item.name }}
            </option>
          </select>
          <span v-if="activePersona" class="field-hint">
            会把《{{ activePersona.name }}》的语气、受众和禁忌写进 system prompt
          </span>
        </label>

        <div class="field-block">
          <span class="field-label">素材引用（可选）</span>
          <div v-if="pickingMaterials.length" class="material-pick">
            <button
              v-for="item in pickingMaterials"
              :key="item.id"
              class="material-chip"
              :class="{ 'is-active': pickedMaterialIds.includes(item.id) }"
              type="button"
              @click="toggleMaterial(item.id)"
            >
              {{ item.name }}
            </button>
          </div>
          <span v-else class="field-hint">素材库还是空的，可以先去「找资料」传几个。</span>
        </div>

        <label class="field-block">
          <span class="field-label">补充你的想法</span>
          <textarea
            v-model="writing.extra"
            class="field"
            rows="3"
            placeholder="你自己的经历、口头语、一定要提到的点"
          />
        </label>

        <button class="btn btn-primary generate" type="button" :disabled="writing.generating" @click="generate">
          <AppIcon name="sparkles" :size="15" />
          {{ writing.generating ? '生成中…' : '生成 3 版' }}
        </button>

        <p class="field-hint">
          生成、修改、重新生成都会消耗一次调用；前 {{ ai.quotaLimit }} 次走免费体验额度。
        </p>
      </section>

      <!-- 右侧结果区 -->
      <section class="studio-result">
        <div v-if="!writing.hasResult" class="result-empty">
          <AppIcon name="pen" :size="28" />
          <p>填好左边的主题，点「生成 3 版」，这里会出现三个不同角度的版本。</p>
        </div>

        <template v-else>
          <div class="version-tabs">
            <button
              v-for="(item, index) in writing.versions"
              :key="index"
              class="version-tab"
              :class="{ 'is-active': writing.activeIndex === index }"
              type="button"
              @click="writing.activeIndex = index"
            >
              版本 {{ index + 1 }} · {{ item.angle }}
            </button>
          </div>

          <input
            v-if="writing.activeVersion"
            class="field version-title"
            :value="writing.activeVersion.title"
            placeholder="标题"
            @input="writing.versions = writing.versions.map((item, index) => index === writing.activeIndex ? { ...item, title: ($event.target as HTMLInputElement).value } : item)"
          />

          <textarea
            v-if="writing.activeVersion"
            class="field version-content"
            :value="writing.activeVersion.content"
            rows="12"
            @input="editContent(($event.target as HTMLTextAreaElement).value)"
          />

          <p v-if="writing.meta" class="result-meta">
            {{ writing.meta.providerLabel }} · {{ writing.meta.model }}
            <span v-if="writing.meta.usedTrialKey" class="chip">体验额度</span>
          </p>

          <!-- 对话式修改 -->
          <div class="revise">
            <div class="revise-row">
              <input
                v-model="instruction"
                class="field"
                placeholder="继续改：再口语化一点 / 开头再狠一点"
                @keydown.enter="revise"
              />
              <button class="btn btn-sm" type="button" :disabled="writing.revising" @click="revise">
                {{ writing.revising ? '修改中…' : '发送' }}
              </button>
            </div>
            <button
              v-if="writing.history.length"
              class="history-toggle"
              type="button"
              @click="showHistory = !showHistory"
            >
              修改历史（{{ writing.history.length }} 条）{{ showHistory ? '收起' : '展开' }}
            </button>
            <ul v-if="showHistory" class="history-list">
              <li v-for="(turn, index) in writing.history" :key="index" :class="turn.role">
                <span class="history-role">{{ turn.role === 'user' ? '我' : 'AI' }}</span>
                <span class="history-text">{{ turn.content.slice(0, 160) }}{{ turn.content.length > 160 ? '…' : '' }}</span>
              </li>
            </ul>
          </div>

          <!-- 配图建议 -->
          <div v-if="writing.imageKeywords.length" class="images">
            <div class="images-head">
              <AppIcon name="image" :size="15" />
              <strong>配图建议</strong>
              <span class="muted">共 {{ writing.imageKeywords.length }} 条</span>
            </div>
            <ul>
              <li v-for="(item, index) in writing.imageKeywords" :key="index">
                <div class="suggestion">
                  <span class="chip">{{ item.position }}</span>
                  <span class="suggestion-keywords">{{ item.keywords }}</span>
                  <span class="suggestion-meta">{{ item.style }} · {{ item.ratio }}</span>
                </div>
                <div class="suggestion-actions">
                  <button class="btn btn-sm btn-ghost" type="button" @click="copyImagePrompt(item)">
                    复制提示词
                  </button>
                  <button class="btn btn-sm" type="button" @click="generateImage(item)">
                    <AppIcon name="image" :size="13" />
                    生成图片
                  </button>
                </div>
              </li>
            </ul>
          </div>

          <div class="result-actions">
            <button class="btn btn-primary" type="button" :disabled="writing.saving" @click="save">
              <AppIcon name="check" :size="15" />
              {{ writing.saving ? '保存中…' : '保存到文案库' }}
            </button>
            <span class="muted">保存时会关联人物小传、文体和配图关键词</span>
          </div>
        </template>
      </section>
    </div>
  </NModal>
</template>

<style scoped>
.studio-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.studio-icon {
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

.studio-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
}

.studio-head p {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-3);
}

.studio-head .chip {
  margin-left: auto;
}

.studio {
  display: grid;
  grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
  gap: 18px;
}

.studio-input,
.studio-result {
  display: flex;
  flex-direction: column;
  gap: 13px;
  min-width: 0;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  color: var(--text-3);
  font-size: 12.5px;
}

.field-hint {
  color: var(--text-4);
  font-size: 11.5px;
  line-height: 1.6;
}

.genre-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.genre-chip {
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
}

.genre-chip.is-active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.material-pick {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.material-chip {
  max-width: 160px;
  padding: 4px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-3);
  font-family: inherit;
  font-size: 11.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.material-chip.is-active {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
}

.generate {
  align-self: flex-start;
}

.result-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 320px;
  padding: 24px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-card);
  color: var(--text-4);
  font-size: 12.5px;
  text-align: center;
}

.version-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.version-tab {
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.version-tab.is-active {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
}

.version-title {
  font-size: 14px;
  font-weight: 600;
}

.version-content {
  font-size: 13px;
  line-height: 1.85;
  min-height: 240px;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-4);
  font-size: 11.5px;
}

.revise {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.revise-row {
  display: flex;
  gap: 8px;
}

.history-toggle {
  align-self: flex-start;
  border: none;
  background: transparent;
  color: var(--text-4);
  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
  padding: 0;
}

.history-toggle:hover {
  color: var(--accent);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
  max-height: 160px;
  overflow-y: auto;
}

.history-list li {
  display: flex;
  gap: 8px;
  font-size: 11.5px;
  line-height: 1.6;
}

.history-role {
  flex: none;
  width: 22px;
  color: var(--text-4);
}

.history-list li.user .history-role {
  color: var(--accent);
}

.history-text {
  color: var(--text-3);
}

.images {
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding: 12px;
  border: 1px solid var(--accent-border);
  border-radius: 10px;
  background: var(--accent-softer);
}

.images-head {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent);
  font-size: 12.5px;
}

.images-head .muted {
  margin-left: auto;
  font-size: 11.5px;
}

.images ul {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.images li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 11px;
  border-radius: 9px;
  background: var(--bg-inset);
}

.suggestion {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.suggestion-keywords {
  flex: 1;
  min-width: 0;
  color: var(--text-2);
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suggestion-meta {
  flex: none;
  color: var(--text-4);
  font-size: 11.5px;
}

.suggestion-actions {
  display: flex;
  gap: 6px;
  flex: none;
}

.result-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px dashed var(--divider);
}

.result-actions .muted {
  font-size: 11.5px;
}

@media (max-width: 980px) {
  .studio {
    grid-template-columns: 1fr;
  }
}
</style>
