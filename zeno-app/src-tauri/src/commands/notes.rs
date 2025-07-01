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
pub async fn create_note(title: Option<String>) -> Result<String, String> {
    // 从配置获取工作空间路径
    let config = crate::commands::get_config().await?;
    let workspace_path = config.workspace_path
        .ok_or("未设置工作空间路径")?;
    
    let workspace = Path::new(&workspace_path);
    if !workspace.exists() {
        return Err(format!("工作空间不存在: {}", workspace.display()));
    }
    
    let title = title.unwrap_or_else(|| "新建笔记".to_string());
    let filename = format!("{}.md", slugify(&title));
    let file_path = workspace.join(&filename);
    
    // 确保文件名唯一
    let mut counter = 1;
    let mut final_path = file_path.clone();
    while final_path.exists() {
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();
        let new_filename = format!("{}-{}.md", stem, counter);
        final_path = workspace.join(new_filename);
        counter += 1;
    }
    
    let content = format!("# {}\n\n", title);
    
    fs::write(&final_path, content)
        .await
        .map_err(|e| format!("创建文件失败: {}", e))?;
    
    Ok(final_path.to_string_lossy().to_string())
}

fn slugify(text: &str) -> String {
    text.trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                // 保留中文字符
                if c.is_alphabetic() {
                    c
                } else {
                    '-'
                }
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[tauri::command]
pub async fn delete_note(file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    
    if !path.is_file() {
        return Err(format!("路径不是文件: {}", path.display()));
    }
    
    // 检查文件扩展名确保只删除 Markdown 文件
    if let Some(ext) = path.extension() {
        if ext != "md" && ext != "markdown" {
            return Err(format!("只能删除 Markdown 文件 (.md, .markdown)"));
        }
    } else {
        return Err(format!("文件没有扩展名"));
    }
    
    fs::remove_file(path)
        .await
        .map_err(|e| format!("删除文件失败: {}", e))
}

#[tauri::command]
pub async fn show_in_folder(file_path: String) -> Result<(), String> {
    use std::process::Command;
    
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &file_path])
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &file_path])
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // 尝试使用 xdg-open 打开包含文件的目录
        if let Some(parent) = path.parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        } else {
            return Err("无法获取文件的父目录".to_string());
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