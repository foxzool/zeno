use crate::error::Result;
use crate::db::models::{Frontmatter, Note};
use pulldown_cmark::{html, Event, Options, Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Markdown解析器
#[derive(Debug, Clone)]
pub struct MarkdownParser {
    options: Options,
}

/// 解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// 前言数据
    pub frontmatter: Frontmatter,
    /// Markdown内容（不包含前言）
    pub content: String,
    /// 渲染后的HTML
    pub html: String,
    /// 提取的链接
    pub links: Vec<Link>,
    /// 双向链接
    pub wiki_links: Vec<WikiLink>,
    /// 提取的标题
    pub headings: Vec<Heading>,
    /// 目录
    pub toc: Vec<TocItem>,
    /// 标签
    pub tags: Vec<String>,
    /// 字数统计
    pub word_count: usize,
    /// 预估阅读时间（分钟）
    pub reading_time: usize,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 链接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// 链接文本
    pub text: String,
    /// 链接URL
    pub url: String,
    /// 链接类型
    pub link_type: LinkType,
    /// 链接标题
    pub title: Option<String>,
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
}

/// 双向链接（Wiki链接）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiLink {
    /// 目标笔记标题或路径
    pub target: String,
    /// 显示文本（如果与目标不同）
    pub display_text: Option<String>,
    /// 链接文本片段
    pub fragment: Option<String>,
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
}

/// 链接类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    /// 外部链接
    External,
    /// 内部链接（指向其他笔记）
    Internal,
    /// 图片链接
    Image,
    /// 邮箱链接
    Email,
    /// 锚点链接
    Anchor,
}

/// 标题信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    /// 标题级别 (1-6)
    pub level: u8,
    /// 标题文本
    pub text: String,
    /// 标题锚点ID
    pub id: String,
    /// 行号
    pub line: usize,
}

/// 目录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    /// 标题级别 (1-6)
    pub level: u8,
    /// 标题文本
    pub text: String,
    /// 锚点ID
    pub id: String,
    /// 子项
    pub children: Vec<TocItem>,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        Self { options }
    }

    /// 创建带有自定义选项的解析器
    pub fn with_options(options: Options) -> Self {
        Self { options }
    }

    /// 解析Markdown文件
    pub fn parse_file(&self, path: &PathBuf) -> Result<Note> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content, path.clone())
    }

    /// 解析Markdown内容并创建Note
    pub fn parse(&self, content: &str, path: PathBuf) -> Result<Note> {
        // 简化版本，先让项目能编译
        let id = uuid::Uuid::new_v4().to_string();
        let file_path = path.to_string_lossy().to_string();
        
        // 提取标题（简单实现）
        let title = self.extract_title_simple(content);
        
        let mut note = Note::new(title, file_path, content.to_string());
        note.id = id;
        
        Ok(note)
    }

    /// 简单的标题提取
    fn extract_title_simple(&self, content: &str) -> String {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("# ") {
                return line[2..].trim().to_string();
            }
        }
        "未命名".to_string()
    }

    /// 解析内容并返回详细结果
    pub fn parse_content(&self, content: &str) -> Result<ParseResult> {
        // 简化版本实现
        let frontmatter = Frontmatter::default();
        let parser = Parser::new_ext(content, self.options);
        let events: Vec<Event> = parser.collect();
        
        // 渲染HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.iter().cloned());
        
        Ok(ParseResult {
            frontmatter,
            content: content.to_string(),
            html: html_output,
            links: Vec::new(),
            wiki_links: Vec::new(),
            headings: Vec::new(),
            toc: Vec::new(),
            tags: Vec::new(),
            word_count: content.len(),
            reading_time: 1,
            metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let parser = MarkdownParser::new();
        let content = "# 测试标题\n\n这是一段测试内容。";
        let path = PathBuf::from("test.md");
        
        let note = parser.parse(content, path).unwrap();
        assert_eq!(note.title, "测试标题");
        assert!(note.content.contains("测试内容"));
    }

    #[test]
    fn test_content_parsing() {
        let parser = MarkdownParser::new();
        let content = "# 测试\n\n这是内容";
        
        let result = parser.parse_content(content).unwrap();
        assert!(!result.html.is_empty());
    }
}