use crate::models::importer::*;
use crate::models::exporter::*;
use crate::services::base_importer::{Importer, BaseImporter};
use crate::services::base_exporter::{Exporter, BaseExporter};
use crate::services::obsidian_importer::ObsidianImporter;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

pub struct ImportExportManager {
    importers: HashMap<ImporterType, Arc<dyn Importer>>,
    exporters: HashMap<ExporterType, Arc<dyn Exporter>>,
}

impl ImportExportManager {
    pub fn new() -> Self {
        let mut manager = Self {
            importers: HashMap::new(),
            exporters: HashMap::new(),
        };
        
        // 注册默认导入器
        manager.register_default_importers();
        
        // 注册默认导出器
        manager.register_default_exporters();
        
        manager
    }

    /// 注册默认导入器
    fn register_default_importers(&mut self) {
        // Obsidian 导入器
        self.importers.insert(
            ImporterType::Obsidian,
            Arc::new(ObsidianImporter::new())
        );
        
        // 基础 Markdown 导入器
        self.importers.insert(
            ImporterType::Markdown,
            Arc::new(MarkdownImporter::new())
        );
        
        // 通用导入器
        self.importers.insert(
            ImporterType::Generic,
            Arc::new(GenericImporter::new())
        );
    }

    /// 注册默认导出器
    fn register_default_exporters(&mut self) {
        // Markdown 导出器
        self.exporters.insert(
            ExporterType::Markdown,
            Arc::new(MarkdownExporter::new())
        );
        
        // HTML 导出器
        self.exporters.insert(
            ExporterType::Html,
            Arc::new(HtmlExporter::new())
        );
        
        // JSON 导出器
        self.exporters.insert(
            ExporterType::Json,
            Arc::new(JsonExporter::new())
        );
    }

    /// 注册自定义导入器
    pub fn register_importer(&mut self, importer_type: ImporterType, importer: Arc<dyn Importer>) {
        self.importers.insert(importer_type, importer);
    }

    /// 注册自定义导出器
    pub fn register_exporter(&mut self, exporter_type: ExporterType, exporter: Arc<dyn Exporter>) {
        self.exporters.insert(exporter_type, exporter);
    }

    /// 获取所有可用的导入器类型
    pub fn get_available_importers(&self) -> Vec<ImporterType> {
        self.importers.keys().cloned().collect()
    }

    /// 获取所有可用的导出器类型
    pub fn get_available_exporters(&self) -> Vec<ExporterType> {
        self.exporters.keys().cloned().collect()
    }

    /// 预览导入
    pub async fn preview_import(&self, config: &ImportConfig) -> Result<ImportPreview> {
        let importer = self.importers.get(&config.importer_type)
            .ok_or_else(|| anyhow!("Unsupported importer type: {:?}", config.importer_type))?;
        
        importer.preview_import(config).await
    }

    /// 执行导入
    pub async fn import(&self, config: &ImportConfig) -> Result<ImportResult> {
        let importer = self.importers.get(&config.importer_type)
            .ok_or_else(|| anyhow!("Unsupported importer type: {:?}", config.importer_type))?;
        
        // 验证源路径
        if !importer.validate_source(&config.source_path).await? {
            return Err(anyhow!("Invalid source path: {}", config.source_path));
        }
        
        importer.import(config).await
    }

    /// 预览导出
    pub async fn preview_export(&self, config: &ExportConfig) -> Result<ExportPreview> {
        let exporter = self.exporters.get(&config.exporter_type)
            .ok_or_else(|| anyhow!("Unsupported exporter type: {:?}", config.exporter_type))?;
        
        exporter.preview_export(config).await
    }

    /// 执行导出
    pub async fn export(&self, config: &ExportConfig) -> Result<ExportResult> {
        let exporter = self.exporters.get(&config.exporter_type)
            .ok_or_else(|| anyhow!("Unsupported exporter type: {:?}", config.exporter_type))?;
        
        // 验证目标路径
        if !exporter.validate_target(&config.target_path).await? {
            return Err(anyhow!("Invalid target path: {}", config.target_path));
        }
        
        exporter.export(config).await
    }
}

impl Default for ImportExportManager {
    fn default() -> Self {
        Self::new()
    }
}

