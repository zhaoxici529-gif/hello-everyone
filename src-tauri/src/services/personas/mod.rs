//! 人物小传。
//!
//! 表结构是 `personas(id, name, category, fields, created_at, updated_at)`，
//! 明细字段统一放进 `fields` 这一列 JSON 里，所以加字段不用改表。

pub mod commands;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::ai::AiError;

/// 人物分类
pub const CATEGORIES: [&str; 3] = ["self_ip", "target_customer", "case"];

pub fn category_label(id: &str) -> &'static str {
    match id {
        "self_ip" => "自我 IP",
        "target_customer" => "目标客户",
        "case" => "案例人物",
        _ => "未分类",
    }
}

/// 引用方：写得清楚一点，方便别的模块按用途取提示词
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// AI 写作：需要语言风格 + 受众 + 禁忌
    Writing,
    /// 模拟采访：需要经历 + 观点 + 语言风格
    Interview,
    /// 图片生成：只要视觉偏好
    Image,
}

impl Purpose {
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "writing" => Some(Purpose::Writing),
            "interview" => Some(Purpose::Interview),
            "image" => Some(Purpose::Image),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Purpose::Writing => "writing",
            Purpose::Interview => "interview",
            Purpose::Image => "image",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Purpose::Writing => "AI 写作",
            Purpose::Interview => "模拟采访",
            Purpose::Image => "图片生成",
        }
    }
}

/// 人物档案明细。全部用空字符串代替 null，前端表单双向绑定更省事。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PersonaFields {
    /// 年龄
    pub age: String,
    /// 身份标签
    pub identity: String,
    /// 核心经历
    pub experience: String,
    /// 性格特点关键词（3-5 个）
    pub traits: Vec<String>,
    /// 口头禅
    pub catchphrases: String,
    /// 常用表达
    pub expressions: String,
    /// 目标受众是谁
    pub audience: String,
    /// 受众的痛点
    pub pain_points: String,
    /// 禁忌话题
    pub taboos: String,
    /// 代表观点
    pub viewpoints: String,
    /// 视觉风格：色调
    pub visual_tone: String,
    /// 视觉风格：风格
    pub visual_style: String,
    /// 视觉风格：场景
    pub visual_scene: String,
    /// 这份档案的来源说明（AI 提取时自动写）
    pub source_note: String,
}

impl PersonaFields {
    pub fn is_blank(&self) -> bool {
        *self == PersonaFields::default()
    }

    /// 组装成给模型看的档案块。
    /// 不同用途只带相关字段，避免把图片提示词里塞满人物经历。
    pub fn to_prompt_block(&self, name: &str, category: &str, purpose: Purpose) -> String {
        let mut lines = vec![format!(
            "【人物档案：{name}（{}）】",
            category_label(category)
        )];

        let mut push = |label: &str, value: &str| {
            if !value.trim().is_empty() {
                lines.push(format!("- {label}：{}", value.trim()));
            }
        };

        if purpose == Purpose::Image {
            push("喜欢的色调", &self.visual_tone);
            push("喜欢的风格", &self.visual_style);
            push("常用场景", &self.visual_scene);
            push("身份标签", &self.identity);
            return lines.join("\n");
        }

        push("年龄", &self.age);
        push("身份标签", &self.identity);
        push("核心经历", &self.experience);
        if !self.traits.is_empty() {
            push("性格特点", &self.traits.join("、"));
        }
        push("口头禅", &self.catchphrases);
        push("常用表达", &self.expressions);
        push("目标受众", &self.audience);
        push("受众痛点", &self.pain_points);
        push("禁忌话题", &self.taboos);
        push("代表观点", &self.viewpoints);

        if purpose == Purpose::Writing {
            push("喜欢的色调", &self.visual_tone);
        }

        lines.join("\n")
    }
}

/* --------------------------------- AI 提取 --------------------------------- */

pub const EXTRACTION_SYSTEM_PROMPT: &str = "你是人物档案整理助手。\n\
请从用户提供的文案中提取人物特征，只输出 JSON，不要输出解释、不要用 markdown 代码块。\n\
文案里没有的信息就留空字符串，不要编造具体事实；只有视觉风格可以按人物气质做合理推断。";

pub fn extraction_user_prompt(source: &str, name: Option<&str>) -> String {
    let mut prompt = String::new();

    if let Some(name) = name.map(str::trim).filter(|value| !value.is_empty()) {
        prompt.push_str(&format!("这位人物叫「{name}」。\n\n"));
    }

    prompt.push_str(
        "请按下面这个 JSON 结构输出，键名必须完全一致：\n\
         {\n\
         \x20 \"age\": \"年龄，如 38 岁，没有就留空\",\n\
         \x20 \"identity\": \"身份标签，多个用、分隔\",\n\
         \x20 \"experience\": \"核心经历，2-3 句\",\n\
         \x20 \"traits\": [\"性格关键词\", \"3-5 个\"],\n\
         \x20 \"catchphrases\": \"口头禅\",\n\
         \x20 \"expressions\": \"常用表达习惯\",\n\
         \x20 \"audience\": \"目标受众是谁\",\n\
         \x20 \"painPoints\": \"受众的痛点\",\n\
         \x20 \"taboos\": \"禁忌话题\",\n\
         \x20 \"viewpoints\": \"代表观点\",\n\
         \x20 \"visualTone\": \"偏好的色调\",\n\
         \x20 \"visualStyle\": \"偏好的画面风格\",\n\
         \x20 \"visualScene\": \"偏好的场景\",\n\
         \x20 \"sourceNote\": \"一句话说明这份档案提取自什么素材\"\n\
         }\n\n文案：\n\"\"\"\n",
    );
    prompt.push_str(source.trim());
    prompt.push_str("\n\"\"\"");

    prompt
}

/// 解析模型返回的 JSON。模型偶尔会加代码块或前后寒暄，这里做容错。
pub fn parse_extraction(content: &str) -> Result<PersonaFields, AiError> {
    let cleaned = strip_code_fence(content);
    let start = cleaned
        .find('{')
        .ok_or_else(|| AiError::internal("模型没有返回 JSON 结构，请重试或换一个模型"))?;
    let end = cleaned
        .rfind('}')
        .filter(|end| *end > start)
        .ok_or_else(|| AiError::internal("模型返回的 JSON 不完整，请重试"))?;

    let slice = &cleaned[start..=end];
    let value: Value = serde_json::from_str(slice)
        .map_err(|error| AiError::internal(format!("解析人物特征失败：{error}")))?;

    serde_json::from_value(value)
        .map_err(|error| AiError::internal(format!("人物特征字段不匹配：{error}")))
}

fn strip_code_fence(content: &str) -> String {
    let trimmed = content.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    trimmed
        .lines()
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n")
}
