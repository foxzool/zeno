use anyhow::{anyhow, Result};
use pulldown_cmark::{html, Event, Options, Parser};
use serde::{Deserialize, Serialize};

use crate::models::{Frontmatter, Note};
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
    pub frontmatter: Option<Frontmatter>,
    /// Markdown内容（不包含前言）
    pub content: String,
    /// 渲染后的HTML
    pub html: String,
    /// 提取的链接
    pub links: Vec<Link>,
    /// 提取的标题
    pub headings: Vec<Heading>,
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

    /// 解析Markdown文件
    pub fn parse_file(&self, path: &PathBuf) -> Result<Note> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content, path.clone())
    }

    /// 解析Markdown内容
    pub fn parse(&self, content: &str, path: PathBuf) -> Result<Note> {
        let parse_result = self.parse_content(content)?;
        
        let mut note = Note::new(path, content.to_string());
        
        // 如果有前言数据，更新笔记的前言
        if let Some(frontmatter) = parse_result.frontmatter {
            note.frontmatter = frontmatter;
        }
        
        // 如果前言中有标题，使用前言中的标题
        if let Some(title) = &note.frontmatter.title {
            note.title = title.clone();
        }

        Ok(note)
    }

    /// 解析内容并返回详细结果
    pub fn parse_content(&self, content: &str) -> Result<ParseResult> {
        let (frontmatter, markdown_content) = self.extract_frontmatter(content)?;
        
        let parser = Parser::new_ext(&markdown_content, self.options);
        let events: Vec<Event> = parser.collect();
        
        // 渲染HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.iter().cloned());
        
        // 提取链接和标题
        let links = self.extract_links(&events);
        let headings = self.extract_headings(&events);

        Ok(ParseResult {
            frontmatter,
            content: markdown_content,
            html: html_output,
            links,
            headings,
        })
    }

    /// 提取前言数据
    fn extract_frontmatter(&self, content: &str) -> Result<(Option<Frontmatter>, String)> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() || !lines[0].trim().starts_with("---") {
            return Ok((None, content.to_string()));
        }

        // 查找前言结束位置
        let mut end_index = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                end_index = Some(i);
                break;
            }
        }

        let Some(end_index) = end_index else {
            return Err(anyhow!("前言格式错误：缺少结束标记"));
        };

        // 提取前言YAML
        let yaml_lines = &lines[1..end_index];
        let yaml_content = yaml_lines.join("\n");
        
        // 解析前言
        let frontmatter = if yaml_content.trim().is_empty() {
            None
        } else {
            Some(serde_yaml::from_str(&yaml_content)?)
        };

        // 提取Markdown内容
        let content_lines = &lines[end_index + 1..];
        let markdown_content = content_lines.join("\n");

        Ok((frontmatter, markdown_content))
    }

    /// 提取链接
    fn extract_links(&self, events: &[Event]) -> Vec<Link> {
        let mut links = Vec::new();
        let mut current_link_url = None;
        let mut current_link_text = String::new();
        let mut in_link = false;

        for event in events {
            match event {
                Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) => {
                    in_link = true;
                    current_link_url = Some(dest_url.to_string());
                    current_link_text.clear();
                }
                Event::End(pulldown_cmark::TagEnd::Link) => {
                    if let Some(url) = current_link_url.take() {
                        let link_type = if url.starts_with("http") {
                            LinkType::External
                        } else {
                            LinkType::Internal
                        };
                        
                        links.push(Link {
                            text: current_link_text.clone(),
                            url,
                            link_type,
                        });
                    }
                    in_link = false;
                    current_link_text.clear();
                }
                Event::Start(pulldown_cmark::Tag::Image { dest_url, .. }) => {
                    links.push(Link {
                        text: String::new(),
                        url: dest_url.to_string(),
                        link_type: LinkType::Image,
                    });
                }
                Event::Text(text) if in_link => {
                    current_link_text.push_str(text);
                }
                _ => {}
            }
        }

        links
    }

    /// 提取标题
    fn extract_headings(&self, events: &[Event]) -> Vec<Heading> {
        let mut headings = Vec::new();
        let mut current_heading_level = None;
        let mut current_heading_text = String::new();
        let mut in_heading = false;

        for event in events {
            match event {
                Event::Start(pulldown_cmark::Tag::Heading { level, .. }) => {
                    in_heading = true;
                    current_heading_level = Some(*level as u8);
                    current_heading_text.clear();
                }
                Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                    if let Some(level) = current_heading_level.take() {
                        let id = Self::generate_heading_id(&current_heading_text);
                        headings.push(Heading {
                            level,
                            text: current_heading_text.clone(),
                            id,
                        });
                    }
                    in_heading = false;
                    current_heading_text.clear();
                }
                Event::Text(text) if in_heading => {
                    current_heading_text.push_str(text);
                }
                _ => {}
            }
        }

        headings
    }

    /// 生成标题锚点ID
    fn generate_heading_id(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

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
    fn test_frontmatter_parsing() {
        let parser = MarkdownParser::new();
        let content = r#"---
title: "前言标题"
tags: ["rust", "markdown"]
---

# 内容标题

这是内容。"#;
        
        let result = parser.parse_content(content).unwrap();
        
        assert!(result.frontmatter.is_some());
        let frontmatter = result.frontmatter.unwrap();
        assert_eq!(frontmatter.title, Some("前言标题".to_string()));
        assert_eq!(frontmatter.tags, vec!["rust", "markdown"]);
    }

    #[test]
    fn test_link_extraction() {
        let parser = MarkdownParser::new();
        let content = "[内部链接](other-note.md) 和 [外部链接](https://example.com)";
        
        let result = parser.parse_content(content).unwrap();
        
        assert_eq!(result.links.len(), 2);
        assert_eq!(result.links[0].text, "内部链接");
        assert!(matches!(result.links[0].link_type, LinkType::Internal));
        assert_eq!(result.links[1].text, "外部链接");
        assert!(matches!(result.links[1].link_type, LinkType::External));
    }

    #[test]
    fn test_heading_extraction() {
        let parser = MarkdownParser::new();
        let content = "# 一级标题\n## 二级标题\n### 三级标题";
        
        let result = parser.parse_content(content).unwrap();
        
        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].level, 1);
        assert_eq!(result.headings[0].text, "一级标题");
        assert_eq!(result.headings[1].level, 2);
        assert_eq!(result.headings[1].text, "二级标题");
    }

    #[test]
    fn test_file_parsing() {
        let parser = MarkdownParser::new();
        
        // 创建临时文件
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        let content = "# 文件测试\n\n这是从文件读取的内容。";
        fs::write(&file_path, content).unwrap();
        
        let note = parser.parse_file(&file_path).unwrap();
        assert_eq!(note.title, "文件测试");
        assert_eq!(note.path, file_path);
    }
}