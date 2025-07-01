use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: Option<String>,
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))
}

#[tauri::command]
pub async fn write_file_content(path: String, content: String) -> Result<(), String> {
    let path = Path::new(&path);
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    fs::write(path, content)
        .await
        .map_err(|e| format!("写入文件失败: {}", e))
}

#[tauri::command]
pub async fn list_notes(dir_path: String) -> Result<Vec<NoteFile>, String> {
    let dir = Path::new(&dir_path);
    if !dir.is_dir() {
        return Err(format!("路径不是目录: {}", dir.display()));
    }
    
    let mut notes = Vec::new();
    let mut entries = fs::read_dir(dir)
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?;
    
    while let Some(entry) = entries.next_entry()
        .await
        .map_err(|e| format!("遍历目录失败: {}", e))? {
        
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" || ext == "markdown" {
                    let metadata = entry.metadata()
                        .await
                        .map_err(|e| format!("获取文件元数据失败: {}", e))?;
                    
                    let modified = metadata.modified()
                        .ok()
                        .map(|time| format!("{:?}", time));
                    
                    notes.push(NoteFile {
                        path: path.to_string_lossy().to_string(),
                        name: path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        size: metadata.len(),
                        modified,
                    });
                }
            }
        }
    }
    
    Ok(notes)
}

#[tauri::command]
pub async fn parse_markdown(content: String, file_path: Option<String>) -> Result<serde_json::Value, String> {
    use zeno_core::parser::MarkdownParser;
    use std::path::PathBuf;
    
    let parser = MarkdownParser::new();
    let path = file_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("untitled.md"));
    
    let parsed = parser.parse(&content, path)
        .map_err(|e| format!("Markdown解析失败: {}", e))?;
    
    serde_json::to_value(parsed)
        .map_err(|e| format!("序列化失败: {}", e))
}