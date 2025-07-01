use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::note::NoteStatus;

/// 前言配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frontmatter {
    /// 标题
    #[serde(default)]
    pub title: Option<String>,
    /// 日期
    #[serde(default)]
    pub date: Option<NaiveDate>,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 分类
    #[serde(default)]
    pub categories: Vec<String>,
    /// 状态
    #[serde(default)]
    pub status: NoteStatus,
    /// 发布配置
    #[serde(default)]
    pub publish: Option<PublishConfig>,
    /// 自定义字段
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// 发布配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishConfig {
    /// 是否启用发布
    pub enabled: bool,
    /// 发布平台
    pub platforms: Vec<String>,
    /// 发布路径
    pub path: Option<String>,
    /// 发布模板
    pub template: Option<String>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            title: None,
            date: None,
            tags: Vec::new(),
            categories: Vec::new(),
            status: NoteStatus::default(),
            publish: None,
            custom: HashMap::new(),
        }
    }
}

impl Frontmatter {
    /// 从 YAML 字符串解析前言
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// 转换为 YAML 字符串
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 添加分类
    pub fn add_category(&mut self, category: String) {
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
    }

    /// 移除分类
    pub fn remove_category(&mut self, category: &str) {
        self.categories.retain(|c| c != category);
    }

    /// 设置自定义字段
    pub fn set_custom_field(&mut self, key: String, value: serde_json::Value) {
        self.custom.insert(key, value);
    }

    /// 获取自定义字段
    pub fn get_custom_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(key)
    }
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            platforms: Vec::new(),
            path: None,
            template: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_default() {
        let frontmatter = Frontmatter::default();
        assert_eq!(frontmatter.status, NoteStatus::Draft);
        assert!(frontmatter.tags.is_empty());
        assert!(frontmatter.categories.is_empty());
    }

    #[test]
    fn test_tag_operations() {
        let mut frontmatter = Frontmatter::default();
        
        frontmatter.add_tag("rust".to_string());
        frontmatter.add_tag("programming".to_string());
        frontmatter.add_tag("rust".to_string()); // 重复添加
        
        assert_eq!(frontmatter.tags.len(), 2);
        assert!(frontmatter.tags.contains(&"rust".to_string()));
        
        frontmatter.remove_tag("rust");
        assert_eq!(frontmatter.tags.len(), 1);
        assert!(!frontmatter.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_custom_fields() {
        let mut frontmatter = Frontmatter::default();
        
        frontmatter.set_custom_field(
            "priority".to_string(),
            serde_json::Value::String("high".to_string()),
        );
        
        assert_eq!(
            frontmatter.get_custom_field("priority"),
            Some(&serde_json::Value::String("high".to_string()))
        );
        
        assert_eq!(frontmatter.get_custom_field("nonexistent"), None);
    }
}