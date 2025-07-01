use crate::models::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace_path: Option<PathBuf>,
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

pub struct ConfigService {
    config_path: PathBuf,
}

impl ConfigService {
    pub fn new() -> Result<Self, AppError> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.json");
        
        Ok(Self { config_path })
    }
    
    pub async fn load_config(&self) -> Result<AppConfig, AppError> {
        if !self.config_path.exists() {
            let default_config = AppConfig::default();
            self.save_config(&default_config).await?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&self.config_path).await?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub async fn save_config(&self, config: &AppConfig) -> Result<(), AppError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }
    
    fn get_config_dir() -> Result<PathBuf, AppError> {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::ConfigError("无法获取用户主目录".to_string()))?;
        
        Ok(PathBuf::from(home).join(".zeno"))
    }
}