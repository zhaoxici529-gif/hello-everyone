<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import AppIcon from '../components/AppIcon.vue'
import WorkspaceCard from '../components/WorkspaceCard.vue'
import AiKeyGuide from '../components/AiKeyGuide.vue'
import { useAppStore } from '../stores/app'
import { useAiStore } from '../stores/ai'
import { dbDescribe, dbSelfTest } from '../api'
import { CONTENT_SYSTEM_PROMPT, TRIAL_PROMPT } from '../config/prompts'
import type { KeyStatus, SelfTestResult, TableInfo } from '../types'

const app = useAppStore()
const ai = useAiStore()
const message = useMessage()

/* --------------------------------- AI 设置 --------------------------------- */

const keyDraft = ref('')
const nicknameDraft = ref('')

const currentProvider = computed(
  () => ai.providers.find((item) => item.id === ai.selectedProvider) ?? null,
)

function keyOf(provider: string): KeyStatus | undefined {
  return ai.keys.find((item) => item.provider === provider)
}

async function selectProvider(id: string): Promise<void> {
  keyDraft.value = ''
  await ai.selectProvider(id)
}

async function changeModel(event: Event): Promise<void> {
  await ai.selectModel((event.target as HTMLSelectElement).value)
}

async function saveKey(): Promise<void> {
  if (!keyDraft.value.trim()) {
    message.warning('先把 API Key 粘贴到输入框里')
    return
  }

  const ok = await ai.saveKey(ai.selectedProvider, keyDraft.value)
  if (ok) {
    keyDraft.value = ''
    message.success('已加密保存到本地数据库')
  } else {
    message.error(ai.lastError?.message ?? '保存失败')
  }
}

async function removeKey(): Promise<void> {
  const ok = await ai.removeKey(ai.selectedProvider)
  if (ok) message.success('已删除本地保存的 Key')
}

async function testConnection(): Promise<void> {
  const result = await ai.testConnection(
    ai.selectedProvider,
    keyDraft.value.trim() || undefined,
  )

  if (result) {
    message.success(
      `连接成功：${result.providerLabel} · ${result.model}（${result.latencyMs}ms）`,
    )
  } else {
    message.error(ai.lastError?.message ?? '连接失败')
  }
}

async function saveNickname(): Promise<void> {
  const ok = await ai.persistNickname(nicknameDraft.value)
  if (ok) message.success('昵称已保存')
}

/** 试用一次：走体验额度，用来验证额度与引导流程 */
async function tryOnce(): Promise<void> {
  const response = await ai.run(TRIAL_PROMPT, { system: CONTENT_SYSTEM_PROMPT })

  if (response) {
    message.success(
      response.usedTrialKey
        ? `体验额度调用成功，还剩 ${response.remainingFree} 次`
        : '调用成功（使用你自己的 Key）',
    )
    return
  }

  if (ai.lastError && ai.lastError.code !== 'quota_exhausted') {
    message.error(ai.lastError.message)
  }
}

/* -------------------------------- 数据库自检 -------------------------------- */

const tables = ref<TableInfo[]>([])
const loadingTables = ref(false)
const runningSelfTest = ref(false)
const selfTest = ref<SelfTestResult | null>(null)
const dbError = ref('')

const totalRows = computed(() => tables.value.reduce((sum, table) => sum + table.rows, 0))
const unusedCount = computed(() =>
  tables.value.filter((table) => table.rows === 0).length,
)

async function loadTables(): Promise<void> {
  if (!app.isDatabaseReady) return

  loadingTables.value = true
  const result = await dbDescribe()
  if (result) {
    tables.value = result
    dbError.value = ''
  } else {
    dbError.value = '读取表结构失败'
  }
  loadingTables.value = false
}

async function runSelfTest(): Promise<void> {
  runningSelfTest.value = true
  const result = await dbSelfTest()

  if (result) {
    selfTest.value = result
    dbError.value = ''
    await loadTables()
  } else {
    dbError.value = '自检失败：当前不在 Tauri 运行时里'
  }
  runningSelfTest.value = false
}

