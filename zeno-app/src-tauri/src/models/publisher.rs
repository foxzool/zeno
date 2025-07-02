use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// 发布配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishConfig {
    pub site_title: String,
    pub site_description: String,
    pub base_url: String,
    pub author: String,
    pub theme: String,
    pub build_drafts: bool,
    pub generate_rss: bool,
    pub output_dir: PathBuf,
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            site_title: "我的知识库".to_string(),
            site_description: "基于 Zeno 构建的个人知识库".to_string(),
            base_url: "https://example.com".to_string(),
            author: "".to_string(),
            theme: "zeno-default".to_string(),
            build_drafts: false,
            generate_rss: true,
            output_dir: PathBuf::from("site"),
        }
    }
}

/// Zola 配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZolaConfig {
    pub base_url: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub default_language: String,
    pub theme: String,
    pub compile_sass: bool,
    pub generate_rss: bool,
    pub build_search_index: bool,
    pub taxonomies: Vec<Taxonomy>,
    pub markdown: MarkdownConfig,
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Taxonomy {
    pub name: String,
    pub paginate_by: Option<usize>,
    pub paginate_path: Option<String>,
    pub rss: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkdownConfig {
    pub highlight_code: bool,
    pub highlight_theme: String,
    pub render_emoji: bool,
    pub external_links_target_blank: bool,
    pub external_links_no_follow: bool,
    pub external_links_no_referrer: bool,
    pub smart_punctuation: bool,
}

impl Default for ZolaConfig {
    fn default() -> Self {
        Self {
            base_url: "https://example.com".to_string(),
            title: "我的知识库".to_string(),
            description: "基于 Zeno 构建的个人知识库".to_string(),
            author: "".to_string(),
            default_language: "zh".to_string(),
            theme: "zeno-default".to_string(),
            compile_sass: true,
            generate_rss: true,
            build_search_index: true,
            taxonomies: vec![
                Taxonomy {
                    name: "tags".to_string(),
                    paginate_by: Some(10),
                    paginate_path: None,
                    rss: true,
                },
                Taxonomy {
                    name: "categories".to_string(),
                    paginate_by: Some(10),
                    paginate_path: None,
                    rss: true,
                },
            ],
            markdown: MarkdownConfig {
                highlight_code: true,
                highlight_theme: "base16-ocean-dark".to_string(),
                render_emoji: true,
                external_links_target_blank: true,
                external_links_no_follow: true,
                external_links_no_referrer: true,
                smart_punctuation: true,
            },
            extra: HashMap::new(),
        }
    }
}

/// 发布结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishResult {
    pub success: bool,
    pub published_files: Vec<PathBuf>,
    pub errors: Vec<PublishError>,
    pub build_output: String,
    pub site_url: Option<String>,
    pub build_time: f64, // 构建时间（秒）
    pub total_pages: usize,
    pub total_size: u64, // 字节
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishError {
    pub file_path: PathBuf,
    pub error_type: PublishErrorType,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PublishErrorType {
    FrontmatterParseError,
    ContentProcessingError,
    AssetCopyError,
    TemplateRenderError,
    BuildError,
}

/// 内容格式
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContentFormat {
    Markdown,
    Html,
    Json,
}

/// 发布状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PublishStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

/// 站点统计信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteStats {
    pub total_pages: usize,
    pub total_words: usize,
    pub total_tags: usize,
    pub total_categories: usize,
    pub last_build: Option<DateTime<Utc>>,
    pub build_time: Option<f64>,
    pub site_size: u64,
}