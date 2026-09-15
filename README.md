# 自媒体 AI 工作台

> 只看安装和使用的话，读这一份就够了 → [使用说明.md](./使用说明.md)
> 下面是开发文档，包含架构、数据库、模块实现细节。

面向「心身同调」领域学员的本地桌面工具，帮他们完成内容创作与引流获客。
界面参考 Obsidian + Notion 的深色内容工作台：卡片式布局、绿色主色调、顶部横向主导航。

## 技术栈

| 层 | 选型 |
| --- | --- |
| 壳 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite |
| UI | Naive UI（深色主题 + 自定义 themeOverrides） |
| 状态 | Pinia |
| 路由 | Vue Router（hash 模式） |
| 数据库 | SQLite（Rust 侧 rusqlite，bundled） |
| 图标 | 内联 SVG 组件（零依赖） |

> 关于图标：环境离线，无法安装 `lucide-vue-next` / `@vicons/ionicons5`，
> 因此 `src/components/AppIcon.vue` 内置了一套 24 网格线性图标（stroke 1.7px），
> 观感与 lucide 一致。网络可用后可以换成图标库，只需替换该组件内部实现。

## 命令

```bash
# 1. 创建项目（本项目已创建完成，此命令仅供参考，不要在现有目录里重复执行）
pnpm create tauri-app media-ai-workbench --template vue-ts

# 2. 安装依赖
pnpm install

# 3. 启动桌面应用（推荐）
pnpm tauri dev

# 只跑前端（浏览器预览 http://localhost:1420，数据库自动降级为 localStorage）
pnpm dev

# 类型检查 + 前端打包
pnpm build

# 打包安装包
pnpm tauri build
```

## 怎么打开应用（重要）

有三种方式，别用错：

| 方式 | 命令 / 路径 | 说明 |
| --- | --- | --- |
| ✅ 开发调试 | `pnpm tauri dev` | 会自动同时拉起前端 dev server 和桌面窗口 |
| ✅ 日常使用 | `src-tauri\target\release\media-ai-workbench.exe` | 正式版，前端已内嵌，**双击即可** |
| ✅ 浏览器预览 | `pnpm dev` 然后开 http://localhost:1420 | 只是看界面，数据走 localStorage |

### 为什么 `target\debug` 里的 exe 打开是白屏

`src-tauri\target\debug\media-ai-workbench.exe` 是**开发版**，它不内嵌前端，
启动时会去 `http://localhost:1420` 拉页面。如果你没同时跑 `pnpm tauri dev`，
或者端口被占用导致 dev server 没起来，这个窗口就是白的。

要一个能直接双击运行的版本，用 `pnpm tauri build --no-bundle`，
产物在 `src-tauri\target\release\media-ai-workbench.exe`。

> 直接双击 `dist\index.html` 也不会白屏（vite 的 `base` 已设为 `./`），
> 但那种方式没有 Rust 后端，数据库相关功能不可用，只适合看界面。

> 离线安装：本机 pnpm store 在 `C:\Users\admin\AppData\Local\pnpm\store`（沙箱外不可写）。
> 仓库根目录的 `.pnpm-local` 通过 junction 指向该 store，离线安装时用：
> `pnpm install --offline --store-dir ../.pnpm-local`

## 发版流程

### 一、改版本号

两个地方要一起改：

- `package.json` 的 `version`
- `src-tauri/tauri.conf.json` 的 `version`

其余地方（安装包文件名、Release 名称）都从这两个值推导，不用管。

### 二、本地打包

```bash
cd media-ai-workbench

# 注入体验 Key（可选；CI 里从 Secret 注入，本地可以用环境变量）
TRIAL_API_KEY=sk-你的体验key node scripts/inject-trial-key.mjs

pnpm install
pnpm tauri build                # 按当前系统出包
pnpm tauri build --bundles nsis # 只出 Windows 免安装安装器
pnpm tauri build --bundles msi  # 只出 MSI
```

产物在 `src-tauri/target/release/bundle/`：Windows 出 `nsis/*.exe` 和 `msi/*.msi`，
macOS 出 `dmg/*.dmg`。

**macOS 打包**必须在 Mac 上做，脚本已经写好：

```bash
./scripts/build-macos.sh          # 通用包（Intel + Apple Silicon 都支持）
./scripts/build-macos.sh --arm    # 只打 Apple Silicon
./scripts/build-macos.sh --intel  # 只打 Intel
```

### 三、用 GitHub Actions 自动出包

工作流：`.github/workflows/release.yml`，两种触发方式：

| 触发 | 效果 |
| --- | --- |
| 推 `v*` 标签（如 `git tag v0.1.0 && git push --tags`） | 三个平台各自出包 + 建一个**草稿 Release**，确认无误后手动发布 |
| Actions 页面点 **Run workflow** | 只出构建产物（Artifacts 里下载），不建 Release。**没有 Mac 时用这个验证 dmg** |

需要的仓库配置（都是可选的，不配也能出包）：

| Secret | 用途 |
| --- | --- |
| `TRIAL_API_KEY` | 体验 Key，构建时写进 `src-tauri/config/trial.json`。不配就用仓库里的占位值（应用会显示「体验 Key 未配置」） |
| `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` / `APPLE_SIGNING_IDENTITY` | macOS 代码签名证书（需要 Apple 开发者账号，$99/年） |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | macOS 公证（notarization），配了用户打开就不会被 Gatekeeper 拦 |