onMounted(async () => {
  if (!app.isDatabaseReady) return

  await ai.load()
  nicknameDraft.value = ai.nickname
  await loadTables()
})
</script>

<template>
  <div class="settings-page">
    <section class="hero">
      <span class="hero-icon"><AppIcon name="gear" :size="26" /></span>
      <div>
        <span class="hero-badge">设置</span>
        <h1>设置</h1>
        <p>选模型、填 Key、看额度，都在这一页。</p>
      </div>
      <span class="hero-note">v{{ app.version }}</span>
    </section>

    <p v-if="!app.isDatabaseReady" class="hint is-error">
      当前是浏览器预览模式，没有 Rust 运行时。AI 配置需要双击「自媒体AI工作台.exe」或执行 pnpm tauri dev。
    </p>

    <!-- ------------------------------ AI 模型与密钥 ------------------------------ -->
    <WorkspaceCard title="AI 模型与密钥" subtitle="选一家模型，填入自己的 API Key" icon="sparkles">
      <template #actions>
        <span class="chip">剩余体验 {{ ai.remaining }} 次</span>
        <button class="btn btn-sm" type="button" :disabled="ai.loading" @click="ai.load()">
          <AppIcon name="reset" :size="14" />
          刷新
        </button>
      </template>

      <div class="provider-grid">
        <button
          v-for="item in ai.providers"
          :key="item.id"
          class="provider-card"
          :class="{ 'is-active': ai.selectedProvider === item.id }"
          type="button"
          @click="selectProvider(item.id)"
        >
          <span class="provider-name">{{ item.label }}</span>
          <span class="provider-state" :class="{ 'is-ok': keyOf(item.id)?.hasKey }">
            <i class="state-dot" />
            {{ keyOf(item.id)?.hasKey ? '已配置' : '未配置' }}
          </span>
        </button>
      </div>

      <div class="key-panel">
        <div class="key-row">
          <label class="key-label">使用模型</label>
          <select
            class="field key-model"
            :value="ai.selectedModel"
            :disabled="!currentProvider"
            @change="changeModel"
          >
            <option v-for="model in currentProvider?.models ?? []" :key="model" :value="model">
              {{ model }}
            </option>
          </select>
          <span class="key-hint">
            {{ currentProvider?.label }} 的 Key 通常以
            <code>{{ currentProvider?.keyPrefix }}</code> 开头
          </span>
        </div>

        <div class="key-row">
          <label class="key-label">API Key</label>
          <input
            v-model="keyDraft"
            class="field key-input"
            type="password"
            autocomplete="off"
            :placeholder="keyOf(ai.selectedProvider)?.hasKey ? '已保存，粘贴新的会覆盖' : '把复制的 Key 粘贴到这里'"
          />
          <button class="btn btn-primary btn-sm" type="button" @click="saveKey">
            <AppIcon name="check" :size="14" />
            保存
          </button>
          <button class="btn btn-sm" type="button" :disabled="ai.testing" @click="testConnection">
            <AppIcon name="play" :size="14" />
            {{ ai.testing ? '测试中…' : '测试连接' }}
          </button>
          <button
            v-if="keyOf(ai.selectedProvider)?.hasKey"
            class="btn btn-sm btn-ghost"
            type="button"
            @click="removeKey"
          >
            <AppIcon name="trash" :size="14" />
            清除
          </button>
        </div>

        <p class="key-saved">
          <template v-if="keyOf(ai.selectedProvider)?.hasKey">
            <AppIcon name="check" :size="14" />
            已保存：<code>{{ keyOf(ai.selectedProvider)?.masked }}</code>
          </template>
          <template v-else>
            <AppIcon name="inbox" :size="14" />
            还没填 Key，AI 功能会先用内置的免费体验额度（共 {{ ai.quotaLimit }} 次）。
          </template>
        </p>

        <p class="key-note">
          <AppIcon name="layers" :size="14" />
          密钥用 AES-256-GCM 加密后存进本地数据库，主密钥放在{{ ai.settings?.keySourceLabel ?? '系统凭据管理器' }}，不会上传。
        </p>
      </div>

      <!-- 测试结果 / 错误提示 -->
      <div v-if="ai.testResult" class="result is-ok">
        <AppIcon name="check" :size="15" />
        <div>
          <strong>连接成功</strong>
          <p>
            {{ ai.testResult.providerLabel }} · {{ ai.testResult.model }} ·
            {{ ai.testResult.latencyMs }}ms，模型回复：{{ ai.testResult.reply }}
          </p>
        </div>
      </div>

      <div v-else-if="ai.lastError" class="result is-error">
        <AppIcon name="close" :size="15" />
        <div>
          <strong>{{ ai.lastError.message }}</strong>
          <p v-if="ai.lastError.hint">{{ ai.lastError.hint }}</p>
          <p v-if="ai.lastError.code === 'quota_exhausted'">
            <button class="btn btn-sm btn-primary" type="button" @click="ai.showQuotaGuide = true">
              查看图文教程
            </button>
          </p>
        </div>
      </div>

      <details class="guide-toggle">
        <summary>
          <AppIcon name="bulb" :size="15" />
          怎么注册、怎么拿 Key、怎么填？（图文教程）
        </summary>
        <div class="guide-body">
          <AiKeyGuide :provider="currentProvider" />
        </div>
      </details>
    </WorkspaceCard>

    <!-- --------------------------- 体验额度 / 昵称 --------------------------- -->
    <div class="settings-grid">
      <WorkspaceCard title="免费体验额度" subtitle="不用填 Key 也能先试 3 次" icon="sparkles">
        <div class="quota">
          <div class="quota-number">
            <span class="num quota-remaining">{{ ai.remaining }}</span>
            <span class="quota-total">/ {{ ai.quotaLimit }} 次剩余</span>
          </div>
          <div class="progress-track">
            <div class="progress-fill" :style="{ width: ai.quotaPercent + '%' }" />
          </div>
          <p class="quota-meta">
            已用 {{ ai.quotaUsed }} 次 · 体验模型
            {{ ai.quota?.providerLabel || '—' }} · {{ ai.quota?.model || '—' }}
          </p>
          <p v-if="ai.quota && !ai.quota.configured" class="quota-warn">
            体验 Key 还没配置：请在 src-tauri/config/trial.json 里填入真实 Key。
          </p>
        </div>

        <div class="quota-actions">
          <button class="btn btn-primary" type="button" :disabled="ai.busy" @click="tryOnce">
            <AppIcon name="play" :size="15" />
            {{ ai.busy ? '调用中…' : '试用一次' }}
          </button>
          <span class="muted">每次调用都会记进 ai_usage 表</span>
        </div>

        <div v-if="ai.lastReply" class="reply">
          <div class="reply-head">
            <span class="chip">
              {{ ai.lastReply.usedTrialKey ? '体验额度' : '我的 Key' }}
            </span>
            <span class="muted">
              {{ ai.lastReply.providerLabel }} · {{ ai.lastReply.model }} ·
              {{ ai.lastReply.tokens }} tokens · {{ ai.lastReply.latencyMs }}ms
            </span>
            <span class="chip">剩余 {{ ai.lastReply.remainingFree }} 次</span>
          </div>
          <pre class="reply-body">{{ ai.lastReply.content }}</pre>
        </div>
      </WorkspaceCard>

      <WorkspaceCard title="学员昵称" subtitle="首页问候语会用到" icon="users">
        <div class="nickname-row">
          <input
            v-model="nicknameDraft"
            class="field"
            placeholder="怎么称呼你？例如：小林"
            maxlength="20"
          />
          <button class="btn btn-primary" type="button" @click="saveNickname">保存</button>
        </div>
        <p class="nickname-preview">
          预览：{{ nicknameDraft ? `上午好，${nicknameDraft}` : '上午好，欢迎回到工作台' }}
        </p>
        <p class="key-note">
          <AppIcon name="layers" :size="14" />
          昵称存在 settings 表里，重启后依然保留。
        </p>
      </WorkspaceCard>
    </div>

    <!-- ------------------------------ 最近调用记录 ------------------------------ -->
    <WorkspaceCard title="最近 AI 调用" subtitle="来自 ai_usage 表" icon="clock">
      <template #actions>
        <button class="btn btn-sm" type="button" @click="ai.refreshQuotaAndUsage()">
          <AppIcon name="reset" :size="14" />
          刷新
        </button>
      </template>

      <ul v-if="ai.usage.length" class="usage-list">
        <li v-for="row in ai.usage" :key="row.id">
          <span class="usage-model num">{{ row.model }}</span>
          <span class="usage-tokens num">{{ row.tokens }} tokens</span>
          <span class="chip" :class="{ 'is-trial': row.is_free_trial === 1 }">
            {{ row.is_free_trial === 1 ? '体验额度' : '自己的 Key' }}
          </span>
          <span class="usage-time num">{{ row.created_at }}</span>
        </li>
      </ul>
      <p v-else class="muted usage-empty">还没有调用记录。点上面的「试用一次」试试。</p>
    </WorkspaceCard>

    <!-- -------------------------------- 数据库 -------------------------------- -->
    <div class="settings-grid">
      <WorkspaceCard title="运行状态" subtitle="当前工作台的本地环境" icon="layers">
        <dl class="status-list">
          <div>
            <dt>数据库状态</dt>
            <dd :class="{ 'is-ok': app.isDatabaseReady }">{{ app.databaseText }}</dd>
          </div>
          <div>
            <dt>数据库文件</dt>
            <dd class="mono">{{ app.databasePath || '—' }}</dd>
          </div>
          <div>
            <dt>SQLite 版本</dt>
            <dd class="mono">{{ app.databaseVersion || '—' }}</dd>
          </div>
          <div>
            <dt>Schema 版本</dt>
            <dd class="mono">{{ app.schemaVersion || '—' }}</dd>
          </div>
          <div>
            <dt>业务表</dt>
            <dd class="mono">{{ tables.length }} / {{ app.tableCount || 13 }} 张 · 共 {{ totalRows }} 行</dd>
          </div>
        </dl>
      </WorkspaceCard>

      <WorkspaceCard title="待配置项" subtitle="接下来要接的东西" icon="target">
        <ul class="row-list">
          <li>
            <span class="row-icon"><AppIcon name="image" :size="15" /></span>
            <span class="row-label">素材目录</span>
            <span class="row-value">待配置</span>
          </li>
          <li>
            <span class="row-icon"><AppIcon name="users" :size="15" /></span>
            <span class="row-label">对标博主名单</span>
            <span class="row-value">存 bloggers 表</span>
          </li>
          <li>
            <span class="row-icon"><AppIcon name="target" :size="15" /></span>
            <span class="row-label">品牌记忆与口吻</span>
            <span class="row-value">存 personas / memories</span>
          </li>
          <li>
            <span class="row-icon"><AppIcon name="sparkles" :size="15" /></span>
            <span class="row-label">图片生成</span>
            <span class="row-value">接口已预留</span>
          </li>
        </ul>
      </WorkspaceCard>

      <WorkspaceCard title="快捷入口" subtitle="常用模块，点一下就过去" icon="compass">
        <ul class="shortcut-list">
          <li>
            <RouterLink to="/personas" class="shortcut">
              <span class="shortcut-icon"><AppIcon name="users" :size="16" /></span>
              <span class="shortcut-text">
                <strong>人物小传</strong>
                <small>写文案前先把语气和立场固定下来</small>
              </span>
              <AppIcon name="arrowRight" :size="15" />
            </RouterLink>
          </li>
          <li>
            <RouterLink to="/create" class="shortcut">
              <span class="shortcut-icon"><AppIcon name="pen" :size="16" /></span>
              <span class="shortcut-text">
                <strong>创作</strong>
                <small>常用助手现在会带上人物档案</small>
              </span>
              <AppIcon name="arrowRight" :size="15" />
            </RouterLink>
          </li>
          <li>
            <RouterLink to="/materials" class="shortcut">
              <span class="shortcut-icon"><AppIcon name="image" :size="16" /></span>
              <span class="shortcut-text">
                <strong>素材库</strong>
                <small>图片、视频、音频、文本都在这里</small>
              </span>
              <AppIcon name="arrowRight" :size="15" />
            </RouterLink>
          </li>
          <li>
            <RouterLink to="/plan" class="shortcut">
              <span class="shortcut-icon"><AppIcon name="calendar" :size="16" /></span>
              <span class="shortcut-text">
                <strong>计划</strong>
                <small>今天、本周、本月的排期</small>
              </span>
              <AppIcon name="arrowRight" :size="15" />
            </RouterLink>
          </li>
        </ul>
      </WorkspaceCard>
    </div>

    <WorkspaceCard title="数据库表结构" subtitle="建表和读写自检" icon="layers">
      <template #actions>
        <span class="chip">{{ tables.length }} 张表 · {{ unusedCount }} 张还没有数据</span>
        <button class="btn btn-sm" type="button" :disabled="loadingTables" @click="loadTables">
          <AppIcon name="reset" :size="14" />
          刷新
        </button>
        <button
          class="btn btn-primary btn-sm"
          type="button"
          :disabled="runningSelfTest"
          @click="runSelfTest"
        >
          <AppIcon name="play" :size="14" />
          运行读写自检
        </button>
      </template>

      <p v-if="dbError" class="hint is-error">{{ dbError }}</p>

      <div v-else class="table-grid">
        <div
          v-for="table in tables"
          :key="table.name"
          class="table-tile"
          :title="table.columns.map((column) => column.name).join(', ')"
        >
          <span class="table-name num">{{ table.name }}</span>
          <span class="table-meta">{{ table.columns.length }} 列 · {{ table.rows }} 行</span>
        </div>
      </div>

      <div v-if="selfTest" class="result" :class="selfTest.ok ? 'is-ok' : 'is-error'">
        <AppIcon :name="selfTest.ok ? 'check' : 'close'" :size="15" />
        <div>
          <strong>读写自检{{ selfTest.ok ? '通过' : '失败' }}</strong>
          <p>写入 notes #{{ selfTest.written.id }}，读回内容一致；当前 notes 共 {{ selfTest.notesCount }} 行。</p>
        </div>
      </div>
    </WorkspaceCard>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  max-width: 1440px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: linear-gradient(120deg, rgba(184, 212, 168, 0.1), rgba(27, 32, 27, 0.9) 55%);
  box-shadow: var(--shadow-card);
}

