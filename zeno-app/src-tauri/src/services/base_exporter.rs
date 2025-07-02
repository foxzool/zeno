use crate::models::exporter::*;
use crate::models::note::Note;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Instant;
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::Utc;

#[async_trait]
pub trait Exporter: Send + Sync {
    /// 获取导出器的名称
    fn name(&self) -> &str;

    /// 获取导出器的版本
    fn version(&self) -> &str;

    /// 获取支持的输出格式
    fn supported_formats(&self) -> Vec<&str>;

    /// 验证目标路径是否有效
    async fn validate_target(&self, target_path: &str) -> Result<bool>;

    /// 预览导出内容
    async fn preview_export(&self, config: &ExportConfig) -> Result<ExportPreview>;

    /// 执行导出
    async fn export(&self, config: &ExportConfig) -> Result<ExportResult>;

    /// 执行导出的内部实现
    async fn export_internal(&self, config: &ExportConfig) -> Result<ExportResult>;

    /// 处理单个笔记
    async fn process_note(&self, note: &Note, config: &ExportConfig) -> Result<ExportedFile>;

    /// 转换内容格式
    fn convert_content(&self, content: &str, from_format: &str, to_format: &str) -> Result<String>;

    /// 处理链接重写
    fn rewrite_links(&self, content: &str, link_mappings: &HashMap<String, String>) -> String;

    /// 嵌入资源文件
    async fn embed_assets(&self, content: &str, base_path: &Path) -> Result<String>;

    /// 过滤笔记
    fn filter_notes(&self, notes: &[Note], filter_options: &FilterOptions) -> Vec<Note>;
}

pub struct BaseExporter {
    pub name: String,
    pub version: String,
}

impl BaseExporter {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    /// 扫描工作空间获取所有笔记
    pub async fn scan_workspace(&self, workspace_path: &str) -> Result<Vec<Note>> {
        let workspace = Path::new(workspace_path);
        if !workspace.exists() {
            return Err(anyhow!("Workspace path does not exist: {}", workspace_path));
        }

        let mut notes = Vec::new();
        self.scan_notes_iterative(workspace, &mut notes).await?;
        Ok(notes)
    }

    /// 迭代式扫描笔记
    async fn scan_notes_iterative(&self, dir: &Path, notes: &mut Vec<Note>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && self.is_markdown_file(&path) {
                if let Ok(note) = self.load_note(&path).await {
                    notes.push(note);
                }
            }
        }
        