不配 Apple 相关 Secret 时，打出来的是**未签名的 dmg**，用户第一次打开要「右键 → 打开」，
《使用说明.md》里已经写了这个方法。

### 四、发布前检查

- [ ] `cargo test`（在 `src-tauri/` 下）全绿
- [ ] `pnpm build`（含 `vue-tsc` 类型检查）通过
- [ ] 版本号已在两个文件里改好
- [ ] 把安装包在一台干净环境里走一遍 [干净环境验证清单](./docs/干净环境验证清单.md)
- [ ] 确认体验 Key 是真实 Key（不是 `sk-在此填入体验Key`）
## 目录结构

```
media-ai-workbench/
├── index.html                 入口 HTML
├── vite.config.ts             固定 1420 端口，忽略 src-tauri 热更新
├── tsconfig.json              严格模式 + noUnusedLocals
├── src/
│   ├── main.ts                挂载 Vue / Pinia / Router
│   ├── App.vue                注入 Naive UI 深色主题与配色覆盖
│   ├── router/index.ts        路由表 + navItems（顶部导航数据源）
│   ├── layouts/
│   │   └── AppLayout.vue      顶栏（品牌 / 居中导航 / 状态胶囊）+ 内容区
│   ├── views/                 页面
│   │   ├── HomeView.vue       首页（12 栅格拼图）
│   │   ├── FlowView.vue       内容流程（占位）
│   │   ├── TrendsView.vue     今日热点（占位）
│   │   ├── TopicsView.vue     选题（占位 + 5 个子导航）
│   │   ├── CreateView.vue     创作（常用助手 + 人物小传入口 + 4 个结构区块）
│   │   ├── PublishView.vue    发布与经营（占位 + 01/02 结构）
│   │   ├── PlanView.vue       计划（占位 + 5 个结构区块）
│   │   ├── PersonasView.vue   人物小传（列表 / 筛选 / 搜索 / 编辑）
│   │   ├── MaterialsView.vue  素材管理（上传 / 网格列表 / 查看）
│   │   ├── ManuscriptsView.vue 文案管理（草稿箱 / 富文本编辑 / 版本历史）
│   │   ├── MemoryView.vue     工作记忆库（索引 / 检索 / 导出导入）
│   │   └── SettingsView.vue   设置（模型密钥 / 体验额度 / 昵称 / 数据库）
│   ├── components/
│   │   ├── AppIcon.vue        内联 SVG 图标集
│   │   ├── TopNav.vue         顶部横向主导航（7 项）
│   │   ├── WorkspaceCard.vue  卡片外壳（12px 圆角 / 标题行 / actions 插槽）
│   │   ├── PlaceholderPage.vue 通用占位页（hero + 子导航 + 区块网格）
│   │   ├── AiKeyGuide.vue     获取 API Key 的图文教程
│   │   ├── AiQuotaGuide.vue   体验额度用完的引导弹窗
│   │   ├── persona/           人物小传组件（卡片 / 编辑器 / 标签输入）
│   │   ├── material/          素材组件（卡片 / 查看器）
│   │   ├── TagInput.vue       通用标签输入
│   │   ├── RichTextEditor.vue 轻量富文本编辑器
│   │   └── home/              首页各卡片
│   │       ├── MissionCard.vue     今天先做什么
│   │       ├── GreetingCard.vue    插画 + 时钟 + 问候
│   │       ├── CalendarCard.vue    日历（今天 / 周 / 日 / 列表）
│   │       ├── HotspotCard.vue     今日 AI 热点（占位）
│   │       ├── BenchmarkCard.vue   对标博主动态（占位）
│   │       ├── TodoCard.vue        今日待办 + 进度条
│   │       ├── PomodoroCard.vue    番茄钟
│   │       ├── AssistantCard.vue   常用助手按钮网格
│   │       ├── QuickNoteCard.vue   随手记录
│   │       └── InboxCard.vue       待处理 + 分类统计
│   ├── stores/
│   │   ├── app.ts             数据库状态、版本
│   │   ├── tasks.ts           今日待办（todos 表 type=task）
│   │   ├── inbox.ts           待处理（todos 表 idea/video/article/text）
│   │   ├── notes.ts           随手记录（notes 表）
│   │   ├── ai.ts              AI 配置、密钥状态、体验额度、调用
│   │   ├── personas.ts        人物小传（列表 / 筛选 / 搜索 / AI 提取）
│   │   ├── materials.ts       素材（导入 / 分类标签 / 筛选搜索 / AI 描述）
│   │   ├── focus.ts           番茄钟计时
│   │   └── workspace.ts       今日主线 + 常用助手清单
│   ├── api/
│   │   ├── invoke.ts          command 调用 + 结构化错误归一
│   │   ├── db.ts              通用 CRUD command 封装 + 浏览器降级
│   │   ├── repositories.ts    领域仓储（todosRepo / notesRepo / settingsRepo）
│   │   ├── ai.ts              AI command 封装
│   │   ├── personas.ts        人物小传 CRUD / 提示词 / AI 提取
│   │   ├── materials.ts       素材导入 / 删除 / 分类 / 提示词 / AI 描述
│   │   └── index.ts           initDatabase() 与统一出口
│   ├── composables/
│   │   └── useAssistant.ts    常用助手执行逻辑（可带上人物档案）
│   ├── config/prompts.ts      品牌口吻与助手提示词
│   ├── utils/
│   │   ├── storage.ts         localStorage 读写（浏览器预览用）
│   │   └── datetime.ts        本地日期时间格式化
│   ├── styles/
│   │   ├── variables.css      设计变量（配色 / 圆角 / 间距 / 字体）
│   │   ├── global.css         重置 + 滚动条 + .btn/.field/.chip 工具类
│   │   └── theme.ts           Naive UI themeOverrides
│   └── types/index.ts         共享类型
└── src-tauri/
    ├── Cargo.toml             tauri 2 + rusqlite(bundled)
    ├── tauri.conf.json        窗口 1360×900 / 深色背景 / 中文标题
    ├── capabilities/default.json
    └── src/
        ├── main.rs
        ├── lib.rs             Builder + 全局 AppState(Mutex<Connection>)
        ├── commands.rs        Tauri command（通用 CRUD / 设置 / AI 用量 / 自检）
        ├── db/
        │   ├── mod.rs         数据库路径解析 + 连接 + PRAGMA
        │   ├── migrations.rs  迁移框架与建表 SQL（13 张核心表）
        │   ├── service.rs     通用 CRUD + 白名单校验 + 结构自省
        │   └── tests.rs       单元测试
        └── services/
            ├── secrets.rs     API Key 本地加密（AES-256-GCM）
            ├── ai/            AI 适配层（见下文）
            ├── personas/      人物小传（字段结构 / 提示词组装 / AI 提取）
            ├── materials/     素材管理（文件库 / 缩略图 / 类型判定 / AI 描述）
            ├── writing/       AI 写作（文体规范 / 3 版生成 / 配图建议 / 对话式修改）
            └── memory/        本地工作记忆库（自动索引 / 检索 / 导出导入）
```

