/**
 * 数据层类型。
 *
 * 约定：
 * - 行数据（Row）的字段名 = 数据库列名，保持 snake_case，不做二次映射；
 * - 查询参数（ListQuery / Filter）用 camelCase，和后端 serde 配置一致。
 */

/* --------------------------------- 表名 --------------------------------- */

export type TableName =
  | 'settings'
  | 'personas'
  | 'materials'
  | 'writings'
  | 'topics'
  | 'bloggers'
  | 'blogger_posts'
  | 'todos'
  | 'notes'
  | 'plans'
  | 'pomodoro'
  | 'memories'
  | 'ai_usage'

export type FilterOp =
  | 'eq'
  | 'ne'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'
  | 'like'
  | 'notLike'
  | 'in'
  | 'isNull'
  | 'notNull'

export interface Filter {
  column: string
  op?: FilterOp
  value?: unknown
}

export interface ListQuery {
  filters?: Filter[]
  orderBy?: string
  desc?: boolean
  limit?: number
  offset?: number
}

/* --------------------------------- 行数据 --------------------------------- */

export interface SettingRow {
  key: string
  value: string
  updated_at: string
}

export type PersonaCategory = 'self_ip' | 'target_customer' | 'case'

export interface Persona {
  id: number
  name: string
  category: PersonaCategory
  /** JSON 字符串 */
  fields: string
  created_at: string
  updated_at: string
}

export type MaterialType = 'image' | 'video' | 'audio' | 'text'

export interface Material {
  id: number
  type: MaterialType
  name: string
  file_path: string | null
  tags: string
  description: string | null
  created_at: string
  /** 所属自定义分类 */
  category_id: number | null
  /** 缩略图路径（图片本地生成，视频由前端抓首帧回传） */
  thumb_path: string | null
  size_bytes: number
  /** 导入时的原始路径 */
  source_path: string | null
  /** 生成过 AI 描述的时间 */
  ai_description_at: string | null
}

export interface MaterialCategory {
  id: number
  name: string
  created_at: string
}

export interface MaterialImportSkip {
  path: string
  reason: string
}

export interface MaterialImportOutcome {
  imported: Material[]
  skipped: MaterialImportSkip[]
}

export interface MaterialLibraryInfo {
  directory: string
  count: number
  totalBytes: number
  categories: number
}

export interface MaterialDescription {
  description: string
  tags: string[]
  provider: string
  providerLabel: string
  model: string
  usedTrialKey: boolean
  remainingFree: number
  /** 是否把图片一并发给了模型 */
  sentImage: boolean
  material: Material
}

export type MaterialPurpose = 'writing' | 'image' | 'video'

export interface MaterialPromptBlock {
  purpose: MaterialPurpose
  purposeLabel: string
  text: string
}

/** 类型元信息，用于筛选条和上传提示 */
export const MATERIAL_TYPES: Array<{
  key: MaterialType
  label: string
  icon: string
  extensions: string
}> = [
  { key: 'image', label: '图片', icon: 'image', extensions: 'jpg / png / webp' },
  { key: 'video', label: '视频', icon: 'video', extensions: 'mp4 / mov' },
  { key: 'audio', label: '音频', icon: 'clock', extensions: 'mp3 / wav' },
  { key: 'text', label: '文本', icon: 'text', extensions: 'txt / md' },
]

/** 素材标签在库里是逗号分隔的字符串 */
export function parseMaterialTags(raw: string | null | undefined): string[] {
  if (!raw) return []
  return raw
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean)
}

/** 字节数转成好读的字符串 */
export function formatBytes(bytes: number): string {
  if (!bytes) return '—'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
}

export type WritingStatus = 'draft' | 'ready' | 'published'

export interface Writing {
  id: number
  title: string
  content: string
  status: WritingStatus
  persona_id: number | null
  topic_id: number | null
  /** JSON 字符串（配图列表） */
  images: string
  created_at: string
  updated_at: string
  /** 文体：xiaohongshu / wechat / script / moments */
  genre: string
  /** 对话式修改的往返记录（JSON 字符串） */
  history: string
  /** 发布日期 */
  published_at: string | null
  /** 最近 5 个版本的内容快照（JSON 字符串） */
  version_history: string
  /** 业务数据：成交人数 / 客资量 / 引流私域人数 / 咨询人数 */
  deal_count: number
  lead_count: number
  private_count: number
  consult_count: number
  /** 业务数据截至 */
  stats_updated_at: string | null
}

/** 版本历史里的一个快照 */
export interface WritingSnapshot {
  title: string
  content: string
  savedAt: string
}

