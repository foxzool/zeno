use crate::models::link::{WikiLink, LinkParseResult, LinkParseError, LinkStats, LinkType};
use regex::Regex;
use std::ops::Range;

/// Wiki链接解析器
pub struct LinkParser {
    /// Wiki链接正则表达式: [[target|alias]] 或 ![[target]]
    wiki_link_regex: Regex,
    /// Markdown链接正则表达式: [text](url)
    markdown_link_regex: Regex,
    /// 标签正则表达式: #tag
    tag_regex: Regex,
}

impl LinkParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            // 匹配 [[target]] 或 [[target|alias]] 或 ![[target]]
            wiki_link_regex: Regex::new(r"(!?)\[\[([^\]]+?)(?:\|([^\]]+?))?\]\]")?,
            // 匹配 [text](url)
            markdown_link_regex: Regex::new(r"\[([^\]]+?)\]\(([^)]+?)\)")?,
            // 匹配 #tag (但不匹配 # 标题)
            tag_regex: Regex::new(r"(?:^|[^#\w])#([a-zA-Z\u4e00-\u9fff][a-zA-Z0-9\u4e00-\u9fff_-]*)")?,
        })
    }

    /// 解析文本中的所有链接
    pub fn parse_links(&self, content: &str) -> LinkParseResult {
        let mut links = Vec::new();
        let mut errors = Vec::new();
        let mut stats = LinkStats::default();

        // 解析Wiki链接
        self.parse_wiki_links(content, &mut links, &mut errors, &mut stats);
        
        // 解析Markdown链接
        self.parse_markdown_links(content, &mut links, &mut errors, &mut stats);

        // 按位置排序
        links.sort_by_key(|link| link.range.start);

        LinkParseResult {
            links,
            errors,
            stats,
        }
    }

    /// 解析Wiki风格的链接
    fn parse_wiki_links(
        &self,
        content: &str,
        links: &mut Vec<WikiLink>,
        errors: &mut Vec<LinkParseError>,
        stats: &mut LinkStats,
    ) {
        for cap in self.wiki_link_regex.captures_iter(content) {
            match self.parse_single_wiki_link(content, &cap) {
                Ok(link) => {
                    if link.is_embed {
                        stats.embed_links += 1;
                    } else {
                        stats.wiki_links += 1;
                    }
                    links.push(link);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }
    }

    /// 解析单个Wiki链接
    fn parse_single_wiki_link(
        &self,
        content: &str,
        cap: &regex::Captures,
    ) -> Result<WikiLink, LinkParseError> {
        let full_match = cap.get(0).unwrap();
        let is_embed = cap.get(1).map_or(false, |m| m.as_str() == "!");
        let target_with_anchor = cap.get(2).unwrap().as_str();
        let alias = cap.get(3).map(|m| m.as_str().to_string());

        // 解析目标和锚点
        let (target, anchor) = if let Some(pos) = target_with_anchor.find('#') {
            (
                target_with_anchor[..pos].to_string(),
                Some(target_with_anchor[pos + 1..].to_string()),
            )
        } else {
            (target_with_anchor.to_string(), None)
        };

        // 验证目标
        if target.trim().is_empty() {
            return Err(LinkParseError {
                position: full_match.range(),
                message: "链接目标不能为空".to_string(),
                raw_text: full_match.as_str().to_string(),
            });
        }

        // 计算行号
        let line_number = self.calculate_line_number(content, full_match.start());

        Ok(WikiLink {
            raw: full_match.as_str().to_string(),
            target: target.trim().to_string(),
            alias: alias.map(|a| a.trim().to_string()),
            anchor: anchor.map(|a| a.trim().to_string()),
            is_embed,
            range: full_match.range(),
            line_number,
        })
    }

    /// 解析Markdown链接
    fn parse_markdown_links(
        &self,
        content: &str,
        links: &mut Vec<WikiLink>,
        _errors: &mut Vec<LinkParseError>,
        stats: &mut LinkStats,
    ) {
        for cap in self.markdown_link_regex.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let text = cap.get(1).unwrap().as_str();
            let url = cap.get(2).unwrap().as_str();

            // 只处理本地文件链接，忽略HTTP链接
            if url.starts_with("http://") || url.starts_with("https://") {
                continue;
            }

            let line_number = self.calculate_line_number(content, full_match.start());

            let link = WikiLink {
                raw: full_match.as_str().to_string(),
                target: url.to_string(),
                alias: Some(text.to_string()),
                anchor: None,
                is_embed: false,
                range: full_match.range(),
                line_number,
            };

            stats.markdown_links += 1;
            links.push(link);
        }
    }

    /// 计算给定位置的行号
    fn calculate_line_number(&self, content: &str, position: usize) -> usize {
        content[..position].matches('\n').count() + 1
    }

    /// 提取链接周围的上下文
    pub fn extract_link_context(&self, content: &str, link: &WikiLink, context_size: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = link.line_number.saturating_sub(1);
        
        if line_idx >= lines.len() {
            return String::new();
        }

        let start_line = line_idx.saturating_sub(context_size);
        let end_line = std::cmp::min(line_idx + context_size + 1, lines.len());

        lines[start_line..end_line].join("\n")
    }

    /// 替换文档中的链接
    pub fn replace_link(&self, content: &str, old_link: &WikiLink, new_target: &str) -> String {
        let new_link = if let Some(alias) = &old_link.alias {
            if old_link.is_embed {
                format!("![[{}]]", new_target)
            } else {
                format!("[[{}|{}]]", new_target, alias)
            }
        } else {
            if old_link.is_embed {
                format!("![[{}]]", new_target)
            } else {
                format!("[[{}]]", new_target)
            }
        };

        let mut result = content.to_string();
        result.replace_range(old_link.range.clone(), &new_link);
        result
    }

    /// 批量替换链接
    pub fn replace_multiple_links(
        &self,
        content: &str,
        replacements: &[(WikiLink, String)],
    ) -> String {
        let mut result = content.to_string();
        
        // 按位置逆序排序，从后往前替换以避免位置偏移
        let mut sorted_replacements = replacements.to_vec();
        sorted_replacements.sort_by_key(|(link, _)| std::cmp::Reverse(link.range.start));

        for (old_link, new_target) in sorted_replacements {
            let new_link = if let Some(alias) = &old_link.alias {
                if old_link.is_embed {
                    format!("![[{}]]", new_target)
                } else {
                    format!("[[{}|{}]]", new_target, alias)
                }
            } else {
                if old_link.is_embed {
                    format!("![[{}]]", new_target)
                } else {
                    format!("[[{}]]", new_target)
                }
            };

            result.replace_range(old_link.range.clone(), &new_link);
        }

        result
    }
}

