use rusqlite::Connection;

/// 一条迁移。version 必须严格递增，且与 `PRAGMA user_version` 对应。
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// 迁移列表。以后要改表结构，往后面追加一条，不要改历史条目。
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "init_core_tables",
        sql: V1_INIT_CORE_TABLES,
    },
    Migration {
        version: 2,
        name: "materials_library",
        sql: V2_MATERIALS_LIBRARY,
    },
    Migration {
        version: 3,
        name: "writing_genre_and_history",
        sql: V3_WRITING_GENRE,
    },
    Migration {
        version: 4,
        name: "writing_publish_stats",
        sql: V4_WRITING_PUBLISH_STATS,
    },
];

/// 执行所有高于当前 user_version 的迁移，返回本次实际执行的版本号。
pub fn run(conn: &Connection) -> rusqlite::Result<Vec<i32>> {
    let current = schema_version(conn)?;
    let mut applied = Vec::new();

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        // 每条迁移单独一个事务：中途失败不会留下半张表。
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute_batch(&format!("PRAGMA user_version = {};", migration.version))?;
        tx.commit()?;

        println!(
            "[db] 已应用迁移 v{} ({})",
            migration.version, migration.name
        );
        applied.push(migration.version);
    }

    Ok(applied)
}

/// 当前 schema 版本
pub fn schema_version(conn: &Connection) -> rusqlite::Result<i32> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

