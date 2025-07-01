use serde::{Deserialize, Serialize};

/// 标签数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    /// 标签ID
    pub id: Option<i64>,
    /// 标签名称
    pub name: String,
    /// URL友好的标签名
    pub slug: String,
    /// 标签颜色
    pub color: Option<String>,
    /// 标签描述
    pub description: Option<String>,
    /// 使用次数
    pub usage_count: usize,
}

impl Tag {
    /// 创建新标签
    pub fn new(name: String) -> Self {
        let slug = Self::generate_slug(&name);
        Self {
            id: None,
            name,
            slug,
            color: None,
            description: None,
            usage_count: 0,
        }
    }

    /// 生成URL友好的slug
    fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
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

    /// 设置颜色
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 增加使用次数
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    /// 减少使用次数
    pub fn decrement_usage(&mut self) {
        if self.usage_count > 0 {
            self.usage_count -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Rust Programming".to_string());
        assert_eq!(tag.name, "Rust Programming");
        assert_eq!(tag.slug, "rust-programming");
        assert_eq!(tag.usage_count, 0);
        assert!(tag.color.is_none());
    }

    #[test]
    fn test_slug_generation() {
        assert_eq!(Tag::generate_slug("Hello World"), "hello-world");
        assert_eq!(Tag::generate_slug("C++"), "c");
        assert_eq!(Tag::generate_slug("Node.js"), "node-js");
        assert_eq!(Tag::generate_slug("  multiple   spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_tag_builder() {
        let tag = Tag::new("test".to_string())
            .with_color("#ff0000".to_string())
            .with_description("测试标签".to_string());
        
        assert_eq!(tag.color, Some("#ff0000".to_string()));
        assert_eq!(tag.description, Some("测试标签".to_string()));
    }

    #[test]
    fn test_usage_count() {
        let mut tag = Tag::new("test".to_string());
        
        tag.increment_usage();
        assert_eq!(tag.usage_count, 1);
        
        tag.increment_usage();
        assert_eq!(tag.usage_count, 2);
        
        tag.decrement_usage();
        assert_eq!(tag.usage_count, 1);
        
        tag.decrement_usage();
        tag.decrement_usage(); // 不应该变成负数
        assert_eq!(tag.usage_count, 0);
    }
}