impl Default for LinkParser {
    fn default() -> Self {
        Self::new().expect("Failed to create LinkParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wiki_links() {
        let parser = LinkParser::new().unwrap();
        let content = r#"
这是一个 [[测试笔记]] 的链接。
还有一个带别名的 [[another-note|另一个笔记]] 链接。
这是嵌入链接 ![[图片.png]]。
带锚点的链接 [[笔记#章节]]。
"#;

        let result = parser.parse_links(content);
        
        assert_eq!(result.links.len(), 4);
        assert_eq!(result.stats.wiki_links, 3);
        assert_eq!(result.stats.embed_links, 1);
        
        // 测试第一个链接
        let first_link = &result.links[0];
        assert_eq!(first_link.target, "测试笔记");
        assert_eq!(first_link.alias, None);
        assert!(!first_link.is_embed);
        
        // 测试带别名的链接
        let alias_link = &result.links[1];
        assert_eq!(alias_link.target, "another-note");
        assert_eq!(alias_link.alias, Some("另一个笔记".to_string()));
        
        // 测试嵌入链接
        let embed_link = &result.links[2];
        assert_eq!(embed_link.target, "图片.png");
        assert!(embed_link.is_embed);
        
        // 测试锚点链接
        let anchor_link = &result.links[3];
        assert_eq!(anchor_link.target, "笔记");
        assert_eq!(anchor_link.anchor, Some("章节".to_string()));
    }

    #[test]
    fn test_parse_markdown_links() {
        let parser = LinkParser::new().unwrap();
        let content = "这是 [Markdown链接](./path/to/file.md) 和 [外部链接](https://example.com)。";

        let result = parser.parse_links(content);
        
        // 应该只解析本地文件链接，忽略HTTP链接
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.stats.markdown_links, 1);
        
        let link = &result.links[0];
        assert_eq!(link.target, "./path/to/file.md");
        assert_eq!(link.alias, Some("Markdown链接".to_string()));
    }

    #[test]
    fn test_link_context_extraction() {
        let parser = LinkParser::new().unwrap();
        let content = r#"第一行
第二行有一个 [[测试链接]]
第三行
第四行"#;

        let result = parser.parse_links(content);
        let link = &result.links[0];
        
        let context = parser.extract_link_context(content, link, 1);
        assert!(context.contains("第一行"));
        assert!(context.contains("第二行有一个 [[测试链接]]"));
        assert!(context.contains("第三行"));
    }
}