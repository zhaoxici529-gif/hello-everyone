//! AI 写作。
//!
//! 三件事：
//! 1. 按文体组装提示词（人物小传进 system，素材进 user）；
//! 2. 一次生成 3 个不同切入角度的版本；
//! 3. 同一次返回配图建议，格式与产品要求一致（位置 / 关键词 / 风格 / 比例）。

pub mod commands;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::services::ai::AiError;

/// 文体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Xiaohongshu,
    Wechat,
    Script,
    Moments,
}

pub const ALL_GENRES: [Genre; 4] = [
    Genre::Xiaohongshu,
    Genre::Wechat,
    Genre::Script,
    Genre::Moments,
];

impl Genre {
    pub fn id(self) -> &'static str {
        match self {
            Genre::Xiaohongshu => "xiaohongshu",
            Genre::Wechat => "wechat",
            Genre::Script => "script",
            Genre::Moments => "moments",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Genre::Xiaohongshu => "小红书",
            Genre::Wechat => "公众号",
            Genre::Script => "短视频脚本",
            Genre::Moments => "朋友圈",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        ALL_GENRES
            .into_iter()
            .find(|genre| genre.id() == id.trim().to_ascii_lowercase())
    }

    /// 每个文体的写作规范，直接拼进提示词
    pub fn style_guide(self) -> &'static str {
        match self {
            Genre::Xiaohongshu => {
                "【小红书要求】\n\
                 - 标题：不超过 20 字，带情绪或反差，让人想点开；可以带 1 个 emoji\n\
                 - 正文：400-600 字，第一人称「我」，口语化\n\
                 - 分段：每段 1-3 行，段与段之间空一行，读起来不累\n\
                 - emoji：每段最多 1-2 个，用来分点或强调，不要堆砌\n\
                 - 结尾：单独一行放 5-8 个话题标签，以 # 开头\n\
                 - 全篇要有具体的场景和细节，不要空泛的大道理"
            }
            Genre::Wechat => {
                "【公众号要求】\n\
                 - 标题：不超过 30 字，清晰说明读者能得到什么\n\
                 - 正文：800-1200 字，有 2-4 个小标题，逻辑递进\n\
                 - 语气：比小红书更稳，但不端着，多用短句\n\
                 - 结尾：给一个可以立刻做的小动作，并引导留言\n\
                 - 不要用 emoji 堆砌，正文里最多点缀 1-2 个"
            }
            Genre::Script => {
                "【短视频脚本要求】\n\
                 - 总时长控制在 60 秒左右\n\
                 - 开头 3 秒必须抓住人：一句反常识的话或一个具体场景\n\
                 - 按分镜写，每段标注【画面】【口播】【字幕】\n\
                 - 口播要短句、口语化，像跟人面对面说话\n\
                 - 结尾给一个互动引导（提问或邀请留言）"
            }
            Genre::Moments => {
                "【朋友圈要求】\n\
                 - 150 字以内，像真人随手发的，不要有营销味\n\
                 - 第一句就是钩子，不要铺垫\n\
                 - 可以有 1-2 个 emoji，不要放话题标签\n\
                 - 留一点余味，让人想评论"
            }
        }
    }
}

/// 生成出来的一个版本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteVersion {
    /// 切入角度，例如「痛点切入」
    pub angle: String,
    pub title: String,
    pub content: String,
}

/// 配图建议。对模型用产品规定的中文键名，对前端输出英文键名。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSuggestion {
    #[serde(rename(deserialize = "位置"), alias = "position", default)]
    pub position: String,
    #[serde(rename(deserialize = "关键词"), alias = "keywords", default)]
    pub keywords: String,
    #[serde(rename(deserialize = "风格"), alias = "style", default)]
    pub style: String,
    #[serde(rename(deserialize = "比例"), alias = "ratio", default)]
    pub ratio: String,
}

