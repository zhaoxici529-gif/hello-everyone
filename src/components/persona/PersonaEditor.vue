<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { NModal, useMessage } from 'naive-ui'
import AppIcon from '../AppIcon.vue'
import TraitInput from '../TagInput.vue'
import { usePersonasStore } from '../../stores/personas'
import { useAiStore } from '../../stores/ai'
import {
  emptyPersonaFields,
  parsePersonaFields,
  type Persona,
  type PersonaCategory,
  type PersonaFields,
} from '../../types'

const props = defineProps<{ show: boolean; persona: Persona | null }>()
const emit = defineEmits<{ 'update:show': [boolean]; saved: [Persona] }>()

const personas = usePersonasStore()
const ai = useAiStore()
const message = useMessage()

const form = reactive({
  name: '',
  category: 'self_ip' as PersonaCategory,
  fields: emptyPersonaFields(),
})

const sourceText = ref('')
const extracting = ref(false)
const saving = ref(false)
const showExtract = ref(false)
const extractNote = ref('')

const title = computed(() => (props.persona ? '编辑人物档案' : '新建人物档案'))
const isEditing = computed(() => Boolean(props.persona))

const CATEGORIES: Array<{ id: PersonaCategory; label: string; hint: string }> = [
  { id: 'self_ip', label: '自我 IP', hint: '你自己 / 主理人' },
  { id: 'target_customer', label: '目标客户', hint: '你要打动的人' },
  { id: 'case', label: '案例人物', hint: '学员 / 真实故事' },
]

watch(
  () => [props.show, props.persona] as const,
  ([show, persona]) => {
    if (!show) return
    form.name = persona?.name ?? ''
    form.category = (persona?.category as PersonaCategory) ?? 'self_ip'
    form.fields = persona ? parsePersonaFields(persona.fields) : emptyPersonaFields()
    sourceText.value = ''
    extractNote.value = ''
  },
  { immediate: true },
)

/** 只把提取到的非空字段填进去，不冲掉用户已经写好的内容 */
function mergeFields(target: PersonaFields, incoming: PersonaFields): number {
  const stringKeys = [
    'age',
    'identity',
    'experience',
    'catchphrases',
    'expressions',
    'audience',
    'painPoints',
    'taboos',
    'viewpoints',
    'visualTone',
    'visualStyle',
    'visualScene',
    'sourceNote',
  ] as const

  let filled = 0
  for (const key of stringKeys) {
    const value = incoming[key]
    if (value.trim()) {
      target[key] = value.trim()
      filled += 1
    }
  }

  if (incoming.traits.length) {
    target.traits = [...incoming.traits].slice(0, 5)
    filled += 1
  }

  return filled
}

async function runExtraction(): Promise<void> {
  if (sourceText.value.trim().length < 20) {
    message.warning('至少粘贴 20 个字的文案，提取才有意义')
    return
  }

  extracting.value = true
  extractNote.value = ''
  const fields = await personas.extract(sourceText.value, form.name || undefined)
  extracting.value = false

  if (!fields) {
    if (personas.error && personas.error.code !== 'quota_exhausted') {
      message.error(personas.error.message)
    }
    return
  }

  const filled = mergeFields(form.fields, fields)
  const result = personas.lastExtraction

  extractNote.value = [
    `已填充 ${filled} 组字段`,
    result ? `模型 ${result.providerLabel} · ${result.model}` : '',
    result?.usedTrialKey ? '本次使用免费体验额度' : '',
  ]
    .filter(Boolean)
    .join(' · ')

  message.success('已提取完成，请核对后再保存')
}