const V1_INIT_CORE_TABLES: &str = r#"
-- ---------------------------------------------------------------- 1. 键值配置
-- 存 API Key（加密后的密文）、当前模型、免费体验次数等
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- ------------------------------------------------------------- 2. 人物小传
CREATE TABLE IF NOT EXISTS personas (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    category   TEXT NOT NULL DEFAULT 'self_ip'
               CHECK (category IN ('self_ip', 'target_customer', 'case')),
    -- 其余个性化字段以 JSON 存放，避免为每个小传维度建列
    fields     TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- ---------------------------------------------------------------- 3. 素材
CREATE TABLE IF NOT EXISTS materials (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    type        TEXT NOT NULL DEFAULT 'text'
                CHECK (type IN ('image', 'video', 'audio', 'text')),
    name        TEXT NOT NULL,
    file_path   TEXT,
    tags        TEXT NOT NULL DEFAULT '',
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_materials_type ON materials (type);

-- ---------------------------------------------------------------- 4. 文案
CREATE TABLE IF NOT EXISTS writings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    content    TEXT NOT NULL DEFAULT '',
    status     TEXT NOT NULL DEFAULT 'draft'
               CHECK (status IN ('draft', 'ready', 'published')),
    persona_id INTEGER REFERENCES personas (id) ON DELETE SET NULL,
    topic_id   INTEGER REFERENCES topics (id) ON DELETE SET NULL,
    -- 配图列表（素材 id 或本地路径的 JSON 数组）
    images     TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_writings_status ON writings (status);
CREATE INDEX IF NOT EXISTS idx_writings_topic ON writings (topic_id);

-- ---------------------------------------------------------------- 5. 选题
CREATE TABLE IF NOT EXISTS topics (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    source     TEXT NOT NULL DEFAULT '热点'
               CHECK (source IN ('课程', '评论区', '热点', '对标', '已发布')),
    status     TEXT NOT NULL DEFAULT 'pending'
               CHECK (status IN ('pending', 'writing', 'published')),
    tags       TEXT NOT NULL DEFAULT '',
    note       TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_topics_status ON topics (status);

-- ----------------------------------------------------------- 6. 对标博主
CREATE TABLE IF NOT EXISTS bloggers (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL,
    platform     TEXT NOT NULL DEFAULT '小红书'
                 CHECK (platform IN ('小红书', '抖音', '视频号')),
    homepage_url TEXT,
    avatar       TEXT,
    tags         TEXT NOT NULL DEFAULT '',
    created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_bloggers_platform ON bloggers (platform);

-- ------------------------------------------------------- 7. 对标博主内容
CREATE TABLE IF NOT EXISTS blogger_posts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    blogger_id      INTEGER NOT NULL REFERENCES bloggers (id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    url             TEXT,
    cover           TEXT,
    likes           INTEGER NOT NULL DEFAULT 0,
    collects        INTEGER NOT NULL DEFAULT 0,
    comments        INTEGER NOT NULL DEFAULT 0,
    shares          INTEGER NOT NULL DEFAULT 0,
    publish_date    TEXT,
    -- 数据异常标记：互动明显偏离时置 1，并记录原因
    is_abnormal     INTEGER NOT NULL DEFAULT 0,
    abnormal_reason TEXT,
    status          TEXT NOT NULL DEFAULT 'new'
                    CHECK (status IN ('new', 'added', 'ignored')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_blogger_posts_blogger ON blogger_posts (blogger_id);
CREATE INDEX IF NOT EXISTS idx_blogger_posts_status ON blogger_posts (status);

-- ---------------------------------------------------------------- 8. 待办
-- type=task 是「今日待办」，idea/video/article/text 是首页「待处理」的四类
CREATE TABLE IF NOT EXISTS todos (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    type       TEXT NOT NULL DEFAULT 'task'
               CHECK (type IN ('task', 'idea', 'video', 'article', 'text')),
    status     TEXT NOT NULL DEFAULT 'pending'
               CHECK (status IN ('pending', 'done')),
    plan_date  TEXT,
    done_at    TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_todos_status ON todos (status);
CREATE INDEX IF NOT EXISTS idx_todos_plan_date ON todos (plan_date);

-- ----------------------------------------------------------- 9. 随手记录
CREATE TABLE IF NOT EXISTS notes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    content      TEXT NOT NULL,
    -- 已转化为：type / topic / todo（或 'todo:12' 这种带目标 id 的形式）
    converted_to TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- ---------------------------------------------------------------- 10. 计划
CREATE TABLE IF NOT EXISTS plans (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    period     TEXT NOT NULL DEFAULT 'week' CHECK (period IN ('month', 'week')),
    goal       TEXT NOT NULL,
    -- 目标拆解，JSON 数组
    breakdown  TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_plans_period ON plans (period);

-- ------------------------------------------------------------ 11. 番茄钟
CREATE TABLE IF NOT EXISTS pomodoro (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    task        TEXT,
    -- 时长（秒）
    duration    INTEGER NOT NULL DEFAULT 0,
    started_at  TEXT,
    finished_at TEXT
);

-- ---------------------------------------------------------- 12. 记忆库索引
CREATE TABLE IF NOT EXISTS memories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL,
    source_id   INTEGER,
    content     TEXT NOT NULL,
    -- 语义检索用的向量（JSON 数组），可选
    embedding   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_memories_source ON memories (source_type, source_id);

-- --------------------------------------------------------- 13. AI 调用记录
-- 用于免费额度控制：统计 is_free_trial=1 的调用次数
CREATE TABLE IF NOT EXISTS ai_usage (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    model         TEXT NOT NULL,
    type          TEXT NOT NULL DEFAULT 'text' CHECK (type IN ('text', 'image')),
    tokens        INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    is_free_trial INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_ai_usage_created ON ai_usage (created_at);
"#;

/// v2：素材管理模块。
/// 素材文件放在应用数据目录的「素材库」下，数据库只存路径与元数据，
/// 所以这里补的是分类、缩略图、大小、原始路径这些描述性字段。
const V2_MATERIALS_LIBRARY: &str = r#"
-- 自定义分类 / 文件夹
CREATE TABLE IF NOT EXISTS material_categories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

INSERT OR IGNORE INTO material_categories (name) VALUES
    ('课程素材'), ('封面图'), ('背景音乐'), ('参考资料');

-- 素材补充字段
ALTER TABLE materials ADD COLUMN category_id INTEGER
    REFERENCES material_categories (id) ON DELETE SET NULL;
-- 缩略图路径（图片和视频）
ALTER TABLE materials ADD COLUMN thumb_path TEXT;
ALTER TABLE materials ADD COLUMN size_bytes INTEGER NOT NULL DEFAULT 0;
-- 导入时用户选择的原始路径，方便追溯
ALTER TABLE materials ADD COLUMN source_path TEXT;
-- 生成过 AI 描述的时间
ALTER TABLE materials ADD COLUMN ai_description_at TEXT;

CREATE INDEX IF NOT EXISTS idx_materials_category ON materials (category_id);
"#;

/// v3：AI 写作模块。文案需要记住「文体」和「对话式修改的历史」。
/// 配图建议复用已有的 images 列（本来就是配图列表）。
const V3_WRITING_GENRE: &str = r#"
ALTER TABLE writings ADD COLUMN genre TEXT NOT NULL DEFAULT 'xiaohongshu';
-- 对话式修改的完整往返记录（JSON 数组）
ALTER TABLE writings ADD COLUMN history TEXT NOT NULL DEFAULT '[]';

CREATE INDEX IF NOT EXISTS idx_writings_genre ON writings (genre);
"#;

/// v4：文案管理的版本历史 + 发布后的业务数据。
/// 业务数据直接挂在 writings 上：一个人一篇作品，不需要再拆表。
const V4_WRITING_PUBLISH_STATS: &str = r#"
-- 发布日期
ALTER TABLE writings ADD COLUMN published_at TEXT;
-- 最近 5 个版本的内容快照（JSON 数组）
ALTER TABLE writings ADD COLUMN version_history TEXT NOT NULL DEFAULT '[]';

-- 业务数据
ALTER TABLE writings ADD COLUMN deal_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE writings ADD COLUMN lead_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE writings ADD COLUMN private_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE writings ADD COLUMN consult_count INTEGER NOT NULL DEFAULT 0;
-- 业务数据截至
ALTER TABLE writings ADD COLUMN stats_updated_at TEXT;

CREATE INDEX IF NOT EXISTS idx_writings_published_at ON writings (published_at);
"#;
