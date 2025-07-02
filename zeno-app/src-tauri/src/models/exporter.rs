use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub exporter_type: ExporterType,
    pub source_workspace: String,
    pub target_path: String,
    pub options: ExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum ExporterType {
    Markdown,
    Html,
    Pdf,
    Epub,
    Latex,
    Json,
    Zip,
    Obsidian,
    Notion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_attachments: bool,
    pub include_metadata: bool,
    pub preserve_structure: bool,
    pub convert_links: bool,
    pub include_tags: bool,
    pub filter_options: FilterOptions,
    pub format_options: FormatOptions,
    pub output_options: OutputOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub date_range: Option<DateRange>,
    pub tag_filter: Vec<String>,
    pub path_filter: Vec<String>,
    pub content_filter: Option<String>,
    pub exclude_drafts: bool,
    pub include_archived: bool,
    pub minimum_word_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOptions {
    pub page_size: Option<PageSize>,
    pub font_family: Option<String>,
    pub font_size: Option<u8>,
    pub line_height: Option<f32>,
    pub margin: Option<Margin>,
    pub header_footer: bool,
    pub table_of_contents: bool,
    pub syntax_highlighting: bool,
    pub math_rendering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    A3,
    A5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputOptions {
    pub compression: bool,
    pub encryption: bool,
    pub password: Option<String>,
    pub split_by_size: Option<u64>,
    pub naming_pattern: String,
    pub custom_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub exported_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub processing_time_ms: u64,
    pub output_files: Vec<ExportedFile>,
    pub total_size: u64,
    pub compression_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedFile {
    pub source_path: String,
    pub output_path: String,
    pub file_type: ExportFileType,
    pub original_size: u64,
    pub exported_size: u64,
    pub status: ExportStatus,
    pub transformations: Vec<ExportTransformation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFileType {
    Note,
    Index,
    Attachment,
    Stylesheet,
    Metadata,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportStatus {
    Success,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTransformation {
    pub transformation_type: ExportTransformationType,
    pub description: String,
    pub from_format: String,
    pub to_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportTransformationType {
    FormatConversion,
    LinkRewriting,
    AssetEmbedding,
    MetadataExtraction,
    ContentFiltering,
    StructureFlattening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    pub total_notes: u32,
    pub total_attachments: u32,
    pub estimated_size: u64,
    pub filtered_notes: u32,
    pub warnings: Vec<String>,
    pub structure: Vec<ExportDirectoryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDirectoryNode {
    pub name: String,
    pub path: String,
    pub note_count: u32,
    pub attachment_count: u32,
    pub children: Vec<ExportDirectoryNode>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_attachments: true,
            include_metadata: true,
            preserve_structure: true,
            convert_links: true,
            include_tags: true,
            filter_options: FilterOptions::default(),
            format_options: FormatOptions::default(),
            output_options: OutputOptions::default(),
        }
    }
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            date_range: None,
            tag_filter: Vec::new(),
            path_filter: Vec::new(),
            content_filter: None,
            exclude_drafts: false,
            include_archived: true,
            minimum_word_count: None,
        }
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            page_size: Some(PageSize::A4),
            font_family: Some("Times New Roman".to_string()),
            font_size: Some(12),
            line_height: Some(1.5),
            margin: Some(Margin {
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            }),
            header_footer: true,
            table_of_contents: true,
            syntax_highlighting: true,
            math_rendering: true,
        }
    }
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            compression: false,
            encryption: false,
            password: None,
            split_by_size: None,
            naming_pattern: "{title}-{date}".to_string(),
            custom_metadata: HashMap::new(),
        }
    }
}

impl ExportConfig {
    pub fn new(exporter_type: ExporterType, source_workspace: String, target_path: String) -> Self {
        Self {
            exporter_type,
            source_workspace,
            target_path,
            options: ExportOptions::default(),
        }
    }
}

impl ExportResult {
    pub fn new() -> Self {
        Self {
            success: false,
            exported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            processing_time_ms: 0,
            output_files: Vec::new(),
            total_size: 0,
            compression_ratio: None,
        }
    }

    pub fn total_processed(&self) -> u32 {
        self.exported_count + self.skipped_count + self.failed_count
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_processed() == 0 {
            return 0.0;
        }
        (self.exported_count as f64) / (self.total_processed() as f64) * 100.0
    }
}