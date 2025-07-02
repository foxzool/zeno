use crate::models::importer::*;
use crate::models::note::Note;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Instant;
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::Utc;

#[async_trait]
pub trait Importer: Send + Sync {
    /// 获取导入器的名称
    fn name(&self) -> &str;

    /// 获取导入器的版本
    fn version(&self) -> &str;

    /// 获取支持的文件扩展名
    fn supported_extensions(&self) -> Vec<&str>;

    /// 验证源路径是否有效
    async fn validate_source(&self, source_path: &str) -> Result<bool>;

    /// 预览导入内容
    async fn preview_import(&self, config: &ImportConfig) -> Result<ImportPreview>;

    /// 执行导入
    async fn import(&self, config: &ImportConfig) -> Result<ImportResult>;

    /// 执行导入的内部实现
    async fn import_internal(&self, config: &ImportConfig) -> Result<ImportResult>;

    /// 处理单个文件
    async fn process_file(&self, file_path: &Path, config: &ImportConfig) -> Result<ImportedFile>;

    /// 转换链接格式
    fn convert_links(&self, content: &str, mappings: &HashMap<String, String>) -> String;

    /// 转换标签格式
    fn convert_tags(&self, content: &str) -> String;

    /// 提取元数据
    fn extract_metadata(&self, content: &str) -> Result<HashMap<String, String>>;

    /// 检查文件冲突
    async fn check_conflicts(&self, config: &ImportConfig) -> Result<Vec<FileConflict>>;
}

pub struct BaseImporter {
    pub name: String,
    pub version: String,
}

impl BaseImporter {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    /// 扫描源目录获取所有文件
    pub async fn scan_source_directory(&self, source_path: &str, extensions: &[&str]) -> Result<Vec<PathBuf>> {
        let source = Path::new(source_path);
        if !source.exists() {
            return Err(anyhow!("Source path does not exist: {}", source_path));
        }

        // 简化为非递归实现
        let mut files = Vec::new();
        self.scan_directory_iterative(source, extensions, &mut files).await?;
        Ok(files)
    }

