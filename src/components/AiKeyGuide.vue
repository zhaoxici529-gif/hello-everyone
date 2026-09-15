<script setup lang="ts">
import { computed } from 'vue'
import type { ProviderInfo } from '../types'
import AppIcon from './AppIcon.vue'

/**
 * 「怎么拿到自己的 API Key」图文教程。
 * 给不懂技术的学员看，所以每一步都配一张示意图 + 一句大白话。
 */
const props = withDefaults(
  defineProps<{
    provider?: ProviderInfo | null
    /** 紧凑模式：用在弹窗里 */
    compact?: boolean
  }>(),
  { provider: null, compact: false },
)

const consoleUrl = computed(() => props.provider?.consoleUrl ?? 'https://platform.deepseek.com/api_keys')
const providerLabel = computed(() => props.provider?.label ?? 'DeepSeek')
const keyPrefix = computed(() => props.provider?.keyPrefix ?? 'sk-')
</script>

<template>
  <div class="guide" :class="{ 'is-compact': compact }">
    <p class="guide-intro">
      用自己的 Key 之后就不限次数了。全程大概 3 分钟，跟着下面三步做就行。
    </p>

    <ol class="steps">
      <li class="step">
        <div class="step-art">
          <!-- 示意图 1：打开官网注册 -->
          <svg viewBox="0 0 132 84" aria-hidden="true">
            <rect x="6" y="8" width="120" height="68" rx="8" fill="#1b201b" stroke="#3a4438" />
            <path d="M6 24h120" stroke="#3a4438" />
            <circle cx="16" cy="16" r="2.6" fill="#8fbc8f" />
            <circle cx="25" cy="16" r="2.6" fill="#3a4438" />
            <circle cx="34" cy="16" r="2.6" fill="#3a4438" />
            <rect x="18" y="36" width="56" height="7" rx="3.5" fill="#2c3529" />
            <rect x="18" y="50" width="40" height="7" rx="3.5" fill="#2c3529" />
            <rect x="18" y="64" width="46" height="6" rx="3" fill="#263024" />
            <rect x="84" y="58" width="34" height="14" rx="7" fill="#b8d4a8" />
            <rect x="92" y="63" width="18" height="4" rx="2" fill="#16200f" />
          </svg>
        </div>
        <div class="step-text">
          <span class="step-index num">01</span>
          <h4>打开官网，注册并登录</h4>
          <p>
            点下面的按钮打开 {{ providerLabel }} 的官网，用手机号或微信注册登录。
            登录后一般会有免费赠送额度，够你试很久。
          </p>
          <a class="btn btn-sm step-link" :href="consoleUrl" target="_blank" rel="noreferrer">
            <AppIcon name="link" :size="14" />
            打开 {{ providerLabel }} 官网
          </a>
        </div>
      </li>

      <li class="step">
        <div class="step-art">
          <!-- 示意图 2：找到 API Keys 并复制 -->
          <svg viewBox="0 0 132 84" aria-hidden="true">
            <rect x="6" y="8" width="120" height="68" rx="8" fill="#1b201b" stroke="#3a4438" />
            <rect x="16" y="18" width="100" height="14" rx="5" fill="#26301f" stroke="#4b6a3f" />
            <rect x="22" y="23" width="44" height="5" rx="2.5" fill="#8fbc8f" />
            <rect x="16" y="38" width="100" height="12" rx="5" fill="#232823" />
            <rect x="22" y="42" width="34" height="4" rx="2" fill="#3a4438" />
            <rect x="16" y="56" width="100" height="12" rx="5" fill="#232823" />
            <rect x="22" y="60" width="40" height="4" rx="2" fill="#3a4438" />
            <rect x="98" y="20" width="12" height="10" rx="3" fill="none" stroke="#b8d4a8" stroke-width="1.6" />
            <path d="M101 17h12v10" stroke="#b8d4a8" stroke-width="1.6" fill="none" stroke-linecap="round" />
          </svg>
        </div>
        <div class="step-text">
          <span class="step-index num">02</span>
          <h4>找到「API Keys」，复制那串字符</h4>
          <p>
            在左侧菜单找 <b>API Keys</b>（有的叫「密钥管理」），点
            <b>新建 / 创建</b>，然后点复制。它通常以
            <code>{{ keyPrefix }}</code> 开头，是一长串字符。
          </p>
          <p class="step-warn">注意：这串字符只在创建时显示一次，关掉就看不到了。</p>
        </div>
      </li>

      <li class="step">
        <div class="step-art">
          <!-- 示意图 3：粘贴回工作台并测试 -->
          <svg viewBox="0 0 132 84" aria-hidden="true">
            <rect x="6" y="8" width="120" height="68" rx="8" fill="#1b201b" stroke="#3a4438" />
            <rect x="16" y="20" width="100" height="16" rx="6" fill="#121512" stroke="#3a4438" />
            <circle cx="27" cy="28" r="2.4" fill="#8fbc8f" />
            <circle cx="35" cy="28" r="2.4" fill="#8fbc8f" />
            <circle cx="43" cy="28" r="2.4" fill="#8fbc8f" />
            <rect x="52" y="26" width="30" height="4" rx="2" fill="#3a4438" />
            <rect x="16" y="46" width="46" height="16" rx="8" fill="#b8d4a8" />
            <rect x="26" y="52" width="26" height="4" rx="2" fill="#16200f" />
            <rect x="68" y="46" width="34" height="16" rx="8" fill="#232823" stroke="#3a4438" />
            <path d="M82 54.5l3.4 3.4 6-6.6" stroke="#8fbc8f" stroke-width="1.8" fill="none" stroke-linecap="round" />
          </svg>
        </div>
        <div class="step-text">
          <span class="step-index num">03</span>
          <h4>粘贴回这里，点「测试连接」</h4>
          <p>
            回到本页的模型卡片，把刚才复制的内容粘贴到输入框，点
            <b>测试连接</b>。看到「连接成功」就说明可以用自己的额度开始创作了。
          </p>
        </div>
      </li>
    </ol>

    <p class="guide-foot">
      <AppIcon name="layers" :size="14" />
      Key 只保存在你自己的电脑上，用 AES-256-GCM 加密后存进本地数据库，不会上传到任何服务器。
    </p>
  </div>
</template>

<style scoped>
.guide {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.guide-intro {
  color: var(--text-2);
  font-size: 13px;
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.step {
  display: flex;
  gap: 14px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-inset);
}

.step-art {
  flex: none;
  width: 132px;
}

.step-art svg {
  width: 100%;
  height: auto;
  border-radius: 8px;
}

.step-text {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.step-index {
  color: var(--accent);
  font-size: 12px;
  letter-spacing: 1px;
}

.step-text h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.step-text p {
  color: var(--text-3);
  font-size: 12.5px;
  line-height: 1.7;
}

.step-text b {
  color: var(--text-1);
  font-weight: 600;
}

.step-text code {
  padding: 1px 6px;
  border-radius: 5px;
  background: var(--accent-soft);
  color: var(--accent);
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.step-warn {
  color: var(--warn) !important;
}

.step-link {
  align-self: flex-start;
  margin-top: 2px;
}

.guide-foot {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 10px 12px;
  border-radius: 9px;
  background: var(--accent-softer);
  color: var(--accent);
  font-size: 12px;
}

.is-compact .step-art {
  width: 104px;
}

@media (max-width: 720px) {
  .step {
    flex-direction: column;
  }

  .step-art {
    width: 100%;
    max-width: 200px;
  }
}
</style>
