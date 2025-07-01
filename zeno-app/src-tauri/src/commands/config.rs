use serde::{Deserialize, Serialize};
use tokio::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace_path: Option<String>,
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub sync_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace_path: None,
            theme: "light".to_string(),
            language: "zh-CN".to_string(),
            auto_save: true,
            sync_enabled: false,
        }
    }
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let content = fs::read_to_string(&config_path)
        .await
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))
}

#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path()?;
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .await
        .map_err(|e| format!("保存配置文件失败: {}", e))
}

#[tauri::command]
pub async fn select_workspace_directory() -> Result<Option<String>, String> {
    // 暂时返回默认路径，后续可以通过前端实现文件选择器
    let home = dirs::home_dir()
        .ok_or("无法获取用户主目录")?;
    
    let default_workspace = home.join("ZenoNotes");
    Ok(Some(default_workspace.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn create_workspace_directory(path: String) -> Result<(), String> {
    let workspace_path = PathBuf::from(&path);
    
    // 创建目录
    fs::create_dir_all(&workspace_path)
        .await
        .map_err(|e| format!("创建工作空间目录失败: {}", e))?;
    
    // 创建默认子目录
    let subdirs = ["daily", "projects", "reference", "templates", "attachments"];
    for subdir in &subdirs {
        let subdir_path = workspace_path.join(subdir);
        fs::create_dir_all(subdir_path)
            .await
            .map_err(|e| format!("创建子目录 {} 失败: {}", subdir, e))?;
    }
    
    // 创建 .zeno 目录（存放数据库等）
    let zeno_dir = workspace_path.join(".zeno");
    fs::create_dir_all(zeno_dir)
        .await
        .map_err(|e| format!("创建 .zeno 目录失败: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn validate_workspace_path(path: String) -> Result<bool, String> {
    let workspace_path = PathBuf::from(&path);
    
    if !workspace_path.exists() {
        return Ok(false);
    }
    
    if !workspace_path.is_dir() {
        return Ok(false);
    }
    
    // 检查是否有读写权限
    match fs::metadata(&workspace_path).await {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                return Ok(false);
            }
        }
        Err(_) => return Ok(false),
    }
    
    Ok(true)
}

fn get_config_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir()
        .ok_or("无法获取用户主目录")?;
    
    Ok(home.join(".zeno").join("config.json"))
}

mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}