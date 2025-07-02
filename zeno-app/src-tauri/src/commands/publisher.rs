use crate::models::publisher::*;
use crate::models::note::Note;
use crate::services::zola_publisher::ZolaPublisher;
use crate::services::note_service::NoteService;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{command, State};
use anyhow::Result;

/// 全局发布配置，使用 Mutex 保护
pub type GlobalPublishConfig = Mutex<PublishConfig>;

/// 初始化 Zola 站点
#[command]
pub async fn initialize_zola_site(
    site_path: String,
    config: ZolaConfig,
) -> Result<(), String> {
    let site_path = PathBuf::from(site_path);
    let publisher = ZolaPublisher::new(config, site_path)
        .map_err(|e| e.to_string())?;
    
    publisher.initialize_site().await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 发布笔记到静态网站
#[command]
pub async fn publish_notes_to_site(
    site_path: String,
    config: ZolaConfig,
    workspace_path: String,
) -> Result<PublishResult, String> {
    let site_path = PathBuf::from(site_path);
    let workspace_path = PathBuf::from(workspace_path);
    
    // 创建 ZolaPublisher
    let publisher = ZolaPublisher::new(config, site_path)
        .map_err(|e| e.to_string())?;
    
    // 获取所有笔记
    let note_service = NoteService::new(workspace_path);
    let notes = note_service.list_notes()
        .await
        .map_err(|e| e.to_string())?;
    
    // 发布笔记
    let result = publisher.publish_notes(notes).await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

/// 获取发布配置
#[command]
pub async fn get_publish_config(
    config: State<'_, GlobalPublishConfig>,
) -> Result<PublishConfig, String> {
    let config = config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// 保存发布配置
#[command]
pub async fn save_publish_config(
    new_config: PublishConfig,
    config: State<'_, GlobalPublishConfig>,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    *config = new_config;
    Ok(())
}

/// 创建默认的 Zola 配置
#[command]
pub async fn create_default_zola_config() -> Result<ZolaConfig, String> {
    Ok(ZolaConfig::default())
}

/// 检查 Zola 是否已安装
#[command]
pub async fn check_zola_installation() -> Result<bool, String> {
    use std::process::Command;
    
    match Command::new("zola").arg("--version").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// 获取站点统计信息
#[command]
pub async fn get_site_stats(
    site_path: String,
) -> Result<SiteStats, String> {
    let site_path = PathBuf::from(site_path);
    let content_dir = site_path.join("content");
    
    if !content_dir.exists() {
        return Ok(SiteStats {
            total_pages: 0,
            total_words: 0,
            total_tags: 0,
            total_categories: 0,
            last_build: None,
            build_time: None,
            site_size: 0,
        });
    }
    
    // 统计页面数量
    let mut total_pages = 0;
    let mut total_words = 0;
    let mut all_tags = std::collections::HashSet::new();
    let mut all_categories = std::collections::HashSet::new();
    
    // 递归遍历内容目录
    fn count_pages(
        dir: &std::path::Path,
        total_pages: &mut usize,
        total_words: &mut usize,
        all_tags: &mut std::collections::HashSet<String>,
        all_categories: &mut std::collections::HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                count_pages(&path, total_pages, total_words, all_tags, all_categories)?;
            } else if path.extension().map_or(false, |ext| ext == "md") {
                *total_pages += 1;
                
                // 读取文件内容并统计字数、标签等
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // 简单的字数统计
                    let word_count = content.split_whitespace().count();
                    *total_words += word_count;
                    
                    // 解析frontmatter来获取标签和分类
                    if let Some(frontmatter_end) = content.find("+++\n") {
                        if let Some(frontmatter_start) = content[frontmatter_end + 4..].find("+++\n") {
                            let frontmatter = &content[frontmatter_end + 4..frontmatter_end + 4 + frontmatter_start];
                            
                            // 简单的标签解析
                            if let Some(tags_start) = frontmatter.find("tags = [") {
                                if let Some(tags_end) = frontmatter[tags_start..].find(']') {
                                    let tags_str = &frontmatter[tags_start + 8..tags_start + tags_end];
                                    for tag in tags_str.split(',') {
                                        let tag = tag.trim().trim_matches('"').trim();
                                        if !tag.is_empty() {
                                            all_tags.insert(tag.to_string());
                                        }
                                    }
                                }
                            }
                            
                            // 简单的分类解析
                            if let Some(cats_start) = frontmatter.find("categories = [") {
                                if let Some(cats_end) = frontmatter[cats_start..].find(']') {
                                    let cats_str = &frontmatter[cats_start + 13..cats_start + cats_end];
                                    for cat in cats_str.split(',') {
                                        let cat = cat.trim().trim_matches('"').trim();
                                        if !cat.is_empty() {
                                            all_categories.insert(cat.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    count_pages(&content_dir, &mut total_pages, &mut total_words, &mut all_tags, &mut all_categories)
        .map_err(|e| e.to_string())?;
    
    // 获取最后构建时间
    let public_dir = site_path.join("public");
    let last_build = if public_dir.exists() {
        std::fs::metadata(&public_dir)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(|time| time.into())
    } else {
        None
    };
    
    // 计算站点大小
    let site_size = if public_dir.exists() {
        calculate_dir_size(&public_dir).unwrap_or(0)
    } else {
        0
    };
    
    Ok(SiteStats {
        total_pages,
        total_words,
        total_tags: all_tags.len(),
        total_categories: all_categories.len(),
        last_build,
        build_time: None, // 需要在构建过程中记录
        site_size,
    })
}

/// 预览发布内容
#[command]
pub async fn preview_publish_content(
    note_path: String,
    config: ZolaConfig,
) -> Result<String, String> {
    use crate::models::note::Frontmatter;
    use crate::services::note_service::NoteService;
    
    let note_path = PathBuf::from(note_path);
    let workspace_path = note_path.parent().unwrap();
    
    let note_service = NoteService::new(workspace_path.to_path_buf());
    let note = note_service.load_note(&note_path)
        .await
        .map_err(|e| e.to_string())?;
    
    // 创建临时的 ZolaPublisher 来进行内容转换
    let temp_site_path = std::env::temp_dir().join("zeno_preview");
    let publisher = ZolaPublisher::new(config, temp_site_path)
        .map_err(|e| e.to_string())?;
    
    // 转换为 Zola 格式
    let zola_content = publisher.convert_to_zola_format(&note).await
        .map_err(|e| e.to_string())?;
    
    Ok(zola_content)
}

/// 辅助函数：计算目录大小
fn calculate_dir_size(dir: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total_size = 0u64;
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_dir_size(&entry.path())?;
        }
    }
    
    Ok(total_size)
}