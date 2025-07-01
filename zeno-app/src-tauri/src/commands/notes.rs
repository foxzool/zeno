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
    if !dir.exists() {
        return Err(format!("目录不存在: {}", dir.display()));
    }
    
    if !dir.is_dir() {
        return Err(format!("路径不是目录: {}", dir.display()));
    }
    
    let mut notes = Vec::new();
    collect_notes_iterative(dir, &mut notes).await?;
    
    // 按修改时间排序（最新的在前）
    notes.sort_by(|a, b| {
        let a_time = a.modified.as_ref().and_then(|s| s.parse::<u64>().ok());
        let b_time = b.modified.as_ref().and_then(|s| s.parse::<u64>().ok());
        b_time.cmp(&a_time)
    });
    
    Ok(notes)
}

async fn collect_notes_iterative(dir: &Path, notes: &mut Vec<NoteFile>) -> Result<(), String> {
    use std::collections::VecDeque;
    
    let mut dirs_to_search = VecDeque::new();
    dirs_to_search.push_back(dir.to_path_buf());
    
    while let Some(current_dir) = dirs_to_search.pop_front() {
        let mut entries = fs::read_dir(&current_dir)
            .await
            .map_err(|e| format!("读取目录失败 {}: {}", current_dir.display(), e))?;
        
        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| format!("遍历目录失败: {}", e))? {
            
            let path = entry.path();
            
            if path.is_dir() {
                // 跳过隐藏目录和 .zeno 目录
                if let Some(dir_name) = path.file_name() {
                    let dir_name_str = dir_name.to_string_lossy();
                    if !dir_name_str.starts_with('.') {
                        dirs_to_search.push_back(path);
                    }
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "markdown" {
                        let metadata = entry.metadata()
                            .await
                            .map_err(|e| format!("获取文件元数据失败: {}", e))?;
                        
                        let modified = metadata.modified()
                            .ok()
                            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|duration| duration.as_secs().to_string());
                        
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
    }
    
    Ok(())
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