use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Wiki链接结构，支持多种链接格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WikiLink {
    /// 原始链接文本，如 "[[target|alias]]"
    pub raw: String,
    /// 目标文件路径或标题
    pub target: String,
    /// 显示别名，如果有的话
    pub alias: Option<String>,
    /// 锚点，如 "#section"
    pub anchor: Option<String>,
    /// 是否为嵌入链接（以 ! 开头）
    pub is_embed: bool,
    /// 在文档中的位置范围
    pub range: Range<usize>,
    /// 链接所在的行号
    pub line_number: usize,
}

impl WikiLink {
    /// 获取显示文本（别名或目标）
    pub fn display_text(&self) -> &str {
        self.alias.as_ref().unwrap_or(&self.target)
    }
    
    /// 获取完整目标（包含锚点）
    pub fn full_target(&self) -> String {
        if let Some(anchor) = &self.anchor {
            format!("{}#{}", self.target, anchor)
        } else {
            self.target.clone()
        }
    }
    
    /// 检查是否为有效链接
    pub fn is_valid(&self) -> bool {
        !self.target.trim().is_empty()
    }
}

/// 反向链接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklinkInfo {
    /// 源笔记ID
    pub source_note_id: String,
    /// 源笔记标题
    pub source_note_title: String,
    /// 源笔记路径
    pub source_note_path: String,
    /// 链接上下文（链接前后的文本）
    pub context: String,
    /// 链接在源笔记中的行号
    pub line_number: usize,
    /// 链接类型
    pub link_type: LinkType,
    /// 链接出现次数
    pub occurrence_count: usize,
}

/// 链接类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinkType {
    /// 普通Wiki链接
    Wiki,
    /// 嵌入链接
    Embed,
    /// Markdown链接
    Markdown,
    /// 标签引用
    Tag,
}

/// 链接解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkParseResult {
    /// 解析到的所有链接
    pub links: Vec<WikiLink>,
    /// 解析错误（如果有）
    pub errors: Vec<LinkParseError>,
    /// 解析统计信息
    pub stats: LinkStats,
}

/// 链接解析错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkParseError {
    /// 错误位置
    pub position: Range<usize>,
    /// 错误信息
    pub message: String,
    /// 原始文本
    pub raw_text: String,
}

/// 链接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStats {
    /// Wiki链接数量
    pub wiki_links: usize,
    /// 嵌入链接数量
    pub embed_links: usize,
    /// Markdown链接数量
    pub markdown_links: usize,
    /// 断链数量
    pub broken_links: usize,
}

impl Default for LinkStats {
    fn default() -> Self {
        Self {
            wiki_links: 0,
            embed_links: 0,
            markdown_links: 0,
            broken_links: 0,
        }
    }
}

/// 相似笔记信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarNote {
    /// 笔记ID
    pub note_id: String,
    /// 笔记标题
    pub title: String,
    /// 笔记路径
    pub path: String,
    /// 相似度评分 (0.0 - 1.0)
    pub similarity_score: f64,
    /// 共同链接
    pub common_links: Vec<String>,
    /// 相似性原因
    pub similarity_reason: String,
}

/// 断链信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenLink {
    /// 源笔记ID
    pub source_note_id: String,
    /// 源笔记路径
    pub source_note_path: String,
    /// 断链信息
    pub link: WikiLink,
    /// 建议的修复方案
    pub suggestions: Vec<String>,
}