## 顶部主导航

数据源是 `src/router/index.ts` 里的 `navItems`，新增页面只要往数组里加一项。

- 位置：`AppLayout` 顶栏用 `grid-template-columns: 1fr auto 1fr`，
  左侧品牌、中间导航、右侧状态与设置入口，因此导航在整窗内严格居中，两侧是留白。
- 选中态：`background: #b8d4a8`（浅绿）+ `color: #16200f`（深色文字）。
- 未选中态：`background: #1b201b`（卡片深色）+ `color: #aab3a7`（浅色文字）。
- 点击即 `RouterLink` 切路由（hash 模式，打包后 `file://` 下也正常）。

## 首页卡片布局实现思路

1. **一个 12 栅格容器**：`HomeView.vue` 里 `.home-grid` 使用
   `grid-template-columns: repeat(12, minmax(0, 1fr))`，`gap: 16px`。
   每张卡片通过 `.span-12 / .span-7 / .span-6 / .span-5` 决定占几列，
   宽度自适应、卡片等高（`align-items: stretch`）。
2. **卡片统一外壳**：所有卡片包在 `WorkspaceCard` 里，它负责 12px 圆角、
   1px 描边、`#1b201b` 背景、标题行与 `actions` 插槽；
   卡片内部只关心自己的内容，样式不会互相污染。
3. **六行栅格映射**：

   | 行 | 左（列宽） | 右（列宽） |
   | --- | --- | --- |
   | 1 | 今天先做什么（12，横向铺满：主线 / 待办 / 待处理三块） | — |
   | 2 | 插画 + 时钟 + 问候（7） | 日历（5） |
   | 3 | 今日 AI 热点（6） | 对标博主动态（6） |
   | 4 | 今日待办 + 进度条（6） | 番茄钟（6） |
   | 5 | 常用助手（6） | 随手记录（6） |
   | 6 | 待处理 + 分类统计（6） | 预留位（6） |

4. **卡内再分栏**：卡片内部用 flex / 小网格排版，
   例如第一行用 `grid-template-columns: 1.4fr 1fr 1fr` 分成主线 + 两个计数块。
5. **响应式**：窗口宽度 ≤1180px 时所有卡片改为 `span 12` 单列堆叠，
   导航文字折叠只剩图标，保证小窗口可用。
6. **交互**：时钟每秒刷新，日历可翻月与切换今天/周/日/列表，
   待办可勾选/新增/删除并实时更新进度条，番茄钟是真实的 25 分钟倒计时，
   随手记录会写进待处理箱并刷新分类统计。

## 数据流

```
Vue 组件 → Pinia store → src/api/repositories.ts（领域方法）
                       → src/api/db.ts（通用 command）
                       → invoke() → Rust commands.rs → db/service.rs → rusqlite → SQLite
                       ↘ 不在 Tauri 时自动退回 localStorage
```

## 数据库层

### 文件位置

| 平台 | 路径 |
| --- | --- |
| Windows | `%APPDATA%\自媒体AI工作台\workbench.db` |
| macOS | `~/Library/Application Support/自媒体AI工作台/workbench.db` |

首次启动自动建目录、建库、跑迁移；顶栏右侧状态胶囊会显示「本地数据库已就绪」。
连接的 PRAGMA：`journal_mode=WAL`、`synchronous=NORMAL`、`foreign_keys=ON`、`busy_timeout=5000`。

### 迁移机制

`src-tauri/src/db/migrations.rs` 里的 `MIGRATIONS` 是一个递增列表，
每条迁移一个事务，执行完把版本写进 `PRAGMA user_version`。
已应用过的版本会被跳过，所以重复启动不会重复建表。
以后改表结构，往数组后面追加一条即可，不要修改历史条目。

### 13 张核心表