        Ok(())
    }

    fn scan_notes_recursive<'a>(
        &'a self,
        dir: &'a Path,
        notes: &'a mut Vec<Note>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // 避免递归调用，暂时跳过子目录
                    // self.scan_notes_recursive(&path, notes).await?;
                } else if self.is_markdown_file(&path) {
                    if let Ok(note) = self.load_note(&path).await {
                        notes.push(note);
                    }
                }
            }
            
            Ok(())
        })
    }

    fn is_markdown_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return matches!(ext_str.to_lowercase().as_str(), "md" | "markdown" | "txt");
            }
        }
        false
    }

    async fn load_note(&self, path: &Path) -> Result<Note> {
        let content = tokio::fs::read_to_string(path).await?;
        let metadata = tokio::fs::metadata(path).await?;
        
        let title = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            
        let mut note = Note::new(path.to_path_buf(), title, content);
        
        // 更新时间戳
        note.created_at = metadata.created()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| Utc::now());
        note.modified_at = metadata.modified()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| Utc::now());
            
        Ok(note)
    }

    /// 创建输出目录结构
    pub async fn create_output_structure(&self, target_path: &str, preserve_structure: bool, notes: &[Note], workspace_root: &Path) -> Result<()> {
        let target_root = Path::new(target_path);
        
        if !target_root.exists() {
            tokio::fs::create_dir_all(target_root).await?;
        }

        if !preserve_structure {
            return Ok(());
        }

        for note in notes {
            let note_path = Path::new(&note.path);
            if let Ok(relative_path) = note_path.strip_prefix(workspace_root) {
                if let Some(parent) = relative_path.parent() {
                    let target_dir = target_root.join(parent);
                    if !target_dir.exists() {
                        tokio::fs::create_dir_all(&target_dir).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 生成输出文件路径
    pub fn generate_output_path(&self, note: &Note, workspace_root: &Path, target_path: &str, preserve_structure: bool, format: &str) -> Result<PathBuf> {
        let target_root = Path::new(target_path);
        let note_path = Path::new(&note.path);
        
        let output_filename = self.generate_output_filename(note, format);
        
        if preserve_structure {
            if let Ok(relative_path) = note_path.strip_prefix(workspace_root) {
                if let Some(parent) = relative_path.parent() {
                    return Ok(target_root.join(parent).join(output_filename));
                }
            }
        }
        
        Ok(target_root.join(output_filename))
    }

    fn generate_output_filename(&self, note: &Note, format: &str) -> String {
        let base_name = Path::new(&note.path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        
        format!("{}.{}", base_name, format)
    }

    /// 应用过滤器
    pub fn apply_filters(&self, notes: &[Note], filter_options: &FilterOptions) -> Vec<Note> {
        notes.iter()
            .filter(|note| self.passes_filter(note, filter_options))
            .cloned()
            .collect()
    }

    fn passes_filter(&self, note: &Note, filter_options: &FilterOptions) -> bool {
        // 日期范围过滤
        if let Some(date_range) = &filter_options.date_range {
            if note.modified_at < date_range.start || note.modified_at > date_range.end {
                return false;
            }
        }

        // 标签过滤
        if !filter_options.tag_filter.is_empty() {
            if let Some(ref frontmatter) = note.frontmatter {
                let has_matching_tag = filter_options.tag_filter.iter()
                    .any(|filter_tag| frontmatter.tags.iter().any(|note_tag| note_tag.contains(filter_tag)));
                if !has_matching_tag {
                    return false;
                }
            } else {
                return false; // 没有 frontmatter，无法过滤标签
            }
        }

        // 路径过滤
        if !filter_options.path_filter.is_empty() {
            let matches_path = filter_options.path_filter.iter()
                .any(|filter_path| note.path.to_string_lossy().contains(filter_path));
            if !matches_path {
                return false;
            }
        }

        // 内容过滤
        if let Some(content_filter) = &filter_options.content_filter {
            if !note.content.contains(content_filter) {
                return false;
            }
        }

        // 最小字数过滤
        if let Some(min_words) = filter_options.minimum_word_count {
            let word_count = note.content.split_whitespace().count() as u32;
            if word_count < min_words {
                return false;
            }
        }

        true
    }

    /// 处理附件文件
    pub async fn process_attachments(&self, notes: &[Note], target_path: &str, include_attachments: bool) -> Result<Vec<ExportedFile>> {
        if !include_attachments {
            return Ok(Vec::new());
        }

        let mut attachment_files = Vec::new();
        let attachment_dir = Path::new(target_path).join("attachments");
        
        if !attachment_dir.exists() {
            tokio::fs::create_dir_all(&attachment_dir).await?;
        }

        for note in notes {
            let attachments = self.extract_attachments_from_note(note)?;
            
            for attachment_path in attachments {
                let source_path = Path::new(&attachment_path);
                if source_path.exists() {
                    let filename = source_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let target_attachment_path = attachment_dir.join(&*filename);
                    
                    tokio::fs::copy(&source_path, &target_attachment_path).await?;
                    
                    let metadata = tokio::fs::metadata(&source_path).await?;
                    
                    attachment_files.push(ExportedFile {
                        source_path: attachment_path,
                        output_path: target_attachment_path.to_string_lossy().to_string(),
                        file_type: ExportFileType::Attachment,
                        original_size: metadata.len(),
                        exported_size: metadata.len(),
                        status: ExportStatus::Success,
                        transformations: Vec::new(),
                    });
                }
            }
        }

        Ok(attachment_files)
    }

    fn extract_attachments_from_note(&self, note: &Note) -> Result<Vec<String>> {
        let mut attachments = Vec::new();
        
        // 简单的正则表达式来匹配图片和链接
        let image_regex = regex::Regex::new(r"!\[.*?\]\(([^)]+)\)").unwrap();
        let link_regex = regex::Regex::new(r"\[.*?\]\(([^)]+)\)").unwrap();
        
        for caps in image_regex.captures_iter(&note.content) {
            if let Some(path) = caps.get(1) {
                let attachment_path = path.as_str();
                if !attachment_path.starts_with("http") {
                    // 相对路径，需要解析为绝对路径
                    let note_dir = Path::new(&note.path).parent().unwrap_or(Path::new("."));
                    let full_path = note_dir.join(attachment_path);
                    attachments.push(full_path.to_string_lossy().to_string());
                }
            }
        }
        
        for caps in link_regex.captures_iter(&note.content) {
            if let Some(path) = caps.get(1) {
                let link_path = path.as_str();
                if !link_path.starts_with("http") && !link_path.ends_with(".md") {
                    // 可能是本地文件链接
                    let note_dir = Path::new(&note.path).parent().unwrap_or(Path::new("."));
                    let full_path = note_dir.join(link_path);
                    if full_path.exists() && !self.is_markdown_file(&full_path) {
                        attachments.push(full_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(attachments)
    }

    /// 生成导出统计
    pub fn generate_export_stats(&self, files: &[ExportedFile], start_time: Instant) -> ExportResult {
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        let mut result = ExportResult::new();
        result.processing_time_ms = processing_time_ms;
        
        let mut total_original_size = 0u64;
        let mut total_exported_size = 0u64;
        
        for file in files {
            total_original_size += file.original_size;
            total_exported_size += file.exported_size;
            
            match file.status {
                ExportStatus::Success => result.exported_count += 1,
                ExportStatus::Skipped => result.skipped_count += 1,
                ExportStatus::Failed => result.failed_count += 1,
                ExportStatus::Warning => {
                    result.exported_count += 1;
                    result.warnings.push(format!("Warning processing {}", file.source_path));
                }
            }
        }
        
        result.output_files = files.to_vec();
        result.total_size = total_exported_size;
        result.success = result.failed_count == 0;
        
        if total_original_size > 0 {
            result.compression_ratio = Some(
                (total_exported_size as f64) / (total_original_size as f64)
            );
        }
        
        result
    }

    /// 创建导出目录树
    pub fn create_export_directory_tree(&self, notes: &[Note], workspace_root: &Path) -> Vec<ExportDirectoryNode> {
        let mut root_nodes = Vec::new();
        let mut path_counts: HashMap<PathBuf, (u32, u32)> = HashMap::new(); // (note_count, attachment_count)
        
        // 计算每个目录的文件数量
        for note in notes {
            let note_path = Path::new(&note.path);
            if let Ok(relative_path) = note_path.strip_prefix(workspace_root) {
                let mut current_path = workspace_root.to_path_buf();
                
                for component in relative_path.components() {
                    current_path.push(component);
                    let (note_count, attachment_count) = path_counts.entry(current_path.clone()).or_insert((0, 0));
                    *note_count += 1;
                    
                    // 简化：每个笔记假设有0个附件
                    if let Ok(attachments) = self.extract_attachments_from_note(note) {
                        *attachment_count += attachments.len() as u32;
                    }
                }
            }
        }
        
        // 构建目录树（简化版本）
        for (path, (note_count, attachment_count)) in path_counts {
            if path.parent() == Some(workspace_root) {
                root_nodes.push(ExportDirectoryNode {
                    name: path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    note_count,
                    attachment_count,
                    children: Vec::new(), // 简化版本，不递归构建子节点
                });
            }
        }
        
        root_nodes
    }

    /// 默认的内容转换实现
    pub fn default_convert_content(&self, content: &str, _from_format: &str, _to_format: &str) -> Result<String> {
        // 基础实现：直接返回原内容
        Ok(content.to_string())
    }

    /// 默认的链接重写实现
    pub fn default_rewrite_links(&self, content: &str, link_mappings: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        
        for (from_link, to_link) in link_mappings {
            result = result.replace(from_link, to_link);
        }
        
        result
    }

    /// 默认的资源嵌入实现
    pub async fn default_embed_assets(&self, content: &str, _base_path: &Path) -> Result<String> {
        // 基础实现：不进行资源嵌入
        Ok(content.to_string())
    }
}