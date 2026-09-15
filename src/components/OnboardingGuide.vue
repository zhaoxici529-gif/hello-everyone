<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NModal, useMessage } from 'naive-ui'
import AppIcon from './AppIcon.vue'
import { useAiStore } from '../stores/ai'
import { usePersonasStore } from '../stores/personas'
import { settingSet } from '../api/db'
import { emptyPersonaFields } from '../types'

/**
 * 首次启动引导。三步：欢迎 → 填 Key（可跳过，先用体验额度）→ 建第一个人物小传。
 * 完成后写 settings.onboarded=1，之后不再弹出。
 */
const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ 'update:show': [boolean]; done: [] }>()

const ai = useAiStore()
const personas = usePersonasStore()
const message = useMessage()

const step = ref(0)
const keyDraft = ref('')
const personaName = ref('')
const personaIdentity = ref('')
const saving = ref(false)

const providerId = computed(() => ai.selectedProvider)
const currentKey = computed(() => ai.keys.find((item) => item.provider === providerId.value))

watch(
  () => props.show,
  async (show) => {
    if (!show) return
    step.value = 0
    keyDraft.value = ''
    personaName.value = ''
    personaIdentity.value = ''
    if (!ai.providers.length) await ai.load()
    if (!personas.all.length) await personas.hydrate()
  },
)

async function finish(): Promise<void> {
  await settingSet('onboarded', '1')
  emit('update:show', false)
  emit('done')
  message.success('设置完成，开始创作吧')
}

async function saveKey(): Promise<void> {
  if (!keyDraft.value.trim()) {
    message.warning('先粘贴 API Key，或者点「先用免费体验额度」跳过')
    return
  }
  const ok = await ai.saveKey(providerId.value, keyDraft.value)
  if (ok) {
    message.success('Key 已加密保存到本地')
    step.value = 2
  } else {
    message.error(ai.lastError?.message ?? '保存失败')
  }
}

async function createPersona(): Promise<void> {
  if (!personaName.value.trim()) {
    message.warning('给这个人物起个名字')
    return
  }

  saving.value = true
  const created = await personas.create({
    name: personaName.value.trim(),
    category: 'self_ip',
    fields: { ...emptyPersonaFields(), identity: personaIdentity.value.trim() },
  })
  saving.value = false

  if (created) await finish()
  else message.error('创建失败，可以稍后在「人物小传」里补上')
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :style="{ width: 'min(680px, 94vw)' }"
    :bordered="false"
    :mask-closable="false"
    :closable="false"
  >
    <template #header>
      <div class="onboard-head">
        <span class="onboard-icon">心</span>
        <div>
          <h2>欢迎使用自媒体 AI 工作台</h2>
          <p>三步就好，全程不用懂技术。</p>
        </div>
        <span class="chip">{{ step + 1 }} / 3</span>
      </div>
    </template>

    <!-- 第一步：欢迎 -->
    <section v-if="step === 0" class="step">
      <p class="lead">
        这是一个只在你电脑上跑的内容创作工作台：人物小传、素材库、AI 写作、文案管理都在本地，
        数据存在你自己的电脑里，不会上传。
      </p>
      <ul class="points">
        <li>
          <AppIcon name="sparkles" :size="15" />
          <span>内置 <b>{{ ai.quotaLimit }} 次免费体验</b>，不填 Key 也能先试</span>
        </li>
        <li>
          <AppIcon name="users" :size="15" />
          <span>填一份人物小传，AI 写出来的内容就会像你自己说的话</span>
        </li>
        <li>
          <AppIcon name="layers" :size="15" />
          <span>数据存在本地数据库，随时可以导出备份</span>
        </li>
      </ul>
    </section>

    <!-- 第二步：API Key -->
    <section v-else-if="step === 1" class="step">
      <p class="lead">填一个自己的 API Key，AI 功能就不限次数了（也可以先跳过，用免费体验）。</p>

      <div class="provider-row">
        <button
          v-for="item in ai.providers"
          :key="item.id"
          class="provider-chip"
          :class="{ 'is-active': providerId === item.id }"
          type="button"
          @click="ai.selectProvider(item.id)"
        >
          {{ item.label }}
        </button>
      </div>

      <input
        v-model="keyDraft"
        class="field"
        type="password"
        autocomplete="off"
        :placeholder="currentKey?.hasKey ? '已保存，粘贴新的会覆盖' : '把复制的 API Key 粘贴到这里'"
      />

      <div class="key-actions">
        <button class="btn btn-primary" type="button" @click="saveKey">
          <AppIcon name="check" :size="15" />
          保存并测试
        </button>
        <button class="btn btn-ghost" type="button" @click="step = 2">
          先用免费体验额度（{{ ai.remaining }} 次）
        </button>
      </div>

      <p class="hint">
        不知道去哪拿 Key？点开设置页的「AI 模型与密钥」，里面有三步图文教程。
      </p>
    </section>

    <!-- 第三步：第一个人物小传 -->
    <section v-else class="step">
      <p class="lead">建第一个人物小传。先从「我自己」开始最省事，两栏填完就能用。</p>

      <label class="field-block">
        <span>姓名 / 昵称</span>
        <input v-model="personaName" class="field" placeholder="例如：小林" />
      </label>

      <label class="field-block">
        <span>身份标签（可留空，之后补）</span>
        <input v-model="personaIdentity" class="field" placeholder="例如：心身同调训练法主理人" />
      </label>

      <div class="key-actions">
        <button class="btn btn-primary" type="button" :disabled="saving" @click="createPersona">
          <AppIcon name="check" :size="15" />
          创建并开始使用
        </button>
        <button class="btn btn-ghost" type="button" @click="finish">先跳过</button>
      </div>
    </section>

    <template #footer>
      <div class="onboard-foot">
        <button v-if="step > 0" class="btn btn-sm btn-ghost" type="button" @click="step -= 1">
          上一步
        </button>
        <span class="muted">这些设置随时能在「设置」和「人物小传」里改</span>
        <button v-if="step === 0" class="btn btn-primary" type="button" @click="step = 1">
          开始设置
          <AppIcon name="arrowRight" :size="15" />
        </button>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.onboard-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.onboard-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: linear-gradient(145deg, var(--accent), var(--accent-strong));
  color: var(--accent-ink);
  font-size: 17px;
  font-weight: 700;
  flex: none;
}

.onboard-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
}

.onboard-head p {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-3);
}

.onboard-head .chip {
  margin-left: auto;
}

.step {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.lead {
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.8;
}

.points {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.points li {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 10px 12px;
  border-radius: 9px;
  background: var(--bg-inset);
  color: var(--text-3);
  font-size: 12.5px;
}

.points li b {
  color: var(--accent);
}

.provider-row {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.provider-chip {
  padding: 6px 13px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
}

.provider-chip.is-active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}

.key-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-block > span {
  color: var(--text-3);
  font-size: 12.5px;
}

.hint {
  color: var(--text-4);
  font-size: 11.5px;
}

.onboard-foot {
  display: flex;
  align-items: center;
  gap: 10px;
}

.onboard-foot .muted {
  flex: 1;
  font-size: 11.5px;
}
</style>
