//! 本地工作记忆库。
//!
//! 每个模块的数据变更都会在这里留一条索引：source_type + source_id + content（文本摘要）。
//! 检索用「关键词命中 + 字符二元组相似度」排序。embedding 列已预留，
//! 接入向量模型后把 similar 换成余弦相似度即可。

pub mod commands;

#[cfg(test)]
mod tests;

use serde_json::Value;

/// 会被索引的数据来源
pub const SOURCES: [&str; 4] = ["persona", "material", "writing", "topic"];

pub fn source_label(id: &str) -> &'static str {
    match id {
        "persona" => "人物小传",
        "material" => "素材",
        "writing" => "文案",
        "topic" => "选题",
        _ => "其它",
    }
}

pub fn is_source(id: &str) -> bool {
    SOURCES.contains(&id)
}

/// 数据库表名 → 记忆来源
pub fn source_of_table(table: &str) -> Option<&'static str> {
    match table {
        "personas" => Some("persona"),
        "materials" => Some("material"),
        "writings" => Some("writing"),
        "topics" => Some("topic"),
        _ => None,
    }
}

/* --------------------------------- 摘要拼装 --------------------------------- */

fn text(row: &Value, key: &str) -> String {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// 富文本正文转纯文本并截断
pub fn plain_text(html: &str, limit: usize) -> String {
    let mut out = String::new();
    let mut inside_tag = false;
    for character in html.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                out.push(' ');
            }
            other if !inside_tag => out.push(other),
            _ => {}
        }
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");

    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(limit).collect()
}

fn push(parts: &mut Vec<String>, label: &str, value: &str) {
    if !value.trim().is_empty() {
        parts.push(format!("{label}：{}", value.trim()));
    }
}

/// 生成一条记忆的内容摘要
pub fn summarize(source_type: &str, row: &Value) -> String {
    let mut parts = Vec::new();

    match source_type {
        "persona" => {
            let name = text(row, "name");
            parts.push(format!("人物小传《{name}》"));
            let fields: Value =
                serde_json::from_str(&text(row, "fields")).unwrap_or(Value::Null);
            let field = |key: &str| {
                fields
                    .get(key)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            };
            let traits = fields
                .get("traits")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("、")
                })
                .unwrap_or_default();

            push(&mut parts, "分类", crate::services::personas::category_label(&text(row, "category")));
            push(&mut parts, "身份", &field("identity"));
            push(&mut parts, "性格", &traits);
            push(&mut parts, "口头禅", &field("catchphrases"));
            push(&mut parts, "受众", &field("audience"));
            push(&mut parts, "痛点", &field("painPoints"));
            push(&mut parts, "观点", &field("viewpoints"));
            push(&mut parts, "经历", &field("experience"));
        }
        "material" => {
            parts.push(format!("素材《{}》", text(row, "name")));
            push(&mut parts, "类型", crate::services::materials::MaterialType::from_id(&text(row, "type")).map(|kind| kind.label()).unwrap_or("未知"));
            push(&mut parts, "标签", &text(row, "tags"));
            push(&mut parts, "描述", &text(row, "description"));
            push(&mut parts, "文件", &text(row, "file_path"));
        }
        "writing" => {
            parts.push(format!("文案《{}》", text(row, "title")));
            push(&mut parts, "文体", crate::services::writing::Genre::from_id(&text(row, "genre")).map(|genre| genre.label()).unwrap_or("未分类"));
            push(&mut parts, "状态", &text(row, "status"));
            push(&mut parts, "正文", &plain_text(&text(row, "content"), 400));
        }
        "topic" => {
            parts.push(format!("选题《{}》", text(row, "title")));
            push(&mut parts, "来源", &text(row, "source"));
            push(&mut parts, "标签", &text(row, "tags"));
            push(&mut parts, "备注", &text(row, "note"));
        }
        other => parts.push(format!("{other} {}", text(row, "name"))),
    }

    parts.join("　")
}

/* --------------------------------- 相似度 --------------------------------- */

fn bigrams(input: &str) -> Vec<String> {
    let chars: Vec<char> = input
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();

    if chars.len() < 2 {
        return chars.iter().map(|c| c.to_string()).collect();
    }

    chars
        .windows(2)
        .map(|pair| pair.iter().collect::<String>())
        .collect()
}

/// 字符二元组的 Jaccard 相似度。embedding 接入前用它做近似语义匹配。
pub fn similar(left: &str, right: &str) -> f64 {
    let a = bigrams(&left.to_lowercase());
    let b = bigrams(&right.to_lowercase());
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let set_b: std::collections::HashSet<&String> = b.iter().collect();
    let overlap = a.iter().filter(|item| set_b.contains(item)).count();
    let union = a.len() + b.len() - overlap;

    overlap as f64 / union.max(1) as f64
}

/// 检索打分：关键词命中优先，其次看整体相似度
pub fn score(query: &str, content: &str) -> f64 {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return 0.0;
    }

    let haystack = content.to_lowercase();
    let hits = if haystack.contains(&needle) { 1.0 } else { 0.0 };
    hits + similar(&needle, &haystack) * 0.9
}
