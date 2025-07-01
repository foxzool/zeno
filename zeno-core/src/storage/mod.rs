pub mod repository;

use crate::error::Result;
use crate::db::models::{Note, Tag, TreeNode, Statistics, SearchQuery, SearchResult};
use std::path::PathBuf;

pub use repository::*;

/// 文件存储接口
#[async_trait::async_trait]
pub trait FileStorage: Send + Sync {
    /// 读取文件内容
    async fn read_file(&self, path: &PathBuf) -> Result<String>;
    
    /// 写入文件内容
    async fn write_file(&self, path: &PathBuf, content: &str) -> Result<()>;
    
    /// 删除文件
    async fn delete_file(&self, path: &PathBuf) -> Result<()>;
    
    /// 检查文件是否存在
    async fn file_exists(&self, path: &PathBuf) -> Result<bool>;
    
    /// 列出目录下的所有Markdown文件
    async fn list_markdown_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>>;
    
    /// 获取文件元数据
    async fn get_file_metadata(&self, path: &PathBuf) -> Result<FileMetadata>;
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// 文件大小（字节）
    pub size: u64,
    /// 创建时间
    pub created: std::time::SystemTime,
    /// 修改时间
    pub modified: std::time::SystemTime,
    /// 是否为目录
    pub is_dir: bool,
}

/// 本地文件系统存储实现
pub struct LocalFileStorage {
    base_path: PathBuf,
}

impl LocalFileStorage {
    /// 创建新的本地存储
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    /// 获取完整路径
    fn full_path(&self, path: &PathBuf) -> PathBuf {
        if path.is_absolute() {
            path.clone()
        } else {
            self.base_path.join(path)
        }
    }
}

#[async_trait::async_trait]
impl FileStorage for LocalFileStorage {
    async fn read_file(&self, path: &PathBuf) -> Result<String> {
        let full_path = self.full_path(path);
        let content = tokio::fs::read_to_string(full_path).await?;
        Ok(content)
    }
    
    async fn write_file(&self, path: &PathBuf, content: &str) -> Result<()> {
        let full_path = self.full_path(path);
        
        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(full_path, content).await?;
        Ok(())
    }
    
    async fn delete_file(&self, path: &PathBuf) -> Result<()> {
        let full_path = self.full_path(path);
        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }
    
    async fn file_exists(&self, path: &PathBuf) -> Result<bool> {
        let full_path = self.full_path(path);
        let exists = tokio::fs::try_exists(full_path).await?;
        Ok(exists)
    }
    
    async fn list_markdown_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>> {
        let full_dir = self.full_path(dir);
        let mut files = Vec::new();
        
        fn collect_markdown_files(
            dir: &PathBuf, 
            base: &PathBuf, 
            files: &mut Vec<PathBuf>
        ) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    collect_markdown_files(&path, base, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(relative_path) = path.strip_prefix(base) {
                        files.push(relative_path.to_path_buf());
                    }
                }
            }
            Ok(())
        }
        
        collect_markdown_files(&full_dir, &self.base_path, &mut files)?;
        Ok(files)
    }
    
    async fn get_file_metadata(&self, path: &PathBuf) -> Result<FileMetadata> {
        let full_path = self.full_path(path);
        let metadata = tokio::fs::metadata(full_path).await?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            created: metadata.created()?,
            modified: metadata.modified()?,
            is_dir: metadata.is_dir(),
        })
    }
}

/// 知识库管理器 - 整合文件存储和数据库存储
pub struct KnowledgeBase {
    file_storage: Box<dyn FileStorage>,
    note_repository: Box<dyn NoteRepository>,
}

impl KnowledgeBase {
    pub fn new(
        file_storage: Box<dyn FileStorage>,
        note_repository: Box<dyn NoteRepository>,
    ) -> Self {
        Self {
            file_storage,
            note_repository,
        }
    }

    /// 从文件系统加载笔记到数据库
    pub async fn import_note_from_file(&self, file_path: &PathBuf) -> Result<Note> {
        // 读取文件内容
        let content = self.file_storage.read_file(file_path).await?;
        
        // 解析 Markdown
        let parser = crate::parser::MarkdownParser::new();
        let mut note = parser.parse(&content, file_path.clone())?;
        
        // 获取文件元数据
        let metadata = self.file_storage.get_file_metadata(file_path).await?;
        note.file_size = metadata.size as i64;
        
        // 保存到数据库
        self.note_repository.save_note(&note).await?;
        
        Ok(note)
    }