.hero-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 54px;
  height: 54px;
  border-radius: 14px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  flex: none;
}

.hero-badge {
  display: inline-block;
  margin-bottom: 6px;
  padding: 2px 9px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11.5px;
}

.hero h1 {
  font-size: 21px;
  font-weight: 600;
}

.hero p {
  margin-top: 3px;
  color: var(--text-3);
  font-size: 13px;
}

.hero-note {
  margin-left: auto;
  color: var(--text-4);
  font-size: 12px;
}

/* ------------------------------- 模型与密钥 ------------------------------- */

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 9px;
}

.provider-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.provider-card:hover {
  border-color: var(--border-strong);
}

.provider-card.is-active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.provider-name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-1);
}

.provider-state {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11.5px;
  color: var(--text-4);
}

.provider-state.is-ok {
  color: var(--accent);
}

.state-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.key-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 16px;
  padding: 15px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.key-row {
  display: flex;
  align-items: center;
  gap: 9px;
  flex-wrap: wrap;
}

.key-label {
  flex: none;
  width: 62px;
  color: var(--text-3);
  font-size: 12.5px;
}

.key-model {
  flex: none;
  width: 200px;
}

.key-input {
  flex: 1;
  min-width: 220px;
  font-family: var(--font-mono);
  font-size: 12.5px;
}

