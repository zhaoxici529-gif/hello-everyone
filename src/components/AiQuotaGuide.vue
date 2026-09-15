<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NModal } from 'naive-ui'
import AppIcon from './AppIcon.vue'
import AiKeyGuide from './AiKeyGuide.vue'
import { useAiStore } from '../stores/ai'

/**
 * 体验额度用完时的引导弹窗。
 * 由 ai store 的 showQuotaGuide 控制，任何 AI 调用触发 quota_exhausted 都会弹出来。
 */
const ai = useAiStore()
const router = useRouter()

const provider = computed(
  () => ai.providers.find((item) => item.id === ai.selectedProvider) ?? null,
)

function goToSettings(): void {
  ai.showQuotaGuide = false
  void router.push('/settings')
}
</script>

<template>
  <NModal
    :show="ai.showQuotaGuide"
    preset="card"
    class="quota-modal"
    :style="{ width: 'min(720px, 92vw)' }"
    :bordered="false"
    :mask-closable="true"
    @update:show="(value: boolean) => (ai.showQuotaGuide = value)"
  >
    <template #header>
      <div class="quota-head">
        <span class="quota-icon"><AppIcon name="sparkles" :size="18" /></span>
        <div>
          <h2>免费体验额度已用完</h2>
          <p>前 {{ ai.quotaLimit }} 次已经用完了，填上自己的 Key 就能继续，不再限次数。</p>
        </div>
      </div>
    </template>

    <AiKeyGuide :provider="provider" compact />

    <template #footer>
      <div class="quota-foot">
        <span class="muted">体验额度不消耗你自己的账户余额，用完为止。</span>
        <div class="quota-actions">
          <button class="btn" type="button" @click="ai.showQuotaGuide = false">以后再说</button>
          <button class="btn btn-primary" type="button" @click="goToSettings">
            <AppIcon name="arrowRight" :size="15" />
            去填写我的 Key
          </button>
        </div>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.quota-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.quota-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  flex: none;
}

.quota-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-1);
}

.quota-head p {
  margin-top: 2px;
  font-size: 12.5px;
  color: var(--text-3);
}

.quota-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.quota-foot .muted {
  font-size: 12px;
}

.quota-actions {
  display: flex;
  gap: 8px;
}
</style>