| # | 表 | 用途 | 关键字段 |
| --- | --- | --- | --- |
| 1 | `settings` | 键值配置 | `key`(PK)、`value`、`updated_at` |
| 2 | `personas` | 人物小传 | `category`(self_ip/target_customer/case)、`fields`(JSON) |
| 3 | `materials` | 素材 | `type`(image/video/audio/text)、`file_path`、`tags` |
| 4 | `writings` | 文案 | `status`(draft/ready/published)、`persona_id`、`topic_id`、`images`(JSON) |
| 5 | `topics` | 选题 | `source`(课程/评论区/热点/对标/已发布)、`status` |
| 6 | `bloggers` | 对标博主 | `platform`(小红书/抖音/视频号)、`homepage_url` |
| 7 | `blogger_posts` | 对标博主内容 | 互动数据、`is_abnormal`、`status`(new/added/ignored) |
| 8 | `todos` | 待办 | `type`(task/idea/video/article/text)、`status`、`plan_date` |
| 9 | `notes` | 随手记录 | `content`、`converted_to` |
| 10 | `plans` | 计划 | `period`(month/week)、`goal`、`breakdown`(JSON) |
| 11 | `pomodoro` | 番茄钟记录 | `task`、`duration`(秒)、`started_at`、`finished_at` |
| 12 | `memories` | 记忆库索引 | `source_type`、`source_id`、`content`、`embedding` |
| 13 | `ai_usage` | AI 调用记录 | `model`、`type`、`tokens`、`is_free_trial` |

另外建了 12 个索引（状态、日期、外键列），`blogger_posts` 通过外键挂在 `bloggers` 上并级联删除。

### 通用 CRUD command（一套顶所有表）

| command | 说明 |
| --- | --- |
| `db_status` | 数据库就绪状态、文件路径、SQLite 版本、schema 版本、表数量 |
| `db_describe` | 所有业务表的列定义 + 行数 |
| `db_list(table, query)` | 查询列表，支持 filters / orderBy / desc / limit / offset |
| `db_get(table, id)` | 按主键查单条 |
| `db_insert(table, data)` | 插入并返回落库后的整行 |
| `db_update(table, id, data)` | 更新并返回更新后的整行，自动刷新 `updated_at` |
| `db_delete(table, id)` | 删除，返回是否命中 |
| `db_count(table, filters)` | 计数 |
| `db_self_test` | 写一条 `notes` 再读回来，验证整条链路 |
| `setting_get` / `setting_set` / `setting_all` | 键值配置（`settings` 主键是 `key`，单独处理） |
| `ai_usage_record` / `ai_usage_summary` | 记录 AI 调用、统计免费体验次数与 tokens |

`filters` 支持 `eq / ne / gt / gte / lt / lte / like / notLike / in / isNull / notNull`。

**安全约定**：表名和列名一律先过 `TABLES` 白名单再拼进 SQL，所有值都用占位符绑定，
所以 `db_list("todos; DROP TABLE todos")` 这类输入会直接报错。单元测试里有对应断言。

### 前端用法

通用层（`src/api/db.ts`）：

```ts
import { dbInsert, dbList } from '@/api/db'

const rows = await dbList<Todo>('todos', {
  filters: [{ column: 'status', op: 'eq', value: 'pending' }],
  orderBy: 'id',
  desc: false,
  limit: 20,
})
```

领域层（`src/api/repositories.ts`）把常用条件固化成语义方法，内部还是走同一套 command：

```ts
import { todosRepo, notesRepo, settingsRepo } from '@/api/repositories'

await todosRepo.create('写一条肩颈紧张的口播稿', 'task')  // -> todos 表
await todosRepo.listInbox()                              // -> idea/video/article/text
await notesRepo.create('学员的这句话很打动人', null)        // -> notes 表
await settingsRepo.set('current_model', 'gpt-5')         // -> settings 表
```

**字段命名**：行数据保持数据库列名（snake_case，如 `plan_date`、`created_at`），
查询参数用 camelCase（`orderBy`、`desc`），这样前端不需要写任何字段映射代码。

### 跑测试

```bash
cd src-tauri
cargo test --offline
```

覆盖：13 张表建表与幂等迁移、todos 增删改查、settings upsert、JSON 列落库、
外键级联删除、非法表名/列名/枚举值被拒绝。

## AI 适配层

```
src-tauri/src/services/
├── secrets.rs            API Key 本地加密（AES-256-GCM）
└── ai/
    ├── mod.rs            门面：chat() / test_connection() / generate_image()（预留）
    ├── provider.rs       五个模型供应商与元信息
    ├── adapters/         每个模型一个 adapter（deepseek / qwen / doubao / kimi / glm）
    ├── openai_compat.rs  统一的请求构造与响应解析
    ├── transport.rs      传输层抽象（真实用 reqwest，测试用 Mock）
    ├── routing.rs        决定这次用体验 Key 还是用户自己的 Key
    ├── trial.rs          内置体验 Key 与免费额度（读 config/trial.json）
    ├── types.rs          统一入参出参（ChatRequest / ChatResponse / ImageRequest）
    ├── error.rs          统一错误 + 面向学员的友好文案
    └── commands.rs       暴露给前端的 Tauri command
```

五家都提供 OpenAI 兼容的 `chat/completions`，所以公共逻辑放在 `openai_compat.rs`，
每个 adapter 只声明自己的接入点和默认模型：