// 基础 Markdown 导入器实现
pub struct MarkdownImporter {
    base: BaseImporter,
}

impl MarkdownImporter {
    pub fn new() -> Self {
        Self {
            base: BaseImporter::new("Markdown Importer".to_string(), "1.0.0".to_string()),
        }
    }

    fn is_supported_file_impl(&self, path: &std::path::Path, extensions: &[&str]) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return extensions.contains(&ext_str);
            }
        }
        false
    }
}

#[async_trait]
impl Importer for MarkdownImporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["md", "markdown", "txt"]
    }

    async fn validate_source(&self, source_path: &str) -> Result<bool> {
        let path = std::path::Path::new(source_path);
        Ok(path.exists() && (path.is_dir() || self.is_supported_file_impl(path, &self.supported_extensions())))
    }

    async fn preview_import(&self, config: &ImportConfig) -> Result<ImportPreview> {
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        
        let mut preview = ImportPreview {
            total_files: files.len() as u32,
            notes: files.len() as u32,
            attachments: 0,
            media_files: 0,
            estimated_size: 0,
            warnings: Vec::new(),
            conflicts: Vec::new(),
            structure: Vec::new(),
        };

        // 计算估计大小
        for file in &files {
            if let Ok(metadata) = tokio::fs::metadata(file).await {
                preview.estimated_size += metadata.len();
            }
        }

        // 检查冲突
        preview.conflicts = self.check_conflicts(config).await?;

        // 创建目录结构
        let source_root = std::path::Path::new(&config.source_path);
        preview.structure = self.base.create_directory_tree(&files, source_root);

        Ok(preview)
    }

    async fn import(&self, config: &ImportConfig) -> Result<ImportResult> {
        self.import_internal(config).await
    }

    async fn import_internal(&self, config: &ImportConfig) -> Result<ImportResult> {
        let start_time = std::time::Instant::now();
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        
        // 创建目标目录结构
        let source_root = std::path::Path::new(&config.source_path);
        self.base.create_target_structure(&config.target_workspace, config.options.preserve_structure, &files, source_root).await?;
        
        let mut imported_files = Vec::new();
        
        for file_path in files {
            match self.process_file(&file_path, config).await {
                Ok(imported_file) => imported_files.push(imported_file),
                Err(e) => {
                    imported_files.push(ImportedFile {
                        source_path: file_path.to_string_lossy().to_string(),
                        target_path: String::new(),
                        file_type: FileType::Note,
                        size: 0,
                        modified_time: chrono::Utc::now(),
                        status: ImportStatus::Failed,
                        transformations: Vec::new(),
                    });
                }
            }
        }
        
        Ok(self.base.generate_import_stats(&imported_files, start_time))
    }

    async fn process_file(&self, file_path: &std::path::Path, config: &ImportConfig) -> Result<ImportedFile> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let metadata = tokio::fs::metadata(file_path).await?;
        
        // 转换内容
        let mut transformed_content = content;
        let mut transformations = Vec::new();
        
        if config.options.convert_links {
            let new_content = self.convert_links(&transformed_content, &config.options.custom_mappings);
            if new_content != transformed_content {
                transformations.push(Transformation {
                    transformation_type: TransformationType::LinkConversion,
                    description: "Converted wiki-style links to markdown links".to_string(),
                    from_value: "[[link]]".to_string(),
                    to_value: "[link](link.md)".to_string(),
                });
                transformed_content = new_content;
            }
        }
        
        if config.options.convert_tags {
            let new_content = self.convert_tags(&transformed_content);
            if new_content != transformed_content {
                transformations.push(Transformation {
                    transformation_type: TransformationType::TagConversion,
                    description: "Converted tag formats".to_string(),
                    from_value: "#tag".to_string(),
                    to_value: "#tag".to_string(),
                });
                transformed_content = new_content;
            }
        }
        
        // 生成目标路径
        let source_root = std::path::Path::new(&config.source_path);
        let target_path = self.base.generate_target_path(file_path, source_root, &config.target_workspace, config.options.preserve_structure)?;
        
        // 处理文件冲突
        if self.base.file_exists(&target_path).await {
            match config.options.merge_mode {
                MergeMode::Skip => {
                    return Ok(ImportedFile {
                        source_path: file_path.to_string_lossy().to_string(),
                        target_path: target_path.to_string_lossy().to_string(),
                        file_type: FileType::Note,
                        size: metadata.len(),
                        modified_time: metadata.modified().map(chrono::DateTime::from).unwrap_or_else(|_| chrono::Utc::now()),
                        status: ImportStatus::Skipped,
                        transformations,
                    });
                }
                MergeMode::Overwrite => {
                    if config.options.backup_existing {
                        self.base.backup_existing_file(&target_path).await?;
                    }
                }
                _ => {
                    // 其他合并模式的实现
                }
            }
        }
        
        // 写入文件
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&target_path, transformed_content).await?;
        
        Ok(ImportedFile {
            source_path: file_path.to_string_lossy().to_string(),
            target_path: target_path.to_string_lossy().to_string(),
            file_type: FileType::Note,
            size: metadata.len(),
            modified_time: metadata.modified().map(chrono::DateTime::from).unwrap_or_else(|_| chrono::Utc::now()),
            status: ImportStatus::Success,
            transformations,
        })
    }

    fn convert_links(&self, content: &str, mappings: &HashMap<String, String>) -> String {
        self.base.default_convert_links(content, mappings)
    }

    fn convert_tags(&self, content: &str) -> String {
        self.base.default_convert_tags(content)
    }

    fn extract_metadata(&self, content: &str) -> Result<HashMap<String, String>> {
        self.base.default_extract_metadata(content)
    }

    async fn check_conflicts(&self, config: &ImportConfig) -> Result<Vec<FileConflict>> {
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        let mut conflicts = Vec::new();
        
        let source_root = std::path::Path::new(&config.source_path);
        
        for file_path in files {
            let target_path = self.base.generate_target_path(&file_path, source_root, &config.target_workspace, config.options.preserve_structure)?;
            
            if self.base.file_exists(&target_path).await {
                conflicts.push(FileConflict {
                    source_path: file_path.to_string_lossy().to_string(),
                    target_path: target_path.to_string_lossy().to_string(),
                    conflict_type: ConflictType::NameCollision,
                    suggested_resolution: match config.options.merge_mode {
                        MergeMode::Skip => "File will be skipped".to_string(),
                        MergeMode::Overwrite => "File will be overwritten".to_string(),
                        MergeMode::Merge => "Files will be merged".to_string(),
                        MergeMode::Rename => "File will be renamed".to_string(),
                    },
                });
            }
        }
        
        Ok(conflicts)
    }
}

