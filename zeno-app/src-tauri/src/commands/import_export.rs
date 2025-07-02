use crate::models::importer::*;
use crate::models::exporter::*;
use crate::services::ImportExportManager;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

// 全局导入导出管理器
type ImportExportManagerState = Arc<Mutex<ImportExportManager>>;

#[tauri::command]
pub async fn get_available_importers(
    manager: State<'_, ImportExportManagerState>,
) -> Result<Vec<ImporterType>, String> {
    let manager = manager.lock().await;
    Ok(manager.get_available_importers())
}

#[tauri::command]
pub async fn get_available_exporters(
    manager: State<'_, ImportExportManagerState>,
) -> Result<Vec<ExporterType>, String> {
    let manager = manager.lock().await;
    Ok(manager.get_available_exporters())
}

#[tauri::command]
pub async fn create_default_import_config(
    importer_type: ImporterType,
    source_path: String,
    target_workspace: String,
) -> Result<ImportConfig, String> {
    Ok(ImportConfig::new(importer_type, source_path, target_workspace))
}

#[tauri::command]
pub async fn create_default_export_config(
    exporter_type: ExporterType,
    source_workspace: String,
    target_path: String,
) -> Result<ExportConfig, String> {
    Ok(ExportConfig::new(exporter_type, source_workspace, target_path))
}

#[tauri::command]
pub async fn preview_import(
    config: ImportConfig,
    manager: State<'_, ImportExportManagerState>,
) -> Result<ImportPreview, String> {
    let manager = manager.lock().await;
    let preview = manager.preview_import(&config).await.map_err(|e| e.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub async fn execute_import(
    config: ImportConfig,
    manager: State<'_, ImportExportManagerState>,
) -> Result<ImportResult, String> {
    let manager = manager.lock().await;
    let result = manager.import(&config).await.map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub async fn preview_export(
    config: ExportConfig,
    manager: State<'_, ImportExportManagerState>,
) -> Result<ExportPreview, String> {
    let manager = manager.lock().await;
    let preview = manager.preview_export(&config).await.map_err(|e| e.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub async fn execute_export(
    config: ExportConfig,
    manager: State<'_, ImportExportManagerState>,
) -> Result<ExportResult, String> {
    let manager = manager.lock().await;
    let result = manager.export(&config).await.map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub async fn validate_import_source(
    importer_type: ImporterType,
    source_path: String,
    manager: State<'_, ImportExportManagerState>,
) -> Result<bool, String> {
    let _manager = manager.lock().await;
    let config = ImportConfig::new(importer_type, source_path, String::new());
    
    // 简化验证：直接检查路径是否存在
    let path = std::path::Path::new(&config.source_path);
    Ok(path.exists())
}

#[tauri::command]
pub async fn validate_export_target(
    exporter_type: ExporterType,
    target_path: String,
    manager: State<'_, ImportExportManagerState>,
) -> Result<bool, String> {
    let _manager = manager.lock().await;
    let config = ExportConfig::new(exporter_type, String::new(), target_path);
    
    // 简化验证：检查目标目录是否可访问
    let path = std::path::Path::new(&config.target_path);
    if let Some(parent) = path.parent() {
        Ok(parent.exists() || std::fs::create_dir_all(parent).is_ok())
    } else {
        Ok(true)
    }
}

#[tauri::command]
pub async fn get_import_conflicts(
    config: ImportConfig,
    manager: State<'_, ImportExportManagerState>,
) -> Result<Vec<FileConflict>, String> {
    let manager = manager.lock().await;
    let preview = manager.preview_import(&config).await.map_err(|e| e.to_string())?;
    Ok(preview.conflicts)
}

#[tauri::command]
pub async fn create_import_options() -> Result<ImportOptions, String> {
    Ok(ImportOptions::default())
}

#[tauri::command]
pub async fn create_export_options() -> Result<ExportOptions, String> {
    Ok(ExportOptions::default())
}

#[tauri::command]
pub async fn get_supported_import_formats() -> Result<Vec<String>, String> {
    Ok(vec![
        "Markdown".to_string(),
        "Obsidian".to_string(),
        "Notion".to_string(),
        "Roam Research".to_string(),
        "LogSeq".to_string(),
        "Generic Text".to_string(),
    ])
}

#[tauri::command]
pub async fn get_supported_export_formats() -> Result<Vec<String>, String> {
    Ok(vec![
        "Markdown".to_string(),
        "HTML".to_string(),
        "PDF".to_string(),
        "EPUB".to_string(),
        "LaTeX".to_string(),
        "JSON".to_string(),
        "ZIP Archive".to_string(),
    ])
}

#[tauri::command]
pub async fn scan_import_directory(source_path: String) -> Result<ImportPreview, String> {
    let path = std::path::Path::new(&source_path);
    if !path.exists() {
        return Err("Source directory does not exist".to_string());
    }

    let mut total_files = 0u32;
    let mut notes = 0u32;
    let mut attachments = 0u32;
    let mut media_files = 0u32;
    let mut estimated_size = 0u64;

    fn scan_recursive(
        dir: &std::path::Path,
        totals: &mut (u32, u32, u32, u32, u64)
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                scan_recursive(&path, totals)?;
            } else {
                totals.0 += 1; // total_files
                
                let metadata = std::fs::metadata(&path)?;
                totals.4 += metadata.len(); // estimated_size
                
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        match ext_str.to_lowercase().as_str() {
                            "md" | "markdown" | "txt" => totals.1 += 1, // notes
                            "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" => totals.3 += 1, // media_files
                            _ => totals.2 += 1, // attachments
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let mut totals = (0u32, 0u32, 0u32, 0u32, 0u64);
    scan_recursive(path, &mut totals).map_err(|e| e.to_string())?;
    
    (total_files, notes, attachments, media_files, estimated_size) = totals;

    Ok(ImportPreview {
        total_files,
        notes,
        attachments,
        media_files,
        estimated_size,
        warnings: Vec::new(),
        conflicts: Vec::new(),
        structure: Vec::new(),
    })
}

#[tauri::command]
pub async fn estimate_export_size(
    workspace_path: String,
    filter_options: Option<FilterOptions>,
) -> Result<ExportPreview, String> {
    let path = std::path::Path::new(&workspace_path);
    if !path.exists() {
        return Err("Workspace directory does not exist".to_string());
    }

    let mut total_notes = 0u32;
    let mut total_attachments = 0u32;
    let mut estimated_size = 0u64;

    fn scan_workspace_recursive(
        dir: &std::path::Path,
        totals: &mut (u32, u32, u64)
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                scan_workspace_recursive(&path, totals)?;
            } else {
                let metadata = std::fs::metadata(&path)?;
                totals.2 += metadata.len(); // estimated_size
                
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        match ext_str.to_lowercase().as_str() {
                            "md" | "markdown" | "txt" => totals.0 += 1, // total_notes
                            _ => totals.1 += 1, // total_attachments
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let mut totals = (0u32, 0u32, 0u64);
    scan_workspace_recursive(path, &mut totals).map_err(|e| e.to_string())?;
    
    (total_notes, total_attachments, estimated_size) = totals;

    // 应用过滤器会减少导出的文件数量
    let filtered_notes = if filter_options.is_some() {
        (total_notes as f32 * 0.8) as u32 // 假设过滤器会减少20%的文件
    } else {
        total_notes
    };

    Ok(ExportPreview {
        total_notes,
        total_attachments,
        estimated_size,
        filtered_notes,
        warnings: Vec::new(),
        structure: Vec::new(),
    })
}