| 模型 | 默认模型名 | 接口 | Key 申请页 |
| --- | --- | --- | --- |
| DeepSeek | `deepseek-chat` | api.deepseek.com | platform.deepseek.com/api_keys |
| 通义千问 | `qwen-plus` | dashscope.aliyuncs.com（兼容模式） | bailian.console.aliyun.com |
| 豆包 | `doubao-pro-32k` | ark.cn-beijing.volces.com | console.volcengine.com/ark |
| Kimi | `moonshot-v1-8k` | api.moonshot.cn | platform.moonshot.cn |
| 智谱 GLM | `glm-4-flash` | open.bigmodel.cn | open.bigmodel.cn/usercenter/apikeys |

> 豆包通常要求模型名填「推理接入点 ID」（`ep-xxxx`），在设置页把模型名换成自己的接入点即可。

### 免费体验逻辑

体验 Key 写在 `src-tauri/config/trial.json`（已编译进程序，换成真实 Key 后重新打包即可）：

```json
{
  "enabled": true,
  "freeQuota": 3,
  "provider": "deepseek",
  "model": "deepseek-chat",
  "apiKey": "sk-在这里填你的体验 Key"
}
```

调用时的选 Key 规则（`routing.rs`）：

1. 用户为当前选中的模型填了自己的 Key → 用自己的（不消耗体验额度）；
2. 没填，且体验额度没用完 → 走内置体验 Key；
3. 都没了 → 返回 `quota_exhausted`，前端弹出图文引导，教用户去注册、拿 Key、粘贴。

额度用尽时代码会在**发出网络请求之前**就返回错误，不会白白消耗一次请求。
调用成功才写 `ai_usage`；失败（Key 无效、网络问题）不记录，所以不会因为一次网络波动扣掉学员的免费次数。

### API Key 加密

- 算法 AES-256-GCM，每次加密用新的随机 nonce，密文格式 `v1:base64(nonce || ciphertext)`；
- 主密钥 32 字节，优先托管在系统凭据管理器（Windows 凭据管理器 / macOS 钥匙串），
  取不到时退化为应用数据目录下的 `secrets.key`，再不行才退到进程内存并打日志；
- 数据库里只存密文，设置页只显示打码结果（`sk-123••••cdef`）；
- 设置页会显示当前主密钥来源。

### 错误码

前端按 `code` 分支，`message` / `hint` 直接展示：

| code | 触发场景 | 处理 |
| --- | --- | --- |
| `invalid_key` | 401 / 403 | 提示重新复制 Key |
| `insufficient_balance` | 402 | 提示充值或换模型 |
| `rate_limited` | 429 | 提示稍后重试 |
| `network_error` | 连接超时 / 失败 | 提示检查网络或代理 |
| `quota_exhausted` | 体验额度用完 | **弹图文引导** |
| `trial_not_configured` | 体验 Key 还是占位值 | 提示配置 trial.json |
| `missing_key` | 没有可用 Key | 提示去设置页填 |
| `provider_error` | 其它 4xx / 5xx | 提示换模型或稍后重试 |

### 相关测试

```bash
cd src-tauri
cargo test                    # 29 项：适配器、路由、加密、错误映射、完整额度流程
cargo test -- --ignored       # 需要联网：五家接口真实可达性 + 无效 Key 映射
```

`trial_then_user_key_flow_persists_across_restart` 这项测试用真实 SQLite 文件完整走查了验收流程：
不填 Key 连用 3 次 → 第 4 次被拦住 → 填自己的 Key 恢复 → 重新打开数据库后昵称、加密 Key、剩余次数都还在。

## 人物小传

入口有三处：**创作页的常用助手**（第一个就是「人物小传」）、设置页的「快捷入口」、以及顶部右上角进入设置后再点。

### 数据怎么存

`personas` 表只有 `name` 和 `category` 两个固定列，其余全部字段打包成 JSON 存在 `fields` 列里。
好处是加字段不用改表结构，也不用写迁移。

| 表单字段 | JSON 键 | 用途 |
| --- | --- | --- |
| 姓名 / 昵称 | — （表列 `name`） | 列表标题 |
| 分类 | — （表列 `category`） | `self_ip` 自我 IP / `target_customer` 目标客户 / `case` 案例人物 |
| 年龄 | `age` | 语气分寸 |
| 身份标签 | `identity` | 一句话说清「他是谁」 |
| 核心经历 | `experience` | 讲故事的素材 |
| 性格特点 | `traits`（数组，3-5 个） | 标签输入 |
| 语言风格 | `catchphrases` / `expressions` | 口头禅、常用表达 |
| 目标受众 | `audience` / `painPoints` | 说给谁听、他卡在哪 |
| 禁忌话题 | `taboos` | 不能碰的红线 |
| 代表观点 | `viewpoints` | 反复强调的立场 |
| 视觉风格偏好 | `visualTone` / `visualStyle` / `visualScene` | 留给后续图片生成 |
| 来源说明 | `sourceNote` | AI 提取时自动写 |

### 列表页

卡片式展示，支持：

- 按分类筛选（全部 / 自我 IP / 目标客户 / 案例人物），每个分类带数量；
- 关键词搜索，命中范围包括姓名、身份、经历、受众、痛点、代表观点、口头禅和性格关键词；
- 每张卡片可以直接「编辑」「删除」，或者复制**写作提示词** / **图片提示词**。

数据量是几十条量级，筛选和搜索放在前端做（`stores/personas.ts` 的 `filtered`），
输入即时出结果，不用来回查库。以后档案多了可以改成走 `db_list` 的 `like` 过滤。

### 供其他模块引用

Rust 侧 `persona_prompt_block(id, purpose)` 把档案转成提示词块，`purpose` 支持三种：