async function save(): Promise<void> {
  if (!form.name.trim()) {
    message.warning('先给这个人物起个名字')
    return
  }

  saving.value = true
  const payload = {
    name: form.name.trim(),
    category: form.category,
    fields: { ...form.fields, traits: [...form.fields.traits] },
  }

  const saved = isEditing.value && props.persona
    ? await personas.update(props.persona.id, payload)
    : await personas.create(payload)

  saving.value = false

  if (!saved) {
    message.error('保存失败，请重试')
    return
  }

  message.success(isEditing.value ? '档案已更新' : '档案已创建')
  emit('saved', saved)
  emit('update:show', false)
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :style="{ width: 'min(940px, 94vw)' }"
    :bordered="false"
    :mask-closable="false"
    @update:show="(value: boolean) => emit('update:show', value)"
  >
    <template #header>
      <div class="editor-head">
        <span class="editor-icon"><AppIcon name="users" :size="17" /></span>
        <div>
          <h2>{{ title }}</h2>
          <p>填得越具体，AI 写出来的内容越像这个人说的话。</p>
        </div>
      </div>
    </template>

    <!-- AI 提取 -->
    <section class="extract">
      <button class="extract-toggle" type="button" @click="showExtract = !showExtract">
        <AppIcon name="sparkles" :size="15" />
        AI 提取特征：粘贴一段已有文案，自动填进下面的字段
        <span class="extract-arrow">{{ showExtract ? '收起' : '展开' }}</span>
      </button>

      <div v-if="showExtract" class="extract-body">
        <textarea
          v-model="sourceText"
          class="field"
          rows="5"
          placeholder="粘贴一段你写过的文案、课程稿、访谈记录，或者学员发给你的长消息。"
        />
        <div class="extract-actions">
          <button class="btn btn-primary btn-sm" type="button" :disabled="extracting" @click="runExtraction">
            <AppIcon name="sparkles" :size="14" />
            {{ extracting ? '提取中…' : 'AI 提取特征' }}
          </button>
          <span class="muted">剩余体验 {{ ai.remaining }} 次 · 只填充空字段，不覆盖你写的内容</span>
        </div>
        <p v-if="extractNote" class="extract-note">{{ extractNote }}</p>
      </div>
    </section>

    <!-- 表单 -->
    <div class="form-grid">
      <label class="field-block">
        <span class="field-label">姓名 / 昵称 <i>*</i></span>
        <input v-model="form.name" class="field" placeholder="例如：我自己（心身同调主理人）" />
      </label>

      <label class="field-block">
        <span class="field-label">年龄</span>
        <input v-model="form.fields.age" class="field" placeholder="例如：38 岁" />
      </label>

      <div class="field-block is-wide">
        <span class="field-label">分类</span>
        <div class="category-row">
          <button
            v-for="item in CATEGORIES"
            :key="item.id"
            class="category-option"
            :class="{ 'is-active': form.category === item.id }"
            type="button"
            @click="form.category = item.id"
          >
            <strong>{{ item.label }}</strong>
            <small>{{ item.hint }}</small>
          </button>
        </div>
      </div>

      <label class="field-block is-wide">
        <span class="field-label">身份标签</span>
        <input
          v-model="form.fields.identity"
          class="field"
          placeholder="多个用「、」分隔，例如：心理咨询师、两个孩子的妈妈"
        />
      </label>

      <label class="field-block is-wide">
        <span class="field-label">核心经历</span>
        <textarea
          v-model="form.fields.experience"
          class="field"
          rows="3"
          placeholder="他/她经历过什么，为什么会变成现在这样。2-3 句就够。"
        />
      </label>

      <div class="field-block is-wide">
        <span class="field-label">性格特点（3-5 个关键词）</span>
        <TraitInput
          v-model="form.fields.traits"
          :max="5"
          recommend="建议 3-5 个"
          placeholder="例如：温和、直接、爱举例"
        />
      </div>

      <label class="field-block">
        <span class="field-label">口头禅</span>
        <input v-model="form.fields.catchphrases" class="field" placeholder="例如：你先别急着讲道理" />
      </label>

      <label class="field-block">
        <span class="field-label">常用表达</span>
        <input v-model="form.fields.expressions" class="field" placeholder="例如：喜欢用「我举个例子啊」开头" />
      </label>

      <label class="field-block">
        <span class="field-label">目标受众是谁</span>
        <input v-model="form.fields.audience" class="field" placeholder="例如：30-45 岁被失眠困扰的女性" />
      </label>

      <label class="field-block">
        <span class="field-label">受众的痛点</span>
        <input v-model="form.fields.painPoints" class="field" placeholder="例如：道理都懂但做不到，越努力越自责" />
      </label>

      <label class="field-block is-wide">
        <span class="field-label">禁忌话题</span>
        <textarea
          v-model="form.fields.taboos"
          class="field"
          rows="2"
          placeholder="不能碰的话题，例如：不聊具体用药、不做诊断、不承诺疗效"
        />
      </label>

      <label class="field-block is-wide">
        <span class="field-label">代表观点</span>
        <textarea
          v-model="form.fields.viewpoints"
          class="field"
          rows="2"
          placeholder="这个人反复强调的观点，例如：先安顿身体，再谈情绪"
        />
      </label>

      <div class="field-block is-wide">
        <span class="field-label">视觉风格偏好（用于后续图片生成）</span>
        <div class="visual-row">
          <input v-model="form.fields.visualTone" class="field" placeholder="色调，例如：暖米色、浅木色" />
          <input v-model="form.fields.visualStyle" class="field" placeholder="风格，例如：自然光、纪实感" />
          <input v-model="form.fields.visualScene" class="field" placeholder="场景，例如：清晨的窗边" />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="editor-foot">
        <span class="muted">
          {{ form.fields.traits.length }} 个性格关键词 · 保存后会写进 personas 表
        </span>
        <div class="editor-actions">
          <button class="btn" type="button" @click="emit('update:show', false)">取消</button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="save">
            <AppIcon name="check" :size="15" />
            {{ saving ? '保存中…' : '保存档案' }}
          </button>
        </div>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.editor-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.editor-icon {
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

.editor-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
}

.editor-head p {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-3);
}

.extract {
  margin-bottom: 16px;
  border: 1px dashed var(--border-strong);
  border-radius: 10px;
  overflow: hidden;
}

.extract-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 11px 13px;
  border: none;
  background: transparent;
  color: var(--accent);
  font-family: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.extract-arrow {
  margin-left: auto;
  color: var(--text-4);
  font-size: 11.5px;
}

.extract-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 13px 13px;
}

.extract-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.extract-actions .muted {
  font-size: 11.5px;
}

.extract-note {
  padding: 8px 11px;
  border-radius: 8px;
  background: var(--accent-softer);
  color: var(--accent);
  font-size: 12px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 13px;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.field-block.is-wide {
  grid-column: span 2;
}

.field-label {
  color: var(--text-3);
  font-size: 12.5px;
}

.field-label i {
  color: var(--danger);
  font-style: normal;
}

.category-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.category-option {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.category-option strong {
  font-size: 13px;
  color: var(--text-1);
}

.category-option small {
  color: var(--text-4);
  font-size: 11.5px;
}

.category-option.is-active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.category-option.is-active strong {
  color: var(--accent);
}

.visual-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.editor-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.editor-foot .muted {
  font-size: 11.5px;
}

.editor-actions {
  display: flex;
  gap: 8px;
}

@media (max-width: 720px) {
  .form-grid,
  .category-row,
  .visual-row {
    grid-template-columns: 1fr;
  }

  .field-block.is-wide {
    grid-column: span 1;
  }
}
</style>