export interface WritingExport {
  filename: string
  content: string
  format: 'txt' | 'md'
}

/* --------------------------------- AI 写作 --------------------------------- */

export type WritingGenre = 'xiaohongshu' | 'wechat' | 'script' | 'moments'

export interface GenreInfo {
  id: WritingGenre
  label: string
}

export interface WriteVersion {
  angle: string
  title: string
  content: string
}

/** 产品规定的配图建议输出格式 */
export interface ImageSuggestion {
  position: string
  keywords: string
  style: string
  ratio: string
}

export interface WriteResult {
  genre: WritingGenre
  genreLabel: string
  versions: WriteVersion[]
  imageKeywords: ImageSuggestion[]
  personaId: number | null
  usedPersona: boolean
  materialCount: number
  provider: string
  providerLabel: string
  model: string
  usedTrialKey: boolean
  remainingFree: number
}

/** 对话式修改里的一条往返 */
export interface ChatTurn {
  role: 'user' | 'assistant'
  content: string
}

export interface ReviseResult {
  content: string
  provider: string
  providerLabel: string
  model: string
  usedTrialKey: boolean
  remainingFree: number
}

export type TopicSource = '课程' | '评论区' | '热点' | '对标' | '已发布'
export type TopicStatus = 'pending' | 'writing' | 'published'

export interface Topic {
  id: number
  title: string
  source: TopicSource
  status: TopicStatus
  tags: string
  note: string | null
  created_at: string
  updated_at: string
}

export type BloggerPlatform = '小红书' | '抖音' | '视频号'

export interface Blogger {
  id: number
  name: string
  platform: BloggerPlatform
  homepage_url: string | null
  avatar: string | null
  tags: string
  created_at: string
}

export type BloggerPostStatus = 'new' | 'added' | 'ignored'

export interface BloggerPost {
  id: number
  blogger_id: number
  title: string
  url: string | null
  cover: string | null
  likes: number
  collects: number
  comments: number
  shares: number
  publish_date: string | null
  /** 0 / 1 */
  is_abnormal: number
  abnormal_reason: string | null
  status: BloggerPostStatus
  created_at: string
}

export type TodoType = 'task' | 'idea' | 'video' | 'article' | 'text'
export type TodoStatus = 'pending' | 'done'

/** 首页「待处理」用的四类（不含 task） */
export type InboxType = Exclude<TodoType, 'task'>

export interface Todo {
  id: number
  title: string
  type: TodoType
  status: TodoStatus
  plan_date: string | null
  done_at: string | null
  created_at: string
}

export interface Note {
  id: number
  content: string
  /** 已转化为：type / topic / todo，或 'todo:12' 这种带目标 id 的形式 */
  converted_to: string | null
  created_at: string
}

export type PlanPeriod = 'month' | 'week'

export interface Plan {
  id: number
  period: PlanPeriod
  goal: string
  /** JSON 字符串（目标拆解） */
  breakdown: string
  created_at: string
  updated_at: string
}

export interface PomodoroRow {
  id: number
  task: string | null
  /** 时长（秒） */
  duration: number
  started_at: string | null
  finished_at: string | null
}

export interface Memory {
  id: number
  source_type: string
  source_id: number | null
  content: string
  /** JSON 字符串（向量），可空 */
  embedding: string | null
  created_at: string
}

export type MemorySource = 'persona' | 'material' | 'writing' | 'topic'

/** 记忆库列表项 */
export interface MemoryRow {
  id: number
  sourceType: MemorySource
  sourceLabel: string
  sourceId: number
  content: string
  createdAt: string
  /** 检索相关度，列表接口为 0 */
  score: number
}

export interface MemorySourceCount {
  sourceType: MemorySource
  label: string
  count: number
}

export interface MemoryStats {
  total: number
  bySource: MemorySourceCount[]
}

export interface MemoryImportResult {
  tables: number
  rows: number
}

export type AiUsageType = 'text' | 'image'

export interface AiUsage {
  id: number
  model: string
  type: AiUsageType
  tokens: number
  created_at: string
  /** 0 / 1 */
  is_free_trial: number
}

export interface AiUsageSummary {
  total: number
  freeTrial: number
  tokens: number
}

/* ------------------------------- 结构自省 ------------------------------- */

export interface ColumnInfo {
  name: string
  dataType: string
  notNull: boolean
  primaryKey: boolean
  defaultValue: string | null
}

export interface TableInfo {
  name: string
  rows: number
  columns: ColumnInfo[]
}