// 通用导入器实现
pub struct GenericImporter {
    base: BaseImporter,
}

impl GenericImporter {
    pub fn new() -> Self {
        Self {
            base: BaseImporter::new("Generic Importer".to_string(), "1.0.0".to_string()),
        }
    }
}

#[async_trait]
impl Importer for GenericImporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["txt", "md", "markdown", "html", "htm", "json", "xml"]
    }

    async fn validate_source(&self, source_path: &str) -> Result<bool> {
        let path = std::path::Path::new(source_path);
        Ok(path.exists())
    }

    async fn preview_import(&self, config: &ImportConfig) -> Result<ImportPreview> {
        // 简化的预览实现
        Ok(ImportPreview {
            total_files: 0,
            notes: 0,
            attachments: 0,
            media_files: 0,
            estimated_size: 0,
            warnings: vec!["Generic importer: limited functionality".to_string()],
            conflicts: Vec::new(),
            structure: Vec::new(),
        })
    }

    async fn import(&self, config: &ImportConfig) -> Result<ImportResult> {
        self.import_internal(config).await
    }

    async fn import_internal(&self, config: &ImportConfig) -> Result<ImportResult> {
        // 简化的导入实现
        Ok(ImportResult::new())
    }

    async fn process_file(&self, _file_path: &std::path::Path, _config: &ImportConfig) -> Result<ImportedFile> {
        Err(anyhow!("Generic importer: process_file not implemented"))
    }

    fn convert_links(&self, content: &str, mappings: &HashMap<String, String>) -> String {
        self.base.default_convert_links(content, mappings)
    }

    fn convert_tags(&self, content: &str) -> String {
        self.base.default_convert_tags(content)
    }

    fn extract_metadata(&self, content: &str) -> Result<HashMap<String, String>> {
        self.base.default_extract_metadata(content)
    }

    async fn check_conflicts(&self, _config: &ImportConfig) -> Result<Vec<FileConflict>> {
        Ok(Vec::new())
    }
}

