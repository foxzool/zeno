use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::frontmatter::Frontmatter;

/// 笔记核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    /// 笔记唯一标识符
    pub id: Uuid,
    /// 文件路径
    pub path: PathBuf,
    /// 笔记标题
    pub title: String,
    /// 原始内容
    pub content: String,
    /// 前言数据
    pub frontmatter: Frontmatter,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 修改时间
    pub modified_at: DateTime<Utc>,
    /// 文件内容校验和
    pub checksum: String,
    /// 字数统计
    pub word_count: usize,
    /// 预计阅读时间（分钟）
    pub reading_time: usize,
}

/// 笔记状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    /// 草稿
    Draft,
    /// 已发布
    Published,
    /// 归档
    Archived,
    /// 已删除
    Deleted,
}

impl Default for NoteStatus {
    fn default() -> Self {
        NoteStatus::Draft
    }
}

impl Note {
    /// 创建新笔记
    pub fn new(path: PathBuf, content: String) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let checksum = Self::calculate_checksum(&content);
        let frontmatter = Frontmatter::default();
        let title = Self::extract_title(&content);
        let word_count = Self::count_words(&content);
        let reading_time = Self::calculate_reading_time(word_count);

        Self {
            id,
            path,
            title,
            content,
            frontmatter,
            created_at: now,
            modified_at: now,
            checksum,
            word_count,
            reading_time,
        }
    }

    /// 更新笔记内容
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.modified_at = Utc::now();
        self.checksum = Self::calculate_checksum(&self.content);
        self.title = Self::extract_title(&self.content);
        self.word_count = Self::count_words(&self.content);
        self.reading_time = Self::calculate_reading_time(self.word_count);
    }

    /// 计算内容校验和
    fn calculate_checksum(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 从内容中提取标题
    fn extract_title(content: &str) -> String {
        // 查找第一个 # 标题
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("# ") {
                return line[2..].trim().to_string();
            }
        }
        
        // 如果没有找到标题，使用"无标题"
        "无标题".to_string()
    }

    /// 计算字数
    fn count_words(content: &str) -> usize {
        content
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
            .count()
    }

    /// 计算阅读时间（假设每分钟阅读 300 字）
    fn calculate_reading_time(word_count: usize) -> usize {
        ((word_count as f64) / 300.0).ceil() as usize
    }

    /// 检查内容是否已更改
    pub fn is_content_changed(&self, new_content: &str) -> bool {
        let new_checksum = Self::calculate_checksum(new_content);
        self.checksum != new_checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let path = PathBuf::from("test.md");
        let content = "# 测试标题\n\n这是测试内容。".to_string();
        
        let note = Note::new(path.clone(), content.clone());
        
        assert_eq!(note.path, path);
        assert_eq!(note.content, content);
        assert_eq!(note.title, "测试标题");
        assert!(note.word_count > 0);
        assert!(note.reading_time > 0);
    }

    #[test]
    fn test_content_update() {
        let mut note = Note::new(
            PathBuf::from("test.md"),
            "# 原标题\n原内容".to_string(),
        );
        
        let old_checksum = note.checksum.clone();
        let old_modified = note.modified_at;
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        note.update_content("# 新标题\n新内容".to_string());
        
        assert_eq!(note.title, "新标题");
        assert_ne!(note.checksum, old_checksum);
        assert!(note.modified_at > old_modified);
    }

    #[test]
    fn test_title_extraction() {
        assert_eq!(Note::extract_title("# 标题"), "标题");
        assert_eq!(Note::extract_title("## 不是标题\n# 真标题"), "真标题");
        assert_eq!(Note::extract_title("没有标题的内容"), "无标题");
    }
}