export interface DbStatus {
  ready: boolean
  /** 数据库文件绝对路径 */
  path: string
  /** 所在目录 */
  directory: string
  /** SQLite 版本 */
  version: string
  /** schema 版本（PRAGMA user_version） */
  schemaVersion: number
  /** 已注册的业务表数量 */
  tableCount: number
}

export interface SelfTestResult {
  ok: boolean
  table: string
  written: Note
  readBack: Note
  notesCount: number
}

/* ------------------------------- 界面元信息 ------------------------------- */

/** 首页「待处理」四个分类的展示信息 */
export const INBOX_TYPES: Array<{ key: InboxType; label: string; icon: string }> = [
  { key: 'idea', label: '灵感', icon: 'bulb' },
  { key: 'video', label: '视频资料', icon: 'video' },
  { key: 'article', label: '网页文章', icon: 'link' },
  { key: 'text', label: '文字', icon: 'text' },
]

/** 正在进行中的内容（首页「今日主线」） */
export interface ContentDraft {
  id: number
  title: string
  stage: string
  updatedAt: string
}

/* --------------------------------- AI 相关 --------------------------------- */

export interface ProviderInfo {
  id: string
  label: string
  defaultModel: string
  models: string[]
  consoleUrl: string
  keyPrefix: string
}

export interface KeyStatus {
  provider: string
  label: string
  hasKey: boolean
  /** 打码后的 Key，例如 sk-123••••cdef */
  masked: string
  consoleUrl: string
  keyPrefix: string
}

export interface QuotaInfo {
  limit: number
  used: number
  remaining: number
  /** 内置体验 Key 是否已配置 */
  configured: boolean
  provider: string
  providerLabel: string
  model: string
}

export interface AiSettings {
  selectedProvider: string
  selectedModel: string
  nickname: string
  keySourceId: 'keyring' | 'keyFile'
  keySourceLabel: string
  keys: KeyStatus[]
  quota: QuotaInfo
}

export interface AiChatResponse {
  provider: string
  providerLabel: string
  model: string
  content: string
  tokens: number
  usedTrialKey: boolean
  remainingFree: number
  latencyMs: number
}

export interface AiTestResult {
  ok: boolean
  provider: string
  providerLabel: string
  model: string
  latencyMs: number
  reply: string
}

/** 后端 AiError 序列化后的形状 */
export interface AiErrorPayload {
  code: string
  message: string
  hint: string
}

/* -------------------------------- 人物小传 -------------------------------- */

/** 人物档案明细，存在 personas.fields 这一列的 JSON 里 */
export interface PersonaFields {
  age: string
  /** 身份标签 */
  identity: string
  /** 核心经历 */
  experience: string
  /** 性格特点关键词（3-5 个） */
  traits: string[]
  /** 口头禅 */
  catchphrases: string
  /** 常用表达 */
  expressions: string
  /** 目标受众是谁 */
  audience: string
  /** 受众痛点 */
  painPoints: string
  /** 禁忌话题 */
  taboos: string
  /** 代表观点 */
  viewpoints: string
  /** 视觉风格：色调 */
  visualTone: string
  /** 视觉风格：风格 */
  visualStyle: string
  /** 视觉风格：场景 */
  visualScene: string
  /** 档案来源说明 */
  sourceNote: string
}

export interface PersonaCategoryInfo {
  id: PersonaCategory
  label: string
}

/** AI 提取特征的返回 */
export interface PersonaExtraction {
  fields: PersonaFields
  provider: string
  providerLabel: string
  model: string
  usedTrialKey: boolean
  remainingFree: number
}

export type PersonaPurpose = 'writing' | 'interview' | 'image'

/** 给别的模块用的人物提示词块 */
export interface PersonaPromptBlock {
  purpose: PersonaPurpose
  purposeLabel: string
  text: string
}

/** 空档案，表单初始化用 */
export function emptyPersonaFields(): PersonaFields {
  return {
    age: '',
    identity: '',
    experience: '',
    traits: [],
    catchphrases: '',
    expressions: '',
    audience: '',
    painPoints: '',
    taboos: '',
    viewpoints: '',
    visualTone: '',
    visualStyle: '',
    visualScene: '',
    sourceNote: '',
  }
}

/** 把数据库里的 JSON 文本解析成档案；坏数据不会让页面崩掉 */
export function parsePersonaFields(raw: string | null | undefined): PersonaFields {
  const base = emptyPersonaFields()
  if (!raw) return base

  try {
    const parsed = JSON.parse(raw) as Partial<PersonaFields>
    return {
      ...base,
      ...parsed,
      traits: Array.isArray(parsed.traits) ? parsed.traits.filter(Boolean) : [],
    }
  } catch {
    return base
  }
}