    /// 迭代式扫描目录
    async fn scan_directory_iterative(&self, dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // 递归扫描子目录 - 简化版本，只扫描一层
                if let Ok(mut sub_entries) = tokio::fs::read_dir(&path).await {
                    while let Some(sub_entry) = sub_entries.next_entry().await? {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() && self.is_supported_file(&sub_path, extensions) {
                            files.push(sub_path);
                        }
                    }
                }
            } else if self.is_supported_file(&path, extensions) {
                files.push(path);
            }
        }
        
        Ok(())
    }

    fn scan_directory_recursive<'a>(
        &'a self,
        dir: &'a Path,
        extensions: &'a [&str],
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // 避免递归调用，暂时跳过子目录
                    // self.scan_directory_recursive(&path, extensions, files).await?;
                } else if self.is_supported_file(&path, extensions) {
                    files.push(path);
                }
            }
            
            Ok(())
        })
    }

    fn is_supported_file(&self, path: &Path, extensions: &[&str]) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str);
            }
        }
        false
    }

    /// 创建目标目录结构
    pub async fn create_target_structure(&self, target_workspace: &str, preserve_structure: bool, source_files: &[PathBuf], source_root: &Path) -> Result<()> {
        if !preserve_structure {
            return Ok(());
        }

        for file_path in source_files {
            if let Ok(relative_path) = file_path.strip_prefix(source_root) {
                if let Some(parent) = relative_path.parent() {
                    let target_dir = Path::new(target_workspace).join(parent);
                    if !target_dir.exists() {
                        tokio::fs::create_dir_all(&target_dir).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 生成目标文件路径
    pub fn generate_target_path(&self, source_file: &Path, source_root: &Path, target_workspace: &str, preserve_structure: bool) -> Result<PathBuf> {
        let target_root = Path::new(target_workspace);
        
        if preserve_structure {
            let relative_path = source_file.strip_prefix(source_root)?;
            Ok(target_root.join(relative_path))
        } else {
            let filename = source_file.file_name()
                .ok_or_else(|| anyhow!("Invalid file name"))?;
            Ok(target_root.join(filename))
        }
    }

    /// 检查文件是否已存在
    pub async fn file_exists(&self, target_path: &Path) -> bool {
        tokio::fs::metadata(target_path).await.is_ok()
    }

    /// 备份现有文件
    pub async fn backup_existing_file(&self, target_path: &Path) -> Result<()> {
        if !self.file_exists(target_path).await {
            return Ok(());
        }

        let backup_path = self.generate_backup_path(target_path);
        tokio::fs::rename(target_path, backup_path).await?;
        Ok(())
    }

    fn generate_backup_path(&self, original_path: &Path) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let mut backup_path = original_path.to_path_buf();
        
        if let Some(stem) = original_path.file_stem() {
            if let Some(extension) = original_path.extension() {
                backup_path.set_file_name(format!("{}_backup_{}.{}", 
                    stem.to_string_lossy(), 
                    timestamp,
                    extension.to_string_lossy()
                ));
            } else {
                backup_path.set_file_name(format!("{}_backup_{}", 
                    stem.to_string_lossy(), 
                    timestamp
                ));
            }
        }
        
        backup_path
    }

    /// 复制附件文件
    pub async fn copy_attachment(&self, source_path: &Path, target_path: &Path) -> Result<()> {
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::copy(source_path, target_path).await?;
        Ok(())
    }

    /// 生成导入统计
    pub fn generate_import_stats(&self, files: &[ImportedFile], start_time: Instant) -> ImportResult {
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        let mut result = ImportResult::new();
        result.processing_time_ms = processing_time_ms;
        
        for file in files {
            match file.status {
                ImportStatus::Success => result.imported_count += 1,
                ImportStatus::Skipped => result.skipped_count += 1,
                ImportStatus::Failed => result.failed_count += 1,
                ImportStatus::Warning => {
                    result.imported_count += 1;
                    result.warnings.push(format!("Warning processing {}", file.source_path));
                }
            }
        }
        
        result.imported_files = files.to_vec();
        result.success = result.failed_count == 0;
        
        result
    }

    /// 创建目录节点树
    pub fn create_directory_tree(&self, files: &[PathBuf], root_path: &Path) -> Vec<DirectoryNode> {
        let mut root_nodes = Vec::new();
        let mut path_counts: HashMap<PathBuf, u32> = HashMap::new();
        
        // 计算每个目录的文件数量
        for file in files {
            if let Ok(relative_path) = file.strip_prefix(root_path) {
                let mut current_path = root_path.to_path_buf();
                
                for component in relative_path.components() {
                    current_path.push(component);
                    *path_counts.entry(current_path.clone()).or_insert(0) += 1;
                }
            }
        }
        
        // 构建目录树（简化版本，实际实现会更复杂）
        for (path, count) in path_counts {
            if path.parent() == Some(root_path) {
                root_nodes.push(DirectoryNode {
                    name: path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    file_count: count,
                    children: Vec::new(), // 简化版本，不递归构建子节点
                });
            }
        }
        
        root_nodes
    }

    /// 默认的链接转换实现
    pub fn default_convert_links(&self, content: &str, _mappings: &HashMap<String, String>) -> String {
        // 基础实现：将 [[link]] 格式转换为 [link](link.md)
        let link_regex = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        link_regex.replace_all(content, |caps: &regex::Captures| {
            let link_text = &caps[1];
            format!("[{}]({}.md)", link_text, link_text.replace(' ', "_").to_lowercase())
        }).to_string()
    }

    /// 默认的标签转换实现
    pub fn default_convert_tags(&self, content: &str) -> String {
        // 基础实现：将 #tag 格式保持不变
        content.to_string()
    }

    /// 默认的元数据提取实现
    pub fn default_extract_metadata(&self, content: &str) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        // 尝试解析 YAML frontmatter
        if content.starts_with("---") {
            if let Some(end_index) = content[3..].find("---") {
                let yaml_content = &content[3..end_index + 3];
                if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
                    if let Some(mapping) = yaml_value.as_mapping() {
                        for (key, value) in mapping {
                            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                                metadata.insert(key_str.to_string(), value_str.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(metadata)
    }
}