.key-hint {
  color: var(--text-4);
  font-size: 11.5px;
}

.key-hint code,
.key-saved code {
  padding: 1px 6px;
  border-radius: 5px;
  background: var(--accent-soft);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.key-saved,
.key-note {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-3);
  font-size: 12px;
}

.key-note {
  color: var(--text-4);
}

.guide-toggle {
  margin-top: 14px;
  border: 1px dashed var(--border-strong);
  border-radius: 10px;
  padding: 12px 14px;
}

.guide-toggle summary {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent);
  font-size: 13px;
  cursor: pointer;
}

.guide-body {
  margin-top: 14px;
}

/* --------------------------------- 结果提示 --------------------------------- */

.result {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-top: 12px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
  font-size: 12.5px;
}

.result.is-ok {
  border-color: var(--accent-border);
  background: var(--accent-softer);
  color: var(--accent);
}

.result.is-error {
  border-color: rgba(215, 154, 146, 0.4);
  background: rgba(215, 154, 146, 0.08);
  color: var(--danger);
}

.result strong {
  display: block;
  font-size: 13px;
}

.result p {
  margin-top: 4px;
  color: var(--text-2);
  line-height: 1.7;
}

/* --------------------------------- 体验额度 --------------------------------- */

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(380px, 1fr));
  gap: var(--gap);
}

