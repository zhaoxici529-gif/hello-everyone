// 打包前把体验 Key 写进 src-tauri/config/trial.json。
// 优先用环境变量 TRIAL_API_KEY（CI 里从 Secret 注入），没有就保持仓库里的占位值。
// 路径基于脚本自身位置解析，所以在任何目录下执行都没问题。
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), '..')
const configPath = join(projectRoot, 'src-tauri/config/trial.json')

const key = (process.env.TRIAL_API_KEY || '').trim()
const config = JSON.parse(readFileSync(configPath, 'utf8'))

if (key) {
  config.apiKey = key
  config.enabled = true
  console.log(`已注入体验 Key（长度 ${key.length}），前 ${config.freeQuota} 次免费`)
} else {
  console.log('未提供 TRIAL_API_KEY，保持文件里的占位值（应用会显示「体验 Key 未配置」）')
}

writeFileSync(configPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8')