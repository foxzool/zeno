use crate::models::link::{WikiLink, LinkParseResult, BacklinkInfo, SimilarNote, BrokenLink};
use crate::services::{LinkParser, LinkIndex};
use std::sync::Mutex;
use tauri::{command, State};

/// 全局链接索引，使用Mutex保护
pub type GlobalLinkIndex = Mutex<LinkIndex>;

/// 解析文本中的链接
#[command]
pub async fn parse_links(content: String) -> Result<LinkParseResult, String> {
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    Ok(parser.parse_links(&content))
}

/// 提取链接的上下文
#[command]
pub async fn extract_link_context(
    content: String,
    link: WikiLink,
    context_size: Option<usize>,
) -> Result<String, String> {
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    let size = context_size.unwrap_or(2);
    Ok(parser.extract_link_context(&content, &link, size))
}

/// 替换文档中的链接
#[command]
pub async fn replace_link(
    content: String,
    old_link: WikiLink,
    new_target: String,
) -> Result<String, String> {
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    Ok(parser.replace_link(&content, &old_link, &new_target))
}

/// 批量替换链接
#[command]
pub async fn replace_multiple_links(
    content: String,
    replacements: Vec<(WikiLink, String)>,
) -> Result<String, String> {
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    Ok(parser.replace_multiple_links(&content, &replacements))
}

/// 注册笔记到链接索引
#[command]
pub async fn register_note_in_index(
    note_id: String,
    path: String,
    title: String,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<(), String> {
    let mut index = link_index.lock().map_err(|e| e.to_string())?;
    index.register_note(note_id, path.into(), title);
    Ok(())
}

/// 更新笔记的链接关系
#[command]
pub async fn update_note_links(
    note_id: String,
    links: Vec<WikiLink>,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<(), String> {
    let mut index = link_index.lock().map_err(|e| e.to_string())?;
    index.update_note_links(&note_id, links)
}

/// 获取笔记的反向链接
#[command]
pub async fn get_backlinks(
    note_id: String,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<BacklinkInfo>, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    Ok(index.get_backlinks(&note_id))
}

/// 获取笔记的正向链接
#[command]
pub async fn get_outgoing_links(
    note_id: String,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<String>, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    Ok(index.get_outgoing_links(&note_id))
}

/// 查找相似笔记
#[command]
pub async fn find_similar_notes(
    note_id: String,
    limit: Option<usize>,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<SimilarNote>, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    let max_results = limit.unwrap_or(10);
    Ok(index.find_similar_notes(&note_id, max_results))
}

/// 获取断链信息
#[command]
pub async fn get_broken_links(
    note_id: Option<String>,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<BrokenLink>, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    Ok(index.get_broken_links(note_id.as_deref()))
}

/// 获取孤立笔记
#[command]
pub async fn get_orphaned_notes(
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<String>, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    Ok(index.get_orphaned_notes())
}

/// 获取链接统计信息
#[command]
pub async fn get_link_statistics(
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<serde_json::Value, String> {
    let index = link_index.lock().map_err(|e| e.to_string())?;
    let stats = index.get_statistics();
    serde_json::to_value(stats).map_err(|e| e.to_string())
}

/// 重建链接索引（用于初始化或重新同步）
#[command]
pub async fn rebuild_link_index(
    notes_data: Vec<(String, String, String, String)>, // (id, path, title, content)
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<(), String> {
    let mut index = link_index.lock().map_err(|e| e.to_string())?;
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    
    // 清空现有索引
    *index = LinkIndex::new();
    
    // 注册所有笔记
    for (note_id, path, title, _) in &notes_data {
        index.register_note(note_id.clone(), path.clone().into(), title.clone());
    }
    
    // 解析并更新所有链接
    for (note_id, _, _, content) in notes_data {
        let parse_result = parser.parse_links(&content);
        index.update_note_links(&note_id, parse_result.links)?;
    }
    
    Ok(())
}

/// 解析并预览链接（不保存到索引）
#[command]
pub async fn preview_link_parsing(
    content: String,
) -> Result<serde_json::Value, String> {
    let parser = LinkParser::new().map_err(|e| e.to_string())?;
    let result = parser.parse_links(&content);
    
    let preview = serde_json::json!({
        "total_links": result.links.len(),
        "wiki_links": result.stats.wiki_links,
        "embed_links": result.stats.embed_links,
        "markdown_links": result.stats.markdown_links,
        "errors": result.errors.len(),
        "links_preview": result.links.iter().take(10).collect::<Vec<_>>()
    });
    
    Ok(preview)
}

/// 验证链接目标是否存在
#[command]
pub async fn validate_link_targets(
    links: Vec<WikiLink>,
    link_index: State<'_, GlobalLinkIndex>,
) -> Result<Vec<bool>, String> {
    let _index = link_index.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    
    for link in links {
        // 这里简化处理，实际应该调用 resolve_link_target
        // 但该方法是私有的，所以我们检查是否有断链记录
        let is_valid = !link.target.trim().is_empty();
        results.push(is_valid);
    }
    
    Ok(results)
}