| purpose | 带上哪些字段 | 给谁用 |
| --- | --- | --- |
| `writing` | 年龄、身份、经历、性格、口头禅、常用表达、受众、痛点、禁忌、代表观点 | AI 写作 |
| `interview` | 同上（去掉视觉偏好） | 模拟采访 |
| `image` | 只带色调、风格、场景、身份标签 | 图片生成 |

前端对应 `personasStore.promptBlock(id, purpose)`。
创作页的常用助手已经接上了：选好人物后点「写文案」，会把这份档案的提示词块拼进 prompt，
所以写出来的语气、立场和禁忌都跟着人物走。档案是空的时，返回的提示词块会带一句
「（这份人物档案还没有填写内容）」，避免把空白人设喂给模型。

### AI 提取特征

编辑器顶部展开「AI 提取特征」，粘贴一段已有文案（≥20 字）就能自动填字段：

```
personasStore.extract(文案)
  → command ai_extract_persona
  → services/ai 的 run_text（和普通生成走同一套 Key 选择和额度控制）
  → 模型返回 JSON → parse_extraction 解析（容错代码块和前后寒暄）
  → 只填充空字段，不覆盖你已经写好的内容
```

提取同样会消耗 AI 调用次数并记进 `ai_usage`，所以体验额度用完时照样弹引导框。

### 相关测试

```bash
cd src-tauri
cargo test
```

人物小传相关：分类标签、purpose 往返、JSON 解析容错（纯 JSON / 代码块 / 带寒暄 / 缺字段 / 非法内容）、
三种 purpose 的提示词块内容、空档案兜底，以及 `persona_crud_round_trip_on_a_real_database`
——在真实 SQLite 文件上跑完「建 → 读 → 按分类筛 → 搜 → 改 → 删」。

## 素材管理

入口在**创作页的「找资料」**，也可以从设置页快捷入口，或直接访问 `#/materials`。

### 文件存在哪

```
%APPDATA%\自媒体AI工作台\
├── workbench.db          数据库（只存路径和元数据）
└── 素材库\
    └── 2026-09\
        ├── 1757851234567_封面图.png        原始文件副本
        └── 1757851234567_封面图.thumb.jpg  自动生成的缩略图
```

导入时会把文件**复制**进素材库（按年月分目录、加时间戳前缀防重名），数据库只登记路径、
大小、标签、描述这些元数据。删除素材时会连文件一起删，但**只删素材库目录内的文件**——
`ensure_inside_library` 会做规范化路径校验，即使数据库被改坏也不会误删用户其它文件。

### 支持的类型与上传方式

| 类型 | 扩展名 | 缩略图 |
| --- | --- | --- |
| 图片 | jpg / jpeg / png / webp / gif / bmp | Rust 侧用 `image` crate 生成（长边 480px JPEG） |
| 视频 | mp4 / mov / m4v / webm | 前端抓首帧回传落盘 |
| 音频 | mp3 / wav / m4a / aac / ogg | 无（列表显示波形占位） |
| 文本 | txt / md / markdown / csv / json | 无（查看时直接显示内容） |

两种导入方式：

- **拖拽**：走 Tauri 的窗口级 `onDragDropEvent`，直接拿到真实路径，不经过 IPC 传输文件内容；
- **点击选择**：Rust 侧用 `rfd` 打开系统文件选择框拿路径。

> 为什么视频缩略图放在前端生成：打包 ffmpeg 太重。前端用 `<video>` + canvas 抓一帧，
> 编码、兼容性都交给 WebView。抓不到时（编码不支持、画布被跨域污染）就退化成占位图，
> 不影响素材使用。图片则完全在 Rust 侧处理，不依赖 WebView 的解码能力。

### 分类、标签与搜索

- **分类**：`material_categories` 表，可自定义增删。删分类不会删素材，素材会回到「未分类」
  （外键 `ON DELETE SET NULL`）。
- **标签**：多标签，以逗号分隔存在 `materials.tags` 列里，也可以在查看器里直接编辑。
- **筛选**：类型（图片/视频/音频/文本，各带数量）+ 分类 + 标签三个维度可以叠加。
- **搜索**：同时匹配名称、标签、类型和描述。

筛选与搜索放在前端 `stores/materials.ts` 的 `filtered` computed 里做，输入即时出结果；
SQL 层的 `like` 过滤同样可用（`importing_a_real_image_creates_file_thumbnail_and_row` 里验证过）。

### AI 素材描述

「AI 描述」会调用 `ai_describe_material`：

1. 图片优先用缩略图、视频用首帧，转成 base64 data URL；
2. 走统一的 `run_text`（同一套 Key 选择、额度控制、`ai_usage` 记录）；
3. 模型返回 `{ description, tags }`，解析后**自动写回**数据库（描述覆盖、标签合并去重）。

图片是作为 OpenAI 多模态格式的 `image_url` 部分发出去的，所以需要选**支持看图的模型**：

| 供应商 | 可用的视觉模型 |
| --- | --- |
| 通义千问 | `qwen-vl-plus`、`qwen-vl-max` |
| 豆包 | `doubao-vision-pro-32k` |
| Kimi | `moonshot-v1-8k-vision-preview` |
| 智谱 GLM | `glm-4v-flash`、`glm-4v-plus` |
| DeepSeek | 暂无视觉模型（只能按名称/标签生成描述） |

选错模型不会崩，会返回「模型不支持看图」这类友好提示；返回结果里的 `sentImage` 字段
也会告诉前端这次到底有没有把图发给模型。