/// 一次生成的完整结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftPayload {
    pub versions: Vec<WriteVersion>,
    #[serde(rename(deserialize = "配图建议"), alias = "imageKeywords", default)]
    pub image_keywords: Vec<ImageSuggestion>,
}

/* --------------------------------- 提示词 --------------------------------- */

pub const BRAND_PROMPT: &str = "你是「心身同调」领域的内容创作助手，服务对象是关注睡眠、情绪、亲子关系与学习动力的普通学员。\n\
说话要像真人：口语化、有画面感、不说教。\n\
不做医疗承诺，不夸大疗效；涉及疾病要建议对方就医。\n\
不要输出你的思考过程，也不要说「以下是」这类客套话。";

pub fn build_system_prompt(genre: Genre, persona_block: Option<&str>) -> String {
    let mut system = String::from(BRAND_PROMPT);
    system.push_str("\n\n");
    system.push_str(genre.style_guide());

    if let Some(block) = persona_block.filter(|text| !text.trim().is_empty()) {
        system.push_str("\n\n【必须遵守人物档案】\n");
        system.push_str(block.trim());
        system.push_str("\n请严格用这个人的语气、立场和禁忌来写，让读者一眼能认出是他/她。");
    }

    system
}

pub fn build_user_prompt(topic: &str, extra: Option<&str>, material_blocks: &[String]) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!("主题：{}\n", topic.trim()));

    if let Some(extra) = extra.map(str::trim).filter(|text| !text.is_empty()) {
        prompt.push_str(&format!("补充想法（务必体现）：{extra}\n"));
    }

    if !material_blocks.is_empty() {
        prompt.push_str("\n可参考的素材：\n");
        for block in material_blocks {
            prompt.push_str(block);
            prompt.push('\n');
        }
    }

    prompt.push_str(
        "\n请输出 JSON，键名必须完全一致：\n\
         {\n\
         \x20 \"versions\": [\n\
         \x20   { \"angle\": \"切入角度，例如 痛点切入\", \"title\": \"标题\", \"content\": \"正文\" },\n\
         \x20   { \"angle\": \"故事切入\", \"title\": \"标题\", \"content\": \"正文\" },\n\
         \x20   { \"angle\": \"金句切入\", \"title\": \"标题\", \"content\": \"正文\" }\n\
         \x20 ],\n\
         \x20 \"配图建议\": [\n\
         \x20   { \"位置\": \"封面\", \"关键词\": \"画面描述，逗号分隔，要具体\", \"风格\": \"治愈系\", \"比例\": \"3:4\" }\n\
         \x20 ]\n\
         }\n\
         要求：3 个版本必须是明显不同的切入角度，不要只是换词；\n\
         配图建议给 3-4 条，位置分别覆盖封面、正文插图、结尾引导图；\n\
         比例从 3:4 / 1:1 / 16:9 里选。",
    );

    prompt
}

pub fn build_revise_system_prompt(genre: Option<Genre>, persona_block: Option<&str>) -> String {
    let mut system = String::from(BRAND_PROMPT);
    if let Some(genre) = genre {
        system.push_str("\n\n");
        system.push_str(genre.style_guide());
    }
    if let Some(block) = persona_block.filter(|text| !text.trim().is_empty()) {
        system.push_str("\n\n【必须遵守人物档案】\n");
        system.push_str(block.trim());
    }

    system.push_str(
        "\n\n用户会给你一版文案和修改要求。请直接输出修改后的完整文案，\
         不要解释改了什么，不要用 markdown 代码块包裹。",
    );
    system
}

pub fn build_revise_prompt(content: &str, instruction: &str) -> String {
    format!(
        "这是当前文案：\n\"\"\"\n{}\n\"\"\"\n\n修改要求：{}\n\n请输出修改后的完整文案。",
        content.trim(),
        instruction.trim()
    )
}

/* --------------------------------- 解析 --------------------------------- */

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

