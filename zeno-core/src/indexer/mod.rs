use anyhow::Result;
use std::path::PathBuf;

use crate::models::Note;

/// 文件索引器接口
#[async_trait::async_trait]
pub trait Indexer: Send + Sync {
    /// 添加或更新笔记索引
    async fn index_note(&mut self, note: &Note) -> Result<()>;
    
    /// 删除笔记索引
    async fn remove_note(&mut self, note_id: &str) -> Result<()>;
    
    /// 搜索笔记
    async fn search(&self, query: &str) -> Result<Vec<Note>>;
    
    /// 根据路径查找笔记
    async fn find_by_path(&self, path: &PathBuf) -> Result<Option<Note>>;
    
    /// 获取所有笔记
    async fn get_all_notes(&self) -> Result<Vec<Note>>;
}