### 供其它模块引用

`material_prompt_block(id, purpose)` 支持三种用途：

| purpose | 说明 |
| --- | --- |
| `writing` | 给 AI 写作：名称、类型、标签、描述、文件路径，并提示「可以参考或引用」 |
| `image` | 给图片生成：作为画面参考，保持色调与风格一致 |
| `video` | 给视频剪辑：作为剪辑素材，按时间线拼进成片 |

前端对应 `materialsStore.promptBlock(id, purpose)`，查看器里有三个「复制提示词」按钮。

### 数据库迁移

素材模块引入了 **v2 迁移**（`materials_library`）：新建 `material_categories` 表并写入 4 个默认分类，
给 `materials` 补 `category_id` / `thumb_path` / `size_bytes` / `source_path` / `ai_description_at` 五列。
老数据库升级时会自动执行，不会丢数据（实测 `user_version` 从 1 升到 2，原表数据保留）。

## 创作页与 AI 写作

### 页面结构

创作页三块，从上到下：

1. **最近正在做**：按更新时间列出最近 3 篇文案，每条带「继续这篇」——点它会把标题、正文、文体、
   人物小传一起装进写作台，接着改。
2. **这次创作哪篇内容**：下拉选「从新想法开始」或某篇已有文案；旁边是「补充你的想法」和「主题」。
3. **创作白板**：3×3 网格，中心是当前内容的实时状态（主题 / 文体 / 人物 / 已有几版），
   周围 6 个动作，位置固定：

   ```
   找资料    起标题    做封面
   学习改稿  [当前内容] 写文案
             发布与复盘
   ```

   找资料跳素材库，发布与复盘跳发布页，做封面用配图关键词拼出提示词（图片模块接入后直接可用），
   起标题和学习改稿走常用助理（会带上当前选中的人物小传）。

### AI 写作主界面

点「写文案」打开，左右分栏：

- **左侧输入**：主题 / 文体（小红书、公众号、短视频脚本、朋友圈）/ 人物小传（可选）/
  素材引用（可选，从素材库挑）/ 补充想法 → 「生成 3 版」
- **右侧结果**：版本切换标签、可编辑的标题与正文、模型与额度信息、
  对话式修改输入框、修改历史、配图建议列表、保存到文案库

### 生成逻辑

一次调用同时产出 3 个**不同切入角度**的版本（痛点切入 / 故事切入 / 金句切入），
避免只是换词。提示词分三层：

1. **品牌层**：心身同调领域、口语化、不做医疗承诺；
2. **文体层**：每个文体一份写作规范，小红书明确要求「标题 ≤20 字带情绪、正文 400-600 字、
   每段 1-3 行、每段 1-2 个 emoji、结尾 5-8 个 # 话题标签」；
3. **人物层**：选了人物小传，就把档案块写进 system prompt，并加上
   「必须遵守人物档案……让读者一眼能认出是他/她」。

素材引用会把素材的标签、描述、文件路径拼进 user prompt。

### 对话式修改

输入「再口语化一点」这类指令即可继续改。实现上把**之前的往返一起发给模型**
（`ChatRequest.history` → OpenAI 的 messages 数组），所以模型知道在改哪一版；
返回的新正文替换当前版本，同时把这一问一答追加进 `history`，保存时写进 `writings.history`。
界面上可以展开「修改历史」逐条查看。

### 配图关键词

和正文同一次生成返回，格式与产品要求一致：

```json
{
  "配图建议": [
    { "位置": "封面", "关键词": "温暖的卧室，柔和灯光，宁静氛围", "风格": "治愈系", "比例": "3:4" }
  ]
}
```

模型用中文键名返回，Rust 侧解析成结构化字段再以英文键名给前端（`position` / `keywords` /
`style` / `ratio`），两边都舒服。每条建议旁边有「生成图片」（当前提示图片模块未接入，会先复制提示词）
和「复制提示词」。

### 保存与发布

「保存到文案库」把标题、正文、文体、人物小传、配图建议、对话历史一起写进 `writings` 表
（`images` 列存配图建议 JSON，`history` 列存对话历史）。再次保存是更新同一篇。

保存后会出现在**发布与经营 → 01 已发布作品**里，可按草稿 / 待发布 / 已发布筛选，
支持展开看正文、标记为已发布、删除。

### 数据库迁移

**v3 迁移**（`writing_genre_and_history`）给 `writings` 加了 `genre` 和 `history` 两列。
实测老库升级 `user_version` 从 2 升到 3，原数据保留。

### 相关测试

```bash
cd src-tauri
cargo test
```

写作相关：四个文体的 id 往返与写作规范、小红书规范的关键要求（标题/emoji/分段/#）、
system prompt 是否嵌入人物档案、user prompt 是否带上主题/补充想法/素材与 JSON 契约、
3 版 + 配图建议的解析（纯 JSON / 代码块 / 带寒暄 / 缺字段 / 英文别名 / 非法内容）、
修改文本的解析，以及 `saving_a_writing_round_trips_genre_history_and_suggestions`
——在真实 SQLite 上存一篇、读回来验证文体/配图建议/历史、按更新时间列出、标记为已发布。

另外 `build_chat_request_replays_conversation_history` 验证对话历史会按
user / assistant 顺序进 messages，非法角色被忽略。

## 文案管理与发布经营

### 文案管理（草稿箱）

入口：创作页「最近正在做」右上角、发布页顶部，或直接访问 `#/manuscripts`。