.quota {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quota-number {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.quota-remaining {
  font-size: 38px;
  font-weight: 600;
  line-height: 1;
  color: var(--accent);
}

.quota-total {
  color: var(--text-3);
  font-size: 13px;
}

.progress-track {
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  background: linear-gradient(90deg, var(--accent), var(--accent-strong));
  transition: width 0.28s ease;
}

.quota-meta {
  color: var(--text-4);
  font-size: 12px;
}

.quota-warn {
  padding: 9px 11px;
  border-radius: 8px;
  background: rgba(216, 192, 138, 0.1);
  color: var(--warn);
  font-size: 12px;
}

.quota-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
}

.quota-actions .muted {
  font-size: 11.5px;
}

.reply {
  margin-top: 14px;
  border: 1px solid var(--accent-border);
  border-radius: 10px;
  background: var(--accent-softer);
  overflow: hidden;
}

.reply-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
  font-size: 11.5px;
}

.reply-head .chip:last-child {
  margin-left: auto;
}

.reply-body {
  margin: 0;
  padding: 12px;
  max-height: 180px;
  overflow-y: auto;
  color: var(--text-1);
  font-family: var(--font-ui);
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
}

/* ---------------------------------- 昵称 ---------------------------------- */

.nickname-row {
  display: flex;
  gap: 9px;
}