fn json_slice(content: &str) -> Result<&str, AiError> {
    let start = content
        .find('{')
        .ok_or_else(|| AiError::internal("模型没有返回 JSON 结构，请重试或换一个模型"))?;
    let end = content
        .rfind('}')
        .filter(|end| *end > start)
        .ok_or_else(|| AiError::internal("模型返回的 JSON 不完整，请重试"))?;
    Ok(&content[start..=end])
}

pub fn parse_draft(raw: &str) -> Result<DraftPayload, AiError> {
    let cleaned = strip_code_fence(raw);
    let slice = json_slice(&cleaned)?;

    let payload: DraftPayload = serde_json::from_str(slice)
        .map_err(|error| AiError::internal(format!("解析生成结果失败：{error}")))?;

    if payload.versions.is_empty() {
        return Err(AiError::internal("模型没有给出任何版本，请重试"));
    }

    Ok(payload)
}

/// 修改结果就是纯文本，去掉可能的代码块包裹
pub fn parse_revision(raw: &str) -> Result<String, AiError> {
    let cleaned = strip_code_fence(raw).trim().to_string();
    if cleaned.is_empty() {
        return Err(AiError::internal("模型返回了空内容，请重试"));
    }
    Ok(cleaned)
}

/// 生成建议的图片提示词，供图片生成模块直接使用（图片模块接入前先在测试里保证行为）
#[allow(dead_code)]
pub fn suggestion_to_prompt(suggestion: &ImageSuggestion, content_title: &str) -> String {
    let mut parts = vec![suggestion.keywords.trim().to_string()];
    if !suggestion.style.trim().is_empty() {
        parts.push(format!("{}风格", suggestion.style.trim()));
    }
    if !content_title.trim().is_empty() {
        parts.push(format!("主题：{}", content_title.trim()));
    }
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("，")
}

/* --------------------------------- 导出 --------------------------------- */

/// 富文本编辑器存的是 HTML，导出时要转成纯文本或 Markdown。
fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// 压缩多余空行：连续 3 个以上换行收敛成 2 个，行尾空格去掉
fn tidy(input: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim_end().to_string();
        if trimmed.is_empty() && lines.last().map(|last: &String| last.is_empty()).unwrap_or(true) {
            continue;
        }
        lines.push(trimmed);
    }

    while lines.last().map(|last| last.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}

fn convert_html(html: &str, markdown: bool) -> String {
    let chars: Vec<char> = html.chars().collect();
    let mut output = String::new();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] != '<' {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        // 读出一个完整标签
        let mut tag = String::new();
        index += 1;
        while index < chars.len() && chars[index] != '>' {
            tag.push(chars[index]);
            index += 1;
        }
        index += 1;

        let closing = tag.starts_with('/');
        let name = tag
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_ascii_lowercase();

        match name.as_str() {
            "br" => output.push('\n'),
            "p" | "div" | "section" | "ul" | "ol" | "blockquote" => {
                if closing {
                    output.push('\n');
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if markdown && !closing {
                    let level = name[1..].parse::<usize>().unwrap_or(1).clamp(1, 3);
                    output.push_str(&"#".repeat(level));
                    output.push(' ');
                }
                if closing {
                    output.push('\n');
                }
            }
            "li" => {
                if closing {
                    output.push('\n');
                } else {
                    output.push_str(if markdown { "- " } else { "· " });
                }
            }
            "strong" | "b" => {
                if markdown {
                    output.push_str("**");
                }
            }
            "em" | "i" => {
                if markdown {
                    output.push('*');
                }
            }
            _ => {}
        }
    }

    tidy(&decode_entities(&output))
}

pub fn html_to_text(html: &str) -> String {
    convert_html(html, false)
}

pub fn html_to_markdown(html: &str) -> String {
    convert_html(html, true)
}

/// 导出用的文件名（不带扩展名）
pub fn export_file_stem(title: &str, id: i64) -> String {
    let cleaned: String = title
        .chars()
        .map(|character| match character {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();

    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        format!("文案-{id}")
    } else {
        trimmed.chars().take(60).collect()
    }
}
