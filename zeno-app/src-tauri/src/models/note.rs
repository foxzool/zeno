use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Option<Frontmatter>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub checksum: String,
    pub word_count: usize,
    pub reading_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub status: NoteStatus,
    pub publish: Option<PublishConfig>,
    pub description: Option<String>,
    pub extra: std::collections::HashMap<String, serde_json::Value>,
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
            description: None,
            extra: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoteStatus {
    Draft,
    Published,
    Archived,
}

impl Default for NoteStatus {
    fn default() -> Self {
        NoteStatus::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub platforms: Vec<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub auto_publish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

impl Note {
    pub fn new(path: PathBuf, title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            path,
            title,
            content: content.clone(),
            frontmatter: None,
            created_at: now,
            modified_at: now,
            checksum: calculate_checksum(&content),
            word_count: count_words(&content),
            reading_time: calculate_reading_time(&content),
        }
    }
}

fn calculate_checksum(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn count_words(content: &str) -> usize {
    content.split_whitespace().count()
}

fn calculate_reading_time(content: &str) -> u32 {
    let words = count_words(content);
    ((words as f32 / 200.0).ceil() as u32).max(1)
}