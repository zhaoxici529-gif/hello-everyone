# AGENTS.md — 自媒体 AI 工作台

本文件记录这个项目的**开发与发版规范**。改代码前先读一遍，能省很多返工。

## 项目速览

- 位置：`media-ai-workbench/`（这个目录本身就是 git 仓库根目录）
- 技术栈：Tauri 2.x + Vue 3 + TypeScript + Naive UI + Pinia + Vue Router + SQLite(rusqlite)
- 面向用户：不懂技术的「心身同调」学员。**界面文案一律中文、口语化、不用术语**
- 产品定位：本地优先。数据都在用户自己电脑上，不上传

## 常用命令

```bash
cd media-ai-workbench
pnpm install
pnpm dev                       # 只跑前端（浏览器预览，没有 Rust 运行时）
pnpm tauri dev                 # 跑桌面应用（开发用，会自动起前端）
pnpm build                     # vue-tsc 类型检查 + vite 打包
cd src-tauri && cargo test     # Rust 单测（改后端必须跑）
cargo test -- --ignored        # 需要联网的测试（真实 API 可达性）
```

## 目录约定

```
src/
  api/         # 调 Tauri command 的封装（一模块一个文件）+ invoke.ts 统一错误归一
  stores/      # Pinia，一模块一个
  views/       # 页面，一模块一个
  components/  # 通用组件 + 每模块一个子目录
  layouts/     # AppLayout（顶部导航 + 滚动容器）
  styles/      # variables.css 设计变量 / global.css / theme.ts（Naive UI 覆盖）
src-tauri/src/
  db/          # migrations.rs（迁移）/ service.rs（通用 CRUD + 白名单）/ tests.rs
  services/    # ai / personas / materials / writing / memory / secrets
  commands.rs  # 通用 CRUD 命令（含记忆索引钩子）
```

**新增一个业务模块的固定套路**（照抄现成的，别自创结构）：

1. `src-tauri/src/services/<模块>/{mod.rs, commands.rs, tests.rs}` —— 业务逻辑 + 命令 + 测试
2. `src/types/index.ts` 加行数据类型（**行数据用数据库列名 snake_case**）
3. `src/api/<模块>.ts` 包命令；查询参数用 camelCase
4. `src/stores/<模块>.ts`
5. `src/views/<模块>View.vue` + `src/router/index.ts` 加路由
6. 入口：至少接一个「常用助手」或「设置页快捷入口」
7. `src-tauri/src/lib.rs` 注册命令
8. README 加一节说明

## 后端规范

- **数据库迁移**：`db/migrations.rs` 的 `MIGRATIONS` 数组**只追加、不改历史条目**，每条一个事务，版本号写进 `PRAGMA user_version`。当前已到 v4。
- **通用 CRUD**：所有表操作走 `db/service.rs`，表名列名先过 `TABLES` 白名单，值一律占位符绑定。不要自己拼 SQL。
- **表结构改动**要同步三处：`migrations.rs` 加迁移、`db/service.rs` 的 `TABLES` 里更新列、`db/tests.rs` 的 `CORE_TABLES` 数量。
- **AI 调用**统一走 `services/ai/commands.rs` 的 `run_text()`，它负责选 Key（体验额度 / 用户 Key）、落 `ai_usage`、错误归一。**不要绕过它直接发请求**，否则额度统计会不准。
- **错误给用户看的**：返回 `AiError { code, message, hint }`，`message` 说人话，`hint` 说怎么办。前端按 `code` 分支处理 `quota_exhausted`。
- **数据变更自动进记忆库**：钩子在 `commands.rs` 的 `db_insert/db_update/db_delete` 里，新表只要加进 `services/memory/mod.rs` 的映射就会被索引。

## 前端规范