.nickname-preview {
  margin-top: 12px;
  color: var(--text-2);
  font-size: 13px;
}

/* -------------------------------- 调用记录 -------------------------------- */

.usage-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.usage-list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
  font-size: 12px;
}

.usage-model {
  color: var(--accent);
  min-width: 140px;
}

.usage-tokens {
  color: var(--text-3);
  min-width: 84px;
}

.usage-time {
  margin-left: auto;
  color: var(--text-4);
}

.usage-empty {
  font-size: 12.5px;
}

/* --------------------------------- 数据库 --------------------------------- */

.status-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
}

.status-list > div {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  padding-bottom: 10px;
  border-bottom: 1px dashed var(--divider);
}

.status-list > div:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.status-list dt {
  color: var(--text-3);
  font-size: 12.5px;
  flex: none;
}

.status-list dd {
  margin: 0;
  color: var(--text-1);
  font-size: 12.5px;
  text-align: right;
  word-break: break-all;
}

.status-list dd.is-ok {
  color: var(--accent);
}

.mono {
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.row-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.row-list li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
}

.row-icon {
  color: var(--accent);
}

.row-label {
  flex: 1;
  font-size: 12.5px;
  color: var(--text-2);
}

.row-value {
  font-size: 12px;
  color: var(--text-4);
}

.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.shortcut {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
  color: var(--text-2);
  text-decoration: none;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.shortcut:hover {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.shortcut-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
  flex: none;
}

.shortcut-text {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.shortcut-text strong {
  font-size: 13px;
  color: var(--text-1);
}

.shortcut-text small {
  color: var(--text-4);
  font-size: 11.5px;
}

.table-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 9px;
}

.table-tile {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
}

.table-name {
  font-size: 12.5px;
  color: var(--accent);
}

.table-meta {
  font-size: 11.5px;
  color: var(--text-4);
}

.hint {
  margin-top: 12px;
  color: var(--text-4);
  font-size: 12px;
}

.hint.is-error {
  color: var(--danger);
}
</style>
