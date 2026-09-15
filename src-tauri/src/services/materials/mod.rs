//! 素材管理。
//!
//! 文件本体放在应用数据目录的「素材库」下（按年月分文件夹），
//! 数据库 materials 表只存路径、缩略图、标签、描述这些元数据。

pub mod commands;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::services::ai::AiError;

pub const LIBRARY_DIR: &str = "素材库";
/// 缩略图长边上限
pub const THUMB_MAX_EDGE: u32 = 480;

/// 支持的文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    Image,
    Video,
    Audio,
    Text,
}

pub const ALL_TYPES: [MaterialType; 4] = [
    MaterialType::Image,
    MaterialType::Video,
    MaterialType::Audio,
    MaterialType::Text,
];

impl MaterialType {
    pub fn id(self) -> &'static str {
        match self {
            MaterialType::Image => "image",
            MaterialType::Video => "video",
            MaterialType::Audio => "audio",
            MaterialType::Text => "text",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MaterialType::Image => "图片",
            MaterialType::Video => "视频",
            MaterialType::Audio => "音频",
            MaterialType::Text => "文本",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            MaterialType::Image => &["jpg", "jpeg", "png", "webp", "gif", "bmp"],
            MaterialType::Video => &["mp4", "mov", "m4v", "webm"],
            MaterialType::Audio => &["mp3", "wav", "m4a", "aac", "ogg"],
            MaterialType::Text => &["txt", "md", "markdown", "csv", "json"],
        }
    }

    /// 图片和视频才需要缩略图
    pub fn is_visual(self) -> bool {
        matches!(self, MaterialType::Image | MaterialType::Video)
    }

    pub fn from_id(id: &str) -> Option<Self> {
        ALL_TYPES.into_iter().find(|kind| kind.id() == id.trim())
    }

    /// 按扩展名判断类型，不支持的返回 None
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())?;

        ALL_TYPES
            .into_iter()
            .find(|kind| kind.extensions().contains(&extension.as_str()))
    }
}

/// 库里每一条素材的元数据
#[derive(Debug, Clone)]
pub struct MaterialRecord {
    /// 数据库主键，供调用方关联用
    #[allow(dead_code)]
    pub id: i64,
    pub name: String,
    pub kind: MaterialType,
    pub file_path: String,
    pub thumb_path: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

impl MaterialRecord {
    pub fn from_row(row: &Value) -> Result<Self, AiError> {
        let raw_type = row.get("type").and_then(Value::as_str).unwrap_or("text");
        let kind = MaterialType::from_id(raw_type)
            .ok_or_else(|| AiError::internal(format!("未知的素材类型：{raw_type}")))?;

        Ok(Self {
            id: row.get("id").and_then(Value::as_i64).unwrap_or_default(),
            name: row
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("未命名")
                .to_string(),
            kind,
            file_path: row
                .get("file_path")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            thumb_path: row
                .get("thumb_path")
                .and_then(Value::as_str)
                .map(str::to_string),
            tags: split_tags(row.get("tags").and_then(Value::as_str).unwrap_or_default()),
            description: row
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

/* ---------------------------------- 路径 ---------------------------------- */

pub fn library_root(data_dir: &Path) -> PathBuf {
    data_dir.join(LIBRARY_DIR)
}

/// 按年月分文件夹，避免一个目录堆几千个文件
pub fn month_folder(root: &Path, year_month: &str) -> PathBuf {
    root.join(year_month)
}

pub fn thumb_path_for(file: &Path) -> PathBuf {
    let stem = file
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "material".to_string());
    file.with_file_name(format!("{stem}.thumb.jpg"))
}

/// 去掉文件名里的非法字符并限长
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| match character {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            control if control.is_control() => '_',
            other => other,
        })
        .collect();

    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        return "material".to_string();
    }

    // 文件名（含扩展名）控制在 120 字符以内
    trimmed.chars().take(120).collect()
}

/// 加时间戳前缀，避免同名覆盖
pub fn unique_file_name(original: &str, stamp_ms: u128) -> String {
    format!("{stamp_ms}_{}", sanitize_file_name(original))
}

/// 只允许删除素材库目录内的文件，防止误删用户其它文件
pub fn ensure_inside_library(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("素材库目录不可用：{error}"))?;
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|error| format!("文件不可访问：{error}"))?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err("拒绝对素材库以外的文件做操作".to_string());
    }
    Ok(canonical_candidate)
}

/* ---------------------------------- 缩略图 --------------------------------- */

/// 用 image crate 生成缩略图，长边不超过 max_edge。
pub fn make_thumbnail(source: &Path, target: &Path, max_edge: u32) -> Result<(), String> {
    let reader = image::ImageReader::open(source)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?;

    let decoded = reader.decode().map_err(|error| error.to_string())?;
    let thumbnail = decoded.thumbnail(max_edge, max_edge);

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    thumbnail.save(target).map_err(|error| error.to_string())
}

/* ---------------------------------- 标签 ---------------------------------- */

/// 标签在库里是逗号分隔的字符串
pub fn split_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|tag| tag.trim().replace(',', ""))
        .filter(|tag| !tag.is_empty())
        .collect()
}

pub fn join_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|tag| tag.trim().replace(',', ""))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

/* ------------------------------- 供其它模块引用 ------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// AI 写作引用素材
    Writing,
    /// 图片生成拿参考图
    Image,
    /// 视频剪辑拿素材文件
    Video,
}

impl Purpose {
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "writing" => Some(Purpose::Writing),
            "image" => Some(Purpose::Image),
            "video" => Some(Purpose::Video),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Purpose::Writing => "writing",
            Purpose::Image => "image",
            Purpose::Video => "video",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Purpose::Writing => "AI 写作",
            Purpose::Image => "图片生成",
            Purpose::Video => "视频剪辑",
        }
    }
}

impl MaterialRecord {
    /// 组装成给模型看的素材块。图片生成场景会明确给出文件路径，方便后续接入。
    pub fn to_prompt_block(&self, purpose: Purpose) -> String {
        let mut lines = vec![format!("【素材：{}（{}）】", self.name, self.kind.label())];

        if !self.tags.is_empty() {
            lines.push(format!("- 标签：{}", self.tags.join("、")));
        }
        if let Some(description) = self
            .description
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            lines.push(format!("- 描述：{description}"));
        }
        if !self.file_path.is_empty() {
            lines.push(format!("- 文件：{}", self.file_path));
        }

        match purpose {
            Purpose::Writing => lines.push("- 用法：可以参考或引用这条素材的内容".to_string()),
            Purpose::Image => lines.push("- 用法：作为画面参考，保持色调与风格一致".to_string()),
            Purpose::Video => lines.push("- 用法：作为剪辑素材，按时间线拼进成片".to_string()),
        }

        lines.join("\n")
    }
}
