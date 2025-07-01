use anyhow::Result;
use std::path::PathBuf;

/// 存储接口
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
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
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
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
impl Storage for LocalStorage {
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