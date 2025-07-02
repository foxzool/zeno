use crate::models::tag::{HierarchicalTag, TagHierarchy, TagStatistics};
use std::sync::Mutex;
use tauri::{command, State};

/// 全局标签层次结构，使用 Mutex 保护
pub type GlobalTagHierarchy = Mutex<TagHierarchy>;

/// 解析和添加标签到层次结构
#[command]
pub async fn parse_and_add_tags(
    tag_strings: Vec<String>,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<String>, String> {
    let mut hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    let mut all_tags = Vec::new();

    for tag_str in tag_strings {
        let tags = hierarchy.parse_tag(&tag_str);
        all_tags.extend(tags);
    }

    Ok(all_tags)
}

/// 获取标签层次结构中的所有标签
#[command]
pub async fn get_all_tags(
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_all_tags().into_iter().cloned().collect())
}

/// 获取根标签
#[command]
pub async fn get_root_tags(
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_root_tags().into_iter().cloned().collect())
}

/// 获取标签的子标签
#[command]
pub async fn get_tag_children(
    tag_name: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_children(&tag_name).into_iter().cloned().collect())
}

/// 获取标签的祖先标签
#[command]
pub async fn get_tag_ancestors(
    tag_name: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_ancestors(&tag_name).into_iter().cloned().collect())
}

/// 获取标签的后代标签
#[command]
pub async fn get_tag_descendants(
    tag_name: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_descendants(&tag_name).into_iter().cloned().collect())
}

/// 搜索标签
#[command]
pub async fn search_tags(
    query: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.search_tags(&query).into_iter().cloned().collect())
}

/// 获取热门标签
#[command]
pub async fn get_popular_tags(
    limit: Option<usize>,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(10);
    Ok(hierarchy.get_popular_tags(limit).into_iter().cloned().collect())
}

/// 获取相关标签建议
#[command]
pub async fn suggest_related_tags(
    tag_name: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.suggest_related_tags(&tag_name).into_iter().cloned().collect())
}

/// 更新标签使用计数
#[command]
pub async fn update_tag_usage(
    tag_name: String,
    increment: bool,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<(), String> {
    let mut hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    
    if increment {
        hierarchy.increment_tag_usage(&tag_name);
    } else {
        hierarchy.decrement_tag_usage(&tag_name);
    }
    
    Ok(())
}

/// 获取特定标签的信息
#[command]
pub async fn get_tag_info(
    tag_name: String,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Option<HierarchicalTag>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_tag(&tag_name).cloned())
}

/// 获取标签统计信息
#[command]
pub async fn get_tag_statistics(
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<TagStatistics, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    Ok(hierarchy.get_statistics())
}

/// 清理未使用的标签
#[command]
pub async fn cleanup_unused_tags(
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<(), String> {
    let mut hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    hierarchy.cleanup_unused_tags();
    Ok(())
}

/// 重建标签层次结构（从笔记数据）
#[command]
pub async fn rebuild_tag_hierarchy(
    notes_tags: Vec<(String, Vec<String>)>, // (note_id, tags)
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<(), String> {
    let mut hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    
    // 重置层次结构
    *hierarchy = TagHierarchy::new();
    
    // 收集所有标签并构建层次结构
    for (_note_id, tags) in &notes_tags {
        for tag_str in tags {
            hierarchy.parse_tag(tag_str);
        }
    }
    
    // 更新使用计数
    for (_note_id, tags) in notes_tags {
        for tag_str in tags {
            // 为层次化标签的每一级都增加计数
            let parsed_tags = hierarchy.parse_tag(&tag_str);
            for tag_name in parsed_tags {
                hierarchy.increment_tag_usage(&tag_name);
            }
        }
    }
    
    Ok(())
}

/// 智能标签建议（基于内容）
#[command]
pub async fn suggest_tags_for_content(
    content: String,
    existing_tags: Vec<String>,
    tag_hierarchy: State<'_, GlobalTagHierarchy>,
) -> Result<Vec<String>, String> {
    let hierarchy = tag_hierarchy.lock().map_err(|e| e.to_string())?;
    let mut suggestions = Vec::new();
    
    // 简单的关键词匹配建议
    let content_lower = content.to_lowercase();
    let words: Vec<&str> = content_lower.split_whitespace().collect();
    
    // 检查现有标签是否匹配内容
    for tag in hierarchy.get_all_tags() {
        let tag_lower = tag.name.to_lowercase();
        let tag_parts: Vec<&str> = tag_lower.split('/').collect();
        
        // 检查标签的每个部分是否在内容中
        for part in &tag_parts {
            if words.iter().any(|word| word.contains(part) || part.contains(word)) {
                if !existing_tags.contains(&tag.name) && !suggestions.contains(&tag.name) {
                    suggestions.push(tag.name.clone());
                }
                break;
            }
        }
    }
    
    // 限制建议数量
    suggestions.truncate(10);
    suggestions.sort();
    
    Ok(suggestions)
}

/// 自动提取标签（从内容中的 #标签 格式）
#[command]
pub async fn extract_tags_from_content(
    content: String,
) -> Result<Vec<String>, String> {
    // 匹配 #标签 格式，支持中文、英文、数字、下划线、横线和斜杠
    let re = regex::Regex::new(r"#([\u4e00-\u9fa5\w\-/]+)")
        .map_err(|e| format!("正则表达式错误: {}", e))?;
    
    let mut tags = Vec::new();
    for cap in re.captures_iter(&content) {
        if let Some(tag) = cap.get(1) {
            let tag_str = tag.as_str().to_string();
            if !tags.contains(&tag_str) {
                tags.push(tag_str);
            }
        }
    }
    
    Ok(tags)
}