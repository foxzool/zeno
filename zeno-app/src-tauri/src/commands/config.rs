use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace_path: Option<String>,
    pub theme: String,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace_path: None,
            theme: "light".to_string(),
            language: "zh-CN".to_string(),
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