// 基础导出器实现
pub struct MarkdownExporter {
    base: BaseExporter,
}

impl MarkdownExporter {
    pub fn new() -> Self {
        Self {
            base: BaseExporter::new("Markdown Exporter".to_string(), "1.0.0".to_string()),
        }
    }
}

#[async_trait]
impl Exporter for MarkdownExporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["md", "markdown"]
    }

    async fn validate_target(&self, target_path: &str) -> Result<bool> {
        let path = std::path::Path::new(target_path);
        if let Some(parent) = path.parent() {
            Ok(parent.exists() || tokio::fs::create_dir_all(parent).await.is_ok())
        } else {
            Ok(true)
        }
    }

    async fn preview_export(&self, config: &ExportConfig) -> Result<ExportPreview> {
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        let mut total_size = 0u64;
        for note in &filtered_notes {
            total_size += note.content.len() as u64;
        }
        
        Ok(ExportPreview {
            total_notes: notes.len() as u32,
            total_attachments: 0,
            estimated_size: total_size,
            filtered_notes: filtered_notes.len() as u32,
            warnings: Vec::new(),
            structure: self.base.create_export_directory_tree(&filtered_notes, std::path::Path::new(&config.source_workspace)),
        })
    }

    async fn export(&self, config: &ExportConfig) -> Result<ExportResult> {
        self.export_internal(config).await
    }

    async fn export_internal(&self, config: &ExportConfig) -> Result<ExportResult> {
        let start_time = std::time::Instant::now();
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        // 创建输出目录结构
        let workspace_root = std::path::Path::new(&config.source_workspace);
        self.base.create_output_structure(&config.target_path, config.options.preserve_structure, &filtered_notes, workspace_root).await?;
        
        let mut exported_files = Vec::new();
        
        for note in filtered_notes {
            match self.process_note(&note, config).await {
                Ok(exported_file) => exported_files.push(exported_file),
                Err(_e) => {
                    exported_files.push(ExportedFile {
                        source_path: note.path.to_string_lossy().to_string(),
                        output_path: String::new(),
                        file_type: ExportFileType::Note,
                        original_size: note.content.len() as u64,
                        exported_size: 0,
                        status: ExportStatus::Failed,
                        transformations: Vec::new(),
                    });
                }
            }
        }
        
        // 处理附件
        if config.options.include_attachments {
            let attachment_files = self.base.process_attachments(&notes, &config.target_path, true).await?;
            exported_files.extend(attachment_files);
        }
        
        Ok(self.base.generate_export_stats(&exported_files, start_time))
    }

    async fn process_note(&self, note: &crate::models::note::Note, config: &ExportConfig) -> Result<ExportedFile> {
        let workspace_root = std::path::Path::new(&config.source_workspace);
        let output_path = self.base.generate_output_path(note, workspace_root, &config.target_path, config.options.preserve_structure, "md")?;
        
        let mut content = note.content.clone();
        let mut transformations = Vec::new();
        
        if config.options.convert_links {
            let link_mappings = HashMap::new(); // 简化实现
            let new_content = self.rewrite_links(&content, &link_mappings);
            if new_content != content {
                transformations.push(ExportTransformation {
                    transformation_type: ExportTransformationType::LinkRewriting,
                    description: "Rewritten internal links".to_string(),
                    from_format: "original".to_string(),
                    to_format: "markdown".to_string(),
                });
                content = new_content;
            }
        }
        
        // 写入文件
        tokio::fs::write(&output_path, &content).await?;
        
        Ok(ExportedFile {
            source_path: note.path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            file_type: ExportFileType::Note,
            original_size: note.content.len() as u64,
            exported_size: content.len() as u64,
            status: ExportStatus::Success,
            transformations,
        })
    }

    fn convert_content(&self, content: &str, from_format: &str, to_format: &str) -> Result<String> {
        self.base.default_convert_content(content, from_format, to_format)
    }

    fn rewrite_links(&self, content: &str, link_mappings: &HashMap<String, String>) -> String {
        self.base.default_rewrite_links(content, link_mappings)
    }

    async fn embed_assets(&self, content: &str, base_path: &std::path::Path) -> Result<String> {
        self.base.default_embed_assets(content, base_path).await
    }

    fn filter_notes(&self, notes: &[crate::models::note::Note], filter_options: &FilterOptions) -> Vec<crate::models::note::Note> {
        self.base.apply_filters(notes, filter_options)
    }
}

