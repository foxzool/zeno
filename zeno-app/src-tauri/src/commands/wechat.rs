use crate::models::wechat::*;
use crate::services::wechat_publisher::WeChatPublisher;
use crate::services::note_service::NoteService;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{command, State};
use anyhow::Result;

/// 全局微信配置，使用 Mutex 保护
pub type GlobalWeChatConfig = Mutex<WeChatConfig>;

/// 测试微信公众号配置
#[command]
pub async fn test_wechat_config(config: WeChatConfig) -> Result<bool, String> {
    let mut publisher = WeChatPublisher::new(config);
    
    publisher.test_configuration()
        .await
        .map_err(|e| e.to_string())
}

/// 获取微信公众号配置
#[command]
pub async fn get_wechat_config(
    config: State<'_, GlobalWeChatConfig>,
) -> Result<WeChatConfig, String> {
    let config = config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// 保存微信公众号配置
#[command]
pub async fn save_wechat_config(
    new_config: WeChatConfig,
    config: State<'_, GlobalWeChatConfig>,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    *config = new_config;
    Ok(())
}

/// 创建默认微信配置
#[command]
pub async fn create_default_wechat_config() -> Result<WeChatConfig, String> {
    Ok(WeChatConfig::default())
}

/// 发布单篇笔记到微信公众号
#[command]
pub async fn publish_note_to_wechat(
    note_path: String,
    config: WeChatConfig,
    settings: Option<WeChatPublishSettings>,
) -> Result<WeChatPublishResult, String> {
    let note_path = PathBuf::from(note_path);
    let workspace_path = note_path.parent()
        .ok_or_else(|| "Invalid note path".to_string())?;
    
    let note_service = NoteService::new(workspace_path.to_path_buf());
    let note = note_service.load_note(&note_path)
        .await
        .map_err(|e| e.to_string())?;
    
    let mut publisher = WeChatPublisher::new(config);
    let result = publisher.publish_note(&note, settings)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

/// 批量发布笔记到微信公众号
#[command]
pub async fn publish_notes_to_wechat(
    workspace_path: String,
    config: WeChatConfig,
    settings: Option<WeChatPublishSettings>,
    note_paths: Option<Vec<String>>,
) -> Result<Vec<WeChatPublishResult>, String> {
    let workspace_path = PathBuf::from(workspace_path);
    let note_service = NoteService::new(workspace_path);
    
    // 获取要发布的笔记
    let notes = if let Some(paths) = note_paths {
        let mut selected_notes = Vec::new();
        for path in paths {
            let note_path = PathBuf::from(path);
            match note_service.load_note(&note_path).await {
                Ok(note) => selected_notes.push(note),
                Err(e) => log::warn!("Failed to load note {}: {}", note_path.display(), e),
            }
        }
        selected_notes
    } else {
        // 获取所有笔记
        note_service.list_notes()
            .await
            .map_err(|e| e.to_string())?
    };
    
    let note_refs: Vec<&_> = notes.iter().collect();
    let mut publisher = WeChatPublisher::new(config);
    let results = publisher.publish_notes(note_refs, settings)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(results)
}

/// 预览笔记的微信转换格式
#[command]
pub async fn preview_wechat_content(
    note_path: String,
    config: WeChatConfig,
) -> Result<String, String> {
    let note_path = PathBuf::from(note_path);
    let workspace_path = note_path.parent()
        .ok_or_else(|| "Invalid note path".to_string())?;
    
    let note_service = NoteService::new(workspace_path.to_path_buf());
    let note = note_service.load_note(&note_path)
        .await
        .map_err(|e| e.to_string())?;
    
    let publisher = WeChatPublisher::new(config);
    let converted_content = publisher.preview_converted_content(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(converted_content)
}

/// 获取微信公众号统计信息
#[command]
pub async fn get_wechat_stats(config: WeChatConfig) -> Result<WeChatStats, String> {
    let publisher = WeChatPublisher::new(config);
    let stats = publisher.get_stats()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(stats)
}

/// 刷新微信访问令牌
#[command]
pub async fn refresh_wechat_token(
    config: State<'_, GlobalWeChatConfig>,
) -> Result<String, String> {
    let current_config = {
        let config_guard = config.lock().map_err(|e| e.to_string())?;
        config_guard.clone()
    };
    
    let mut publisher = WeChatPublisher::new(current_config);
    
    // 强制刷新令牌
    match publisher.test_configuration().await {
        Ok(_) => {
            // 更新全局配置中的令牌信息
            // 注意：这里简化处理，实际应该从 publisher 中获取更新后的配置
            Ok("Token refreshed successfully".to_string())
        }
        Err(e) => Err(format!("Failed to refresh token: {}", e)),
    }
}

/// 上传媒体文件到微信
#[command]
pub async fn upload_media_to_wechat(
    file_path: String,
    media_type: String,
    config: WeChatConfig,
) -> Result<MediaInfo, String> {
    let file_data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let media_type_enum = match media_type.to_lowercase().as_str() {
        "image" => MediaType::Image,
        "video" => MediaType::Video,
        "voice" => MediaType::Voice,
        "thumb" => MediaType::Thumb,
        _ => return Err("Invalid media type".to_string()),
    };
    
    let publisher = WeChatPublisher::new(config);
    let media_info = publisher.upload_media(&file_data, &filename, media_type_enum)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(media_info)
}

/// 创建默认微信发布设置
#[command]
pub async fn create_default_wechat_settings() -> Result<WeChatPublishSettings, String> {
    Ok(WeChatPublishSettings::default())
}

/// 验证微信内容格式
#[command]
pub async fn validate_wechat_content(content: String) -> Result<ValidationResult, String> {
    let validation_result = validate_content_for_wechat(&content);
    Ok(validation_result)
}

/// 内容验证结果
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
    pub content_stats: ContentStats,
}

/// 内容统计信息
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentStats {
    pub character_count: usize,
    pub word_count: usize,
    pub image_count: usize,
    pub link_count: usize,
    pub estimated_read_time: u32,
}

/// 验证内容是否适合微信发布
fn validate_content_for_wechat(content: &str) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut suggestions = Vec::new();

    let character_count = content.chars().count();
    let word_count = content.split_whitespace().count();
    
    // 统计图片数量
    let image_regex = regex::Regex::new(r"<img[^>]*>|!\[[^\]]*\]\([^)]*\)").unwrap();
    let image_count = image_regex.find_iter(content).count();
    
    // 统计链接数量
    let link_regex = regex::Regex::new(r"<a[^>]*>|(?<!\!)\[[^\]]*\]\([^)]*\)").unwrap();
    let link_count = link_regex.find_iter(content).count();
    
    // 估算阅读时间（按每分钟 200 字计算）
    let estimated_read_time = ((character_count as f32 / 200.0).ceil() as u32).max(1);

    // 内容长度检查
    if character_count > 20000 {
        errors.push("内容过长，微信公众号文章建议不超过 20,000 字".to_string());
    } else if character_count > 15000 {
        warnings.push("内容较长，建议考虑分拆成多篇文章".to_string());
    }

    if character_count < 100 {
        warnings.push("内容过短，建议增加更多内容以提升阅读体验".to_string());
    }

    // 图片数量检查
    if image_count > 30 {
        warnings.push("图片数量较多，可能影响加载速度".to_string());
    }

    // 链接检查
    if link_count > 10 {
        warnings.push("外部链接较多，微信公众号对外链有限制".to_string());
    }

    // 提供改进建议
    if image_count == 0 {
        suggestions.push("考虑添加一些图片来提升文章的视觉效果".to_string());
    }

    if content.contains("```") {
        suggestions.push("代码块将被转换为样式化的文本框".to_string());
    }

    if content.contains("$$") || content.contains("$") {
        suggestions.push("数学公式将被转换为文本描述".to_string());
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        warnings,
        errors,
        suggestions,
        content_stats: ContentStats {
            character_count,
            word_count,
            image_count,
            link_count,
            estimated_read_time,
        },
    }
}