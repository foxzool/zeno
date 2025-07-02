use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    pub importer_type: ImporterType,
    pub source_path: String,
    pub target_workspace: String,
    pub options: ImportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum ImporterType {
    Obsidian,
    Notion,
    Markdown,
    Roam,
    LogSeq,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub preserve_structure: bool,
    pub merge_mode: MergeMode,
    pub include_attachments: bool,
    pub convert_links: bool,
    pub convert_tags: bool,
    pub dry_run: bool,
    pub skip_duplicates: bool,
    pub backup_existing: bool,
    pub custom_mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeMode {
    Overwrite,
    Skip,
    Merge,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub processing_time_ms: u64,
    pub imported_files: Vec<ImportedFile>,
    pub duplicate_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedFile {
    pub source_path: String,
    pub target_path: String,
    pub file_type: FileType,
    pub size: u64,
    pub modified_time: DateTime<Utc>,
    pub status: ImportStatus,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Note,
    Attachment,
    Media,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportStatus {
    Success,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub transformation_type: TransformationType,
    pub description: String,
    pub from_value: String,
    pub to_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    LinkConversion,
    TagConversion,
    FrontmatterConversion,
    PathRewrite,
    ContentReformat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub total_files: u32,
    pub notes: u32,
    pub attachments: u32,
    pub media_files: u32,
    pub estimated_size: u64,
    pub warnings: Vec<String>,
    pub conflicts: Vec<FileConflict>,
    pub structure: Vec<DirectoryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflict {
    pub source_path: String,
    pub target_path: String,
    pub conflict_type: ConflictType,
    pub suggested_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    NameCollision,
    ContentMismatch,
    SizeDiscrepancy,
    TimestampConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub name: String,
    pub path: String,
    pub file_count: u32,
    pub children: Vec<DirectoryNode>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            preserve_structure: true,
            merge_mode: MergeMode::Skip,
            include_attachments: true,
            convert_links: true,
            convert_tags: true,
            dry_run: false,
            skip_duplicates: true,
            backup_existing: true,
            custom_mappings: HashMap::new(),
        }
    }
}

impl ImportConfig {
    pub fn new(importer_type: ImporterType, source_path: String, target_workspace: String) -> Self {
        Self {
            importer_type,
            source_path,
            target_workspace,
            options: ImportOptions::default(),
        }
    }
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            success: false,
            imported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            processing_time_ms: 0,
            imported_files: Vec::new(),
            duplicate_files: Vec::new(),
        }
    }

    pub fn total_processed(&self) -> u32 {
        self.imported_count + self.skipped_count + self.failed_count
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_processed() == 0 {
            return 0.0;
        }
        (self.imported_count as f64) / (self.total_processed() as f64) * 100.0
    }
}