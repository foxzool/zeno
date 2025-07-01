use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

/// 笔记状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    Draft,
    Published,
    Archived,
    Deleted,
}

impl Default for NoteStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for NoteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteStatus::Draft => write!(f, "draft"),
            NoteStatus::Published => write!(f, "published"),
            NoteStatus::Archived => write!(f, "archived"),
            NoteStatus::Deleted => write!(f, "deleted"),
        }
    }
}

impl std::str::FromStr for NoteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(NoteStatus::Draft),
            "published" => Ok(NoteStatus::Published),
            "archived" => Ok(NoteStatus::Archived),
            "deleted" => Ok(NoteStatus::Deleted),
            _ => Err(format!("无效的笔记状态: {}", s)),
        }
    }
}

/// 链接类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Reference,
    Embed,
    Citation,
}

impl Default for LinkType {
    fn default() -> Self {
        Self::Reference
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkType::Reference => write!(f, "reference"),
            LinkType::Embed => write!(f, "embed"),
            LinkType::Citation => write!(f, "citation"),
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reference" => Ok(LinkType::Reference),
            "embed" => Ok(LinkType::Embed),
            "citation" => Ok(LinkType::Citation),
            _ => Err(format!("无效的链接类型: {}", s)),
        }
    }
}

/// 笔记模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub content: String,
    pub html_content: Option<String>,
    pub word_count: i32,
    pub reading_time: i32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    pub status: String,
    pub frontmatter: String,
    pub file_size: i64,
    pub file_hash: String,
    pub search_vector: Option<String>,
}

impl Note {
    /// 创建新笔记
    pub fn new(title: String, file_path: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            file_path,
            content,
            html_content: None,
            word_count: 0,
            reading_time: 0,
            created_at: now,
            modified_at: now,
            indexed_at: now,
            status: NoteStatus::Draft.to_string(),
            frontmatter: "{}".to_string(),
            file_size: 0,
            file_hash: String::new(),
            search_vector: None,
        }
    }

    /// 获取笔记状态
    pub fn get_status(&self) -> Result<NoteStatus, String> {
        self.status.parse()
    }

    /// 设置笔记状态
    pub fn set_status(&mut self, status: NoteStatus) {
        self.status = status.to_string();
    }

    /// 解析 frontmatter
    pub fn get_frontmatter(&self) -> Result<Frontmatter, serde_json::Error> {
        serde_json::from_str(&self.frontmatter)
    }

    /// 设置 frontmatter
    pub fn set_frontmatter(&mut self, frontmatter: &Frontmatter) -> Result<(), serde_json::Error> {
        self.frontmatter = serde_json::to_string(frontmatter)?;
        Ok(())
    }

    /// 计算字数
    pub fn calculate_word_count(&mut self) {
        // 简单的字数统计，不包括 frontmatter
        let content_without_frontmatter = if self.content.starts_with("---") {
            if let Some(end_pos) = self.content[3..].find("---") {
                &self.content[end_pos + 6..]
            } else {
                &self.content
            }
        } else {
            &self.content
        };

        // 统计中文字符和英文单词
        let chinese_chars = content_without_frontmatter
            .chars()
            .filter(|c| c.is_alphabetic() && *c as u32 > 127)
            .count();
        
        let english_words = content_without_frontmatter
            .split_whitespace()
            .filter(|word| word.chars().any(|c| c.is_alphabetic() && c as u32 <= 127))
            .count();

        self.word_count = (chinese_chars + english_words) as i32;
    }

    /// 计算预估阅读时间（分钟）
    pub fn calculate_reading_time(&mut self) {
        // 中文阅读速度：250-300字/分钟，英文：200-250词/分钟
        // 这里使用 250 字/词 每分钟作为基准
        self.reading_time = (self.word_count as f32 / 250.0).ceil() as i32;
        if self.reading_time == 0 {
            self.reading_time = 1; // 至少1分钟
        }
    }
}

/// frontmatter 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub status: Option<NoteStatus>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub publish: Option<PublishConfig>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            title: None,
            date: None,
            tags: Vec::new(),
            categories: Vec::new(),
            status: None,
            description: None,
            author: None,
            publish: None,
            extra: HashMap::new(),
        }
    }
}

/// 发布配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub enabled: bool,
    pub platforms: Vec<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// 标签模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub usage_count: i32,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // 数据库自动分配
            name,
            color: "#6B7280".to_string(),
            description: String::new(),
            created_at: Utc::now(),
            usage_count: 0,
        }
    }
}

/// 分类模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub description: String,
    pub color: String,
    pub icon: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // 数据库自动分配
            name,
            parent_id: None,
            description: String::new(),
            color: "#6B7280".to_string(),
            icon: "folder".to_string(),
            sort_order: 0,
            created_at: Utc::now(),
        }
    }
}

/// 链接模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: i64,
    pub source_id: String,
    pub target_id: String,
    pub link_type: String,
    pub anchor_text: String,
    pub source_line: i32,
    pub source_column: i32,
    pub created_at: DateTime<Utc>,
}

impl Link {
    pub fn new(source_id: String, target_id: String, anchor_text: String) -> Self {
        Self {
            id: 0, // 数据库自动分配
            source_id,
            target_id,
            link_type: LinkType::Reference.to_string(),
            anchor_text,
            source_line: 0,
            source_column: 0,
            created_at: Utc::now(),
        }
    }

    /// 获取链接类型
    pub fn get_link_type(&self) -> Result<LinkType, String> {
        self.link_type.parse()
    }

    /// 设置链接类型
    pub fn set_link_type(&mut self, link_type: LinkType) {
        self.link_type = link_type.to_string();
    }
}

/// 笔记详情视图 (包含关联数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDetail {
    #[serde(flatten)]
    pub note: Note,
    pub tags: Vec<Tag>,
    pub categories: Vec<Category>,
    pub outbound_links: Vec<Link>,
    pub inbound_links: Vec<Link>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub note: Note,
    pub score: f32,
    pub highlights: Vec<String>,
    pub matched_fields: Vec<String>,
}

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub filters: SearchFilters,
}

/// 搜索过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub status: Option<NoteStatus>,
    pub date_range: Option<DateRange>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            categories: Vec::new(),
            status: None,
            date_range: None,
        }
    }
}

/// 日期范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 文件事件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileEvent {
    pub id: i64,
    pub file_path: String,
    pub event_type: String,
    pub old_path: Option<String>,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

/// 文件事件类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

impl std::fmt::Display for FileEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileEventType::Created => write!(f, "created"),
            FileEventType::Modified => write!(f, "modified"),
            FileEventType::Deleted => write!(f, "deleted"),
            FileEventType::Renamed => write!(f, "renamed"),
        }
    }
}

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub key: String,
    pub value: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 目录树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub category: Category,
    pub children: Vec<TreeNode>,
    pub note_count: usize,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_notes: i64,
    pub total_words: i64,
    pub total_tags: i64,
    pub total_categories: i64,
    pub total_links: i64,
    pub notes_by_status: HashMap<String, i64>,
    pub top_tags: Vec<(String, i32)>,
    pub recent_notes: Vec<Note>,
}