// HTML 导出器实现
pub struct HtmlExporter {
    base: BaseExporter,
}

impl HtmlExporter {
    pub fn new() -> Self {
        Self {
            base: BaseExporter::new("HTML Exporter".to_string(), "1.0.0".to_string()),
        }
    }
}

#[async_trait]
impl Exporter for HtmlExporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["html", "htm"]
    }

    async fn validate_target(&self, target_path: &str) -> Result<bool> {
        let path = std::path::Path::new(target_path);
        if let Some(parent) = path.parent() {
            Ok(parent.exists() || tokio::fs::create_dir_all(parent).await.is_ok())
        } else {
            Ok(true)
        }
    }

    async fn preview_export(&self, config: &ExportConfig) -> Result<ExportPreview> {
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        Ok(ExportPreview {
            total_notes: notes.len() as u32,
            total_attachments: 0,
            estimated_size: filtered_notes.iter().map(|n| n.content.len() as u64).sum(),
            filtered_notes: filtered_notes.len() as u32,
            warnings: Vec::new(),
            structure: self.base.create_export_directory_tree(&filtered_notes, std::path::Path::new(&config.source_workspace)),
        })
    }

    async fn export(&self, config: &ExportConfig) -> Result<ExportResult> {
        self.export_internal(config).await
    }

    async fn export_internal(&self, config: &ExportConfig) -> Result<ExportResult> {
        let start_time = std::time::Instant::now();
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        let mut exported_files = Vec::new();
        
        for note in filtered_notes {
            match self.process_note(&note, config).await {
                Ok(exported_file) => exported_files.push(exported_file),
                Err(_e) => {
                    exported_files.push(ExportedFile {
                        source_path: note.path.to_string_lossy().to_string(),
                        output_path: String::new(),
                        file_type: ExportFileType::Note,
                        original_size: note.content.len() as u64,
                        exported_size: 0,
                        status: ExportStatus::Failed,
                        transformations: Vec::new(),
                    });
                }
            }
        }
        
        Ok(self.base.generate_export_stats(&exported_files, start_time))
    }

    async fn process_note(&self, note: &crate::models::note::Note, config: &ExportConfig) -> Result<ExportedFile> {
        let workspace_root = std::path::Path::new(&config.source_workspace);
        let output_path = self.base.generate_output_path(note, workspace_root, &config.target_path, config.options.preserve_structure, "html")?;
        
        // 将 Markdown 转换为 HTML
        let html_content = self.markdown_to_html(&note.content)?;
        
        // 写入文件
        tokio::fs::write(&output_path, &html_content).await?;
        
        Ok(ExportedFile {
            source_path: note.path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            file_type: ExportFileType::Note,
            original_size: note.content.len() as u64,
            exported_size: html_content.len() as u64,
            status: ExportStatus::Success,
            transformations: vec![ExportTransformation {
                transformation_type: ExportTransformationType::FormatConversion,
                description: "Converted Markdown to HTML".to_string(),
                from_format: "markdown".to_string(),
                to_format: "html".to_string(),
            }],
        })
    }

    fn convert_content(&self, content: &str, from_format: &str, to_format: &str) -> Result<String> {
        if from_format == "markdown" && to_format == "html" {
            self.markdown_to_html(content)
        } else {
            self.base.default_convert_content(content, from_format, to_format)
        }
    }

    fn rewrite_links(&self, content: &str, link_mappings: &HashMap<String, String>) -> String {
        self.base.default_rewrite_links(content, link_mappings)
    }

    async fn embed_assets(&self, content: &str, base_path: &std::path::Path) -> Result<String> {
        self.base.default_embed_assets(content, base_path).await
    }

    fn filter_notes(&self, notes: &[crate::models::note::Note], filter_options: &FilterOptions) -> Vec<crate::models::note::Note> {
        self.base.apply_filters(notes, filter_options)
    }
}