左边是列表，右边是详情，一块屏就能改完：

- **筛选**：状态（全部 / 草稿 / 待发 / 已发，各带数量）+ 文体 + 关键词（标题、正文、标签）
- **编辑**：富文本编辑器（加粗 / 斜体 / 小标题 / 列表 / 引用 / 清格式），存的是 HTML
- **版本历史**：每次保存前把当前标题与正文压进历史，只保留最近 5 版，可一键载入某一版
- **状态流转**：草稿 → 待发 → 已发，点某个状态直接切换，或「一键切换」按顺序轮转；
  第一次标为已发会自动补上发布日期
- **关联**：人物小传（下拉）、选题（显示已关联的选题 id）、配图（展示这篇的配图建议并可复制提示词）
- **操作**：复制正文（去标签）、导出 txt、导出 markdown、删除

导出由 Rust 侧 `writing_export` 完成：`html_to_text` 把块级标签换成换行、解码实体、压掉多余空行；
`html_to_markdown` 额外把 `<h2>` 转成 `##`、`<strong>` 转成 `**`、`<li>` 转成 `- `。
markdown 导出会在开头补一行 `# 标题`，txt 导出会补标题和分隔线。

### 发布与经营 - 已发布作品

顶部按月份筛选（月份从作品的发布日期推导，没填就用更新时间），右侧按作品标题或主题搜索。
表格列与产品要求一致：

| 列 | 说明 |
| --- | --- |
| 作品标题 | 下面附文体与状态 |
| 主题 / 角色 | 角色取关联的人物小传，主题取关联的选题 |
| 发布日期 | 可直接改，改完自动保存 |
| 成交人数 / 客资量 / 引流私域人数 / 咨询人数 | 数字输入框，失焦即存 |
| 业务数据截至 | 日期输入；第一次填业务数据时自动补当天 |

业务数据直接挂在 `writings` 表上（一个人一篇作品，不值得再拆表）。

### 数据库迁移

**v4 迁移**（`writing_publish_stats`）给 `writings` 加了 7 列：
`published_at`、`version_history`、`deal_count`、`lead_count`、`private_count`、
`consult_count`、`stats_updated_at`。实测老库升级 `user_version` 从 3 升到 4，原数据保留。

### 相关测试

```bash
cd src-tauri
cargo test
```

新增：HTML → 纯文本（br 转换行、实体还原、空行压缩、不残留标签）、
HTML → Markdown（标题 / 加粗 / 斜体 / 列表）、导出文件名清洗（路径非法字符替换、空标题兜底、超长截断）。

## 本地工作记忆库

入口：首页「常用助手」里的**知识库维护管家**，或直接访问 `#/memory`。

### 自动索引

索引钩子挂在**通用 CRUD 命令**里（`db_insert` / `db_update` / `db_delete`），不在每个页面里各写一遍。
所以只要数据是走常规接口存进 `personas`、`materials`、`writings`、`topics` 这四张表的，
就会自动写进 `memories`，并且**以后新增的模块也自动被索引**。

每条记忆存 `source_type`（persona / material / writing / topic）+ `source_id` + `content`（文本摘要）。
摘要是按类型拼的，例如：

```
人物小传《小林》　分类：自我 IP　身份：心理咨询师　性格：温和、直接　受众：职场妈妈　痛点：辅导作业就发火
文案《总是睡不好》　文体：小红书　状态：draft　正文：很多人以为睡不着是脑子太兴奋…
```

同一份数据反复保存不会产生重复记忆（先按 source_type + source_id 删除再写入）；
删除数据时会连带删掉它的记忆。

### 检索

- **关键词检索**：`memory_list(source_type, keyword, limit)`，支持按来源类型筛选；
- **相关度排序**：命中的排前面，其次是字符二元组相似度高的（`similar`），列表会显示相关度；
- **给其它模块的 API**：`memory_search(query, source_types, limit)` 返回相关片段，
  `memory_prompt_block(query, limit)` 直接拼成可塞进 prompt 的「【工作记忆】」段落。

> 语义检索（P1）：目前用的是**关键词命中 + 字符二元组相似度**，属于本地近似匹配，
> 不做真正的语义理解（比如「失眠」和「睡不着」不会互相召回）。
> `memories.embedding` 列已预留，接入向量模型后把 `score()` 换成余弦相似度即可，
> 接口形状不用变。

### 管理与备份

- **重建索引**：清空后按四张业务表重新摘要一遍（幂等，重复重建不会翻倍）；数据删了但记忆还在的脏数据会被清掉。
- **删除单条**：只删索引，不动原始数据。
- **一键导出**：全部业务表导出为一个 JSON（含 `version`、`exportedAt`、`tables`），
  共 13 张表；`settings` 刻意排除——里面的 API Key 是密文，换台机器解不开。
- **导入恢复**：按 id 覆盖写入，关联关系（persona_id 等）保留；导入前会弹确认，
  因为会先清空对应表。

### 相关测试

四种来源的摘要拼装、富文本转纯文本、关键词/相似度打分（命中 > 部分重叠 > 无关）、
以及三项端到端：`data_changes_are_indexed_and_searchable`（建人物 + 写文案 → 各有一条记忆 →
关键词搜到 → 按来源筛 → 更新不重复 → 删除同步清理 → 检索 API 返回片段）、
`reindex_rebuilds_everything`、`export_and_import_round_trip`（导出 JSON 结构正确 →
导入到新库 → id 与关联关系保留 → 坏 JSON 给出友好错误）。