    /// 将笔记导出到文件系统
    pub async fn export_note_to_file(&self, note: &Note, file_path: &PathBuf) -> Result<()> {
        // 构建完整的 Markdown 内容（包含 frontmatter）
        let frontmatter = note.get_frontmatter()?;
        let yaml_content = serde_yaml::to_string(&frontmatter)?;
        
        let full_content = if !yaml_content.trim().is_empty() && yaml_content.trim() != "{}" {
            format!("---\n{}\n---\n\n{}", yaml_content.trim(), note.content)
        } else {
            note.content.clone()
        };
        
        // 写入文件
        self.file_storage.write_file(file_path, &full_content).await?;
        
        Ok(())
    }

    /// 同步文件系统到数据库
    pub async fn sync_filesystem_to_database(&self, directory: &PathBuf) -> Result<Vec<Note>> {
        let mut synchronized_notes = Vec::new();
        
        // 获取所有 Markdown 文件
        let file_paths = self.file_storage.list_markdown_files(directory).await?;
        
        for file_path in file_paths {
            // 检查数据库中是否存在该笔记
            let existing_note = self.note_repository.get_note_by_path(&file_path.to_string_lossy()).await?;
            
            // 获取文件元数据
            let file_metadata = self.file_storage.get_file_metadata(&file_path).await?;
            
            let should_update = if let Some(existing) = &existing_note {
                // 检查文件是否已修改
                existing.modified_at.timestamp() < file_metadata.modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
            } else {
                true // 新文件
            };
            
            if should_update {
                let note = self.import_note_from_file(&file_path).await?;
                synchronized_notes.push(note);
            }
        }
        
        Ok(synchronized_notes)
    }

    /// 获取知识库统计信息
    pub async fn get_statistics(&self) -> Result<Statistics> {
        self.note_repository.get_statistics().await
    }

    /// 搜索笔记
    pub async fn search_notes(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        self.note_repository.search_notes(query).await
    }

    /// 获取笔记
    pub async fn get_note(&self, note_id: &str) -> Result<Option<Note>> {
        self.note_repository.get_note_by_id(note_id).await
    }

    /// 保存笔记
    pub async fn save_note(&self, note: &Note) -> Result<()> {
        self.note_repository.save_note(note).await
    }

    /// 删除笔记
    pub async fn delete_note(&self, note_id: &str) -> Result<()> {
        self.note_repository.delete_note(note_id).await
    }

    /// 列出所有笔记
    pub async fn list_notes(&self, filter: Option<NoteFilter>) -> Result<Vec<Note>> {
        self.note_repository.list_notes(filter).await
    }

    /// 获取所有标签
    pub async fn get_tags(&self) -> Result<Vec<Tag>> {
        self.note_repository.get_all_tags().await
    }

    /// 获取分类树
    pub async fn get_category_tree(&self) -> Result<Vec<TreeNode>> {
        self.note_repository.get_category_tree().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_local_file_storage() {
        let dir = tempdir().unwrap();
        let storage = LocalFileStorage::new(dir.path().to_path_buf());
        
        let test_path = PathBuf::from("test.md");
        let content = "# Test\n\nThis is a test.";
        
        // 写入文件
        storage.write_file(&test_path, content).await.unwrap();
        
        // 检查文件存在
        assert!(storage.file_exists(&test_path).await.unwrap());
        
        // 读取文件
        let read_content = storage.read_file(&test_path).await.unwrap();
        assert_eq!(read_content, content);
        
        // 获取元数据
        let metadata = storage.get_file_metadata(&test_path).await.unwrap();
        assert!(!metadata.is_dir);
        assert!(metadata.size > 0);
        
        // 删除文件
        storage.delete_file(&test_path).await.unwrap();
        assert!(!storage.file_exists(&test_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_list_markdown_files() {
        let dir = tempdir().unwrap();
        let storage = LocalFileStorage::new(dir.path().to_path_buf());
        
        // 创建测试文件
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("test1.md"), "# Test 1").unwrap();
        fs::write(dir.path().join("test2.md"), "# Test 2").unwrap();
        fs::write(dir.path().join("subdir/test3.md"), "# Test 3").unwrap();
        fs::write(dir.path().join("not_markdown.txt"), "Not markdown").unwrap();
        
        let files = storage.list_markdown_files(&PathBuf::from(".")).await.unwrap();
        
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|p| p.to_string_lossy().contains("test1.md")));
        assert!(files.iter().any(|p| p.to_string_lossy().contains("test2.md")));
        assert!(files.iter().any(|p| p.to_string_lossy().contains("test3.md")));
    }
}