impl HtmlExporter {
    fn markdown_to_html(&self, markdown: &str) -> Result<String> {
        use pulldown_cmark::{Parser, html};
        
        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        // 包装在完整的 HTML 文档中
        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Exported Note</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1, h2, h3, h4, h5, h6 {{ color: #333; }}
        code {{ background: #f4f4f4; padding: 2px 4px; border-radius: 3px; }}
        pre {{ background: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }}
        blockquote {{ border-left: 4px solid #ddd; margin: 0; padding-left: 20px; color: #666; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            html_output
        );
        
        Ok(full_html)
    }
}

// JSON 导出器实现
pub struct JsonExporter {
    base: BaseExporter,
}

impl JsonExporter {
    pub fn new() -> Self {
        Self {
            base: BaseExporter::new("JSON Exporter".to_string(), "1.0.0".to_string()),
        }
    }
}

#[async_trait]
impl Exporter for JsonExporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["json"]
    }

    async fn validate_target(&self, target_path: &str) -> Result<bool> {
        let path = std::path::Path::new(target_path);
        if let Some(parent) = path.parent() {
            Ok(parent.exists() || tokio::fs::create_dir_all(parent).await.is_ok())
        } else {
            Ok(true)
        }
    }

    async fn preview_export(&self, config: &ExportConfig) -> Result<ExportPreview> {
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        Ok(ExportPreview {
            total_notes: notes.len() as u32,
            total_attachments: 0,
            estimated_size: filtered_notes.iter().map(|n| n.content.len() as u64).sum(),
            filtered_notes: filtered_notes.len() as u32,
            warnings: Vec::new(),
            structure: Vec::new(),
        })
    }

    async fn export(&self, config: &ExportConfig) -> Result<ExportResult> {
        self.export_internal(config).await
    }

    async fn export_internal(&self, config: &ExportConfig) -> Result<ExportResult> {
        let start_time = std::time::Instant::now();
        let notes = self.base.scan_workspace(&config.source_workspace).await?;
        let filtered_notes = self.filter_notes(&notes, &config.options.filter_options);
        
        // 将所有笔记序列化为 JSON
        let json_content = serde_json::to_string_pretty(&filtered_notes)?;
        
        // 写入单个 JSON 文件
        let output_path = std::path::Path::new(&config.target_path);
        tokio::fs::write(output_path, &json_content).await?;
        
        let exported_file = ExportedFile {
            source_path: config.source_workspace.clone(),
            output_path: config.target_path.clone(),
            file_type: ExportFileType::Archive,
            original_size: filtered_notes.iter().map(|n| n.content.len() as u64).sum(),
            exported_size: json_content.len() as u64,
            status: ExportStatus::Success,
            transformations: vec![ExportTransformation {
                transformation_type: ExportTransformationType::FormatConversion,
                description: "Exported all notes as JSON".to_string(),
                from_format: "markdown".to_string(),
                to_format: "json".to_string(),
            }],
        };
        
        Ok(self.base.generate_export_stats(&[exported_file], start_time))
    }

    async fn process_note(&self, _note: &crate::models::note::Note, _config: &ExportConfig) -> Result<ExportedFile> {
        // JSON 导出器一次性处理所有笔记，不单独处理
        Err(anyhow!("JSON exporter processes all notes at once"))
    }

    fn convert_content(&self, content: &str, from_format: &str, to_format: &str) -> Result<String> {
        self.base.default_convert_content(content, from_format, to_format)
    }

    fn rewrite_links(&self, content: &str, link_mappings: &HashMap<String, String>) -> String {
        self.base.default_rewrite_links(content, link_mappings)
    }

    async fn embed_assets(&self, content: &str, base_path: &std::path::Path) -> Result<String> {
        self.base.default_embed_assets(content, base_path).await
    }

    fn filter_notes(&self, notes: &[crate::models::note::Note], filter_options: &FilterOptions) -> Vec<crate::models::note::Note> {
        self.base.apply_filters(notes, filter_options)
    }
}