- 只用**项目已有的依赖**。图标是 `AppIcon.vue` 里的内联 SVG（零依赖），不要引图标库。
- `AppIcon` 的 `name` 必须是 `ICONS` 里已有的键，否则会静默显示成文本图标。
- 按钮用全局类：`.btn` / `.btn-primary` / `.btn-ghost` / `.btn-sm`；输入框用 `.field`。
- 卡片统一包 `WorkspaceCard`，不要自己写圆角和背景。
- **模板里的坑**：不要在 `{{ }}` 里写 `<` 或正则字面量（Vue 解析器会当成标签），抽成函数；不要写空的 `<th></th>`。
- 深色主题的颜色一律用 `styles/variables.css` 的变量（`--accent`、`--text-1`、`--bg-card`…），不要写死颜色。
- 浏览器预览（`pnpm dev`）没有 Rust 运行时，所有 `invoke` 都会失败。写新功能时要考虑降级路径，并给「浏览器预览模式」的提示。

## 打包与发版规范

- **版本号两处同步改**：`package.json` 和 `src-tauri/tauri.conf.json`。
- **打包产物不入库**：`dist/`、`src-tauri/target/`、`自媒体AI工作台.exe`、`log/` 都已在 `.gitignore` 里。
- **体验 Key 不进仓库**：`src-tauri/config/trial.json` 里保留占位值，真实 Key 通过环境变量 `TRIAL_API_KEY`（CI 用 Secret）在构建前注入，脚本是 `scripts/inject-trial-key.mjs`。
- **发布用 tag 触发**：`git tag v0.1.0 && git push --tags` → GitHub Actions 出 Windows(.exe/.msi) + macOS(.dmg) 并建草稿 Release。细节见 README「发版流程」。
- **macOS 只能在 Mac 上打包**。本地用 `scripts/build-macos.sh`，或者用 Actions 的 `macos-14` runner（也是 Mac）。Windows 上不要尝试。
- **发版前必做**：`cargo test` 全绿 + `pnpm build` 通过 + 在干净环境按《docs/干净环境验证清单.md》走一遍。

## 环境相关的已知坑

这台开发机有些特殊情况，踩过一次就别再踩：

- **`apply_patch` 工具可能不可用**。此时用 PowerShell 做精确替换：先 `[System.IO.File]::ReadAllText($path, UTF8)`，`.Contains()` 校验锚点唯一后再 `.Replace()`，最后 `WriteAllText` 写回 **UTF-8 无 BOM**。文件换行符注意：本仓库是 `\n`，`.sh` / `.yml` 必须保持 LF。
- **没有 git**：这台机器 PATH 里没有 git，提交/推送到仓库要在有 git 的环境做。
- **网络受限**：沙箱内不通网，需要联网的操作要显式申请权限（下载依赖、访问 crates.io/GitHub）。
- **MSI 打包**：应用名是中文，WiX 必须用 `zh-CN` 代码页（已在 `tauri.conf.json` 配好）；WiX 工具链 Tauri 会自动下载，超时的话手动放到 `%LOCALAPPDATA%\tauri\WixTools314`。
- **窗口高度链**：Naive UI 的 `NConfigProvider` 会包一层 div，`global.css` 里的 `#app > * { height: 100% }` 不能删，删了页面就滚不动。
- **覆盖 `%APPDATA%` 环境变量对 Tauri 无效**：它的 `data_dir()` 走 Windows Shell API。想验证「首次启动」必须真的删/换数据目录。

## 验证要求

- 改完后端：`cd src-tauri && cargo test` 必须全绿，且不新增编译警告。
- 改完前端：`pnpm build` 必须通过（含 `vue-tsc` 类型检查）。
- **不要只靠「代码看起来对」就说完成**。能跑测试就跑测试；跑不了的（比如需要真实 API Key、需要点击原生窗口）要在回复里明确说「这一条我没验证，需要你确认」。
- 涉及用户数据的验证（删数据库、改数据目录）先备份，验证完立刻恢复，并在回复里说明做了什么。