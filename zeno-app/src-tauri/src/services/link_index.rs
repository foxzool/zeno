use crate::models::link::{WikiLink, BacklinkInfo, SimilarNote, BrokenLink, LinkType};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 链接索引和管理系统
#[derive(Debug, Clone)]
pub struct LinkIndex {
    /// 正向链接映射: 笔记ID -> 链接目标集合
    outgoing_links: HashMap<String, HashSet<String>>,
    /// 反向链接映射: 笔记ID -> 引用源集合
    incoming_links: HashMap<String, HashSet<String>>,
    /// 笔记路径到ID的映射
    path_to_id: HashMap<PathBuf, String>,
    /// 笔记ID到路径的映射
    id_to_path: HashMap<String, PathBuf>,
    /// 笔记标题到ID的映射
    title_to_id: HashMap<String, String>,
    /// 断链集合: 源笔记ID -> 断链列表
    broken_links: HashMap<String, Vec<WikiLink>>,
    /// 孤立笔记集合
    orphaned_notes: HashSet<String>,
}

impl LinkIndex {
    pub fn new() -> Self {
        Self {
            outgoing_links: HashMap::new(),
            incoming_links: HashMap::new(),
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
            title_to_id: HashMap::new(),
            broken_links: HashMap::new(),
            orphaned_notes: HashSet::new(),
        }
    }

    /// 注册一个笔记
    pub fn register_note(&mut self, note_id: String, path: PathBuf, title: String) {
        self.path_to_id.insert(path.clone(), note_id.clone());
        self.id_to_path.insert(note_id.clone(), path);
        self.title_to_id.insert(title, note_id.clone());
        
        // 初始化链接集合
        self.outgoing_links.entry(note_id.clone()).or_insert_with(HashSet::new);
        self.incoming_links.entry(note_id.clone()).or_insert_with(HashSet::new);
    }

    /// 移除笔记注册
    pub fn unregister_note(&mut self, note_id: &str) {
        if let Some(path) = self.id_to_path.remove(note_id) {
            self.path_to_id.remove(&path);
        }
        
        // 清理标题映射
        self.title_to_id.retain(|_, id| id != note_id);
        
        // 清理链接关系
        if let Some(outgoing) = self.outgoing_links.remove(note_id) {
            for target_id in outgoing {
                if let Some(incoming) = self.incoming_links.get_mut(&target_id) {
                    incoming.remove(note_id);
                }
            }
        }
        
        if let Some(incoming) = self.incoming_links.remove(note_id) {
            for source_id in incoming {
                if let Some(outgoing) = self.outgoing_links.get_mut(&source_id) {
                    outgoing.remove(note_id);
                }
            }
        }
        
        // 清理断链和孤立状态
        self.broken_links.remove(note_id);
        self.orphaned_notes.remove(note_id);
    }

    /// 更新笔记的链接关系
    pub fn update_note_links(&mut self, note_id: &str, links: Vec<WikiLink>) -> Result<(), String> {
        // 清理旧的链接关系
        self.clear_note_links(note_id);
        
        let mut resolved_targets = Vec::new();
        let mut broken_links = Vec::new();
        
        // 解析每个链接
        for link in links {
            match self.resolve_link_target(&link) {
                Some(target_id) => {
                    self.add_link_relationship(note_id, &target_id);
                    resolved_targets.push(target_id);
                }
                None => {
                    broken_links.push(link);
                }
            }
        }
        
        // 更新断链记录
        if !broken_links.is_empty() {
            self.broken_links.insert(note_id.to_string(), broken_links);
        }
        
        // 更新孤立状态
        self.update_orphaned_status();
        
        Ok(())
    }

    /// 清理笔记的所有链接关系
    fn clear_note_links(&mut self, note_id: &str) {
        if let Some(outgoing) = self.outgoing_links.get(note_id).cloned() {
            for target_id in outgoing {
                if let Some(incoming) = self.incoming_links.get_mut(&target_id) {
                    incoming.remove(note_id);
                }
            }
        }
        
        self.outgoing_links.insert(note_id.to_string(), HashSet::new());
        self.broken_links.remove(note_id);
    }

    /// 添加链接关系
    fn add_link_relationship(&mut self, source_id: &str, target_id: &str) {
        self.outgoing_links
            .entry(source_id.to_string())
            .or_default()
            .insert(target_id.to_string());
        
        self.incoming_links
            .entry(target_id.to_string())
            .or_default()
            .insert(source_id.to_string());
    }

    /// 解析链接目标到笔记ID
    fn resolve_link_target(&self, link: &WikiLink) -> Option<String> {
        // 1. 尝试精确匹配标题
        if let Some(note_id) = self.title_to_id.get(&link.target) {
            return Some(note_id.clone());
        }
        
        // 2. 尝试路径匹配
        let target_path = Path::new(&link.target);
        if let Some(note_id) = self.path_to_id.get(target_path) {
            return Some(note_id.clone());
        }
        
        // 3. 尝试相对路径匹配
        for (path, note_id) in &self.path_to_id {
            if path.file_name() == target_path.file_name() {
                return Some(note_id.clone());
            }
            
            // 检查路径结尾是否匹配
            if path.to_string_lossy().ends_with(&link.target) {
                return Some(note_id.clone());
            }
        }
        
        // 4. 尝试模糊匹配标题
        let target_lower = link.target.to_lowercase();
        for (title, note_id) in &self.title_to_id {
            if title.to_lowercase().contains(&target_lower) {
                return Some(note_id.clone());
            }
        }
        
        None
    }

    /// 获取笔记的反向链接
    pub fn get_backlinks(&self, note_id: &str) -> Vec<BacklinkInfo> {
        let mut backlinks = Vec::new();
        
        if let Some(incoming) = self.incoming_links.get(note_id) {
            for source_id in incoming {
                if let Some(path) = self.id_to_path.get(source_id) {
                    backlinks.push(BacklinkInfo {
                        source_note_id: source_id.clone(),
                        source_note_title: self.get_note_title(source_id).unwrap_or_default(),
                        source_note_path: path.to_string_lossy().to_string(),
                        context: "".to_string(), // 这里需要从实际内容中提取
                        line_number: 0, // 这里需要从实际内容中计算
                        link_type: LinkType::Wiki,
                        occurrence_count: 1, // 这里需要计算实际出现次数
                    });
                }
            }
        }
        
        backlinks
    }

    /// 获取笔记的正向链接
    pub fn get_outgoing_links(&self, note_id: &str) -> Vec<String> {
        self.outgoing_links
            .get(note_id)
            .map(|links| links.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 查找相似笔记
    pub fn find_similar_notes(&self, note_id: &str, limit: usize) -> Vec<SimilarNote> {
        let mut similarities = Vec::new();
        
        let empty_outgoing = HashSet::new();
        let empty_incoming = HashSet::new();
        let outgoing = self.outgoing_links.get(note_id).unwrap_or(&empty_outgoing);
        let incoming = self.incoming_links.get(note_id).unwrap_or(&empty_incoming);
        
        for (other_id, other_outgoing) in &self.outgoing_links {
            if other_id == note_id {
                continue;
            }
            
            let empty_other_incoming = HashSet::new();
            let other_incoming = self.incoming_links.get(other_id).unwrap_or(&empty_other_incoming);
            
            // 计算基于共同出链的相似度
            let common_outgoing = outgoing.intersection(other_outgoing).count();
            let total_outgoing = outgoing.union(other_outgoing).count();
            
            // 计算基于共同入链的相似度
            let common_incoming = incoming.intersection(other_incoming).count();
            let total_incoming = incoming.union(other_incoming).count();
            
            // 综合相似度计算
            let outgoing_sim = if total_outgoing > 0 {
                common_outgoing as f64 / total_outgoing as f64
            } else {
                0.0
            };
            
            let incoming_sim = if total_incoming > 0 {
                common_incoming as f64 / total_incoming as f64
            } else {
                0.0
            };
            
            let similarity = (outgoing_sim + incoming_sim) / 2.0;
            
            if similarity > 0.1 {
                let common_links: Vec<String> = outgoing
                    .intersection(other_outgoing)
                    .chain(incoming.intersection(other_incoming))
                    .cloned()
                    .collect();
                
                let common_links_count = common_links.len();
                
                similarities.push(SimilarNote {
                    note_id: other_id.clone(),
                    title: self.get_note_title(other_id).unwrap_or_default(),
                    path: self.id_to_path
                        .get(other_id)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    similarity_score: similarity,
                    common_links,
                    similarity_reason: format!("共同链接: {}", common_links_count),
                });
            }
        }
        
        // 按相似度排序
        similarities.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        similarities.truncate(limit);
        
        similarities
    }

    /// 获取断链信息
    pub fn get_broken_links(&self, note_id: Option<&str>) -> Vec<BrokenLink> {
        let mut broken_links = Vec::new();
        
        let sources = if let Some(id) = note_id {
            vec![id]
        } else {
            self.broken_links.keys().map(|s| s.as_str()).collect()
        };
        
        for source_id in sources {
            if let Some(links) = self.broken_links.get(source_id) {
                for link in links {
                    broken_links.push(BrokenLink {
                        source_note_id: source_id.to_string(),
                        source_note_path: self.id_to_path
                            .get(source_id)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        link: link.clone(),
                        suggestions: self.suggest_link_fixes(link),
                    });
                }
            }
        }
        
        broken_links
    }

    /// 建议断链修复方案
    fn suggest_link_fixes(&self, link: &WikiLink) -> Vec<String> {
        let mut suggestions = Vec::new();
        let target_lower = link.target.to_lowercase();
        
        // 查找相似的标题
        for title in self.title_to_id.keys() {
            let similarity = self.calculate_string_similarity(&target_lower, &title.to_lowercase());
            if similarity > 0.6 {
                suggestions.push(title.clone());
            }
        }
        
        // 查找相似的文件名
        for path in self.path_to_id.keys() {
            if let Some(filename) = path.file_stem() {
                let filename_str = filename.to_string_lossy();
                let similarity = self.calculate_string_similarity(&target_lower, &filename_str.to_lowercase());
                if similarity > 0.6 {
                    suggestions.push(filename_str.to_string());
                }
            }
        }
        
        // 限制建议数量
        suggestions.sort_by(|a, b| {
            let sim_a = self.calculate_string_similarity(&target_lower, &a.to_lowercase());
            let sim_b = self.calculate_string_similarity(&target_lower, &b.to_lowercase());
            sim_b.partial_cmp(&sim_a).unwrap()
        });
        
        suggestions.truncate(5);
        suggestions
    }

    /// 计算字符串相似度（简单的编辑距离）
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let max_len = std::cmp::max(len1, len2);
        let distance = self.levenshtein_distance(s1, s2);
        
        1.0 - (distance as f64 / max_len as f64)
    }

    /// 计算编辑距离
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }
        
        matrix[len1][len2]
    }

    /// 更新孤立笔记状态
    fn update_orphaned_status(&mut self) {
        self.orphaned_notes.clear();
        
        for note_id in self.id_to_path.keys() {
            let has_outgoing = self.outgoing_links
                .get(note_id)
                .map(|links| !links.is_empty())
                .unwrap_or(false);
            
            let has_incoming = self.incoming_links
                .get(note_id)
                .map(|links| !links.is_empty())
                .unwrap_or(false);
            
            if !has_outgoing && !has_incoming {
                self.orphaned_notes.insert(note_id.clone());
            }
        }
    }

    /// 获取孤立笔记列表
    pub fn get_orphaned_notes(&self) -> Vec<String> {
        self.orphaned_notes.iter().cloned().collect()
    }

    /// 获取链接统计信息
    pub fn get_statistics(&self) -> LinkIndexStats {
        let total_notes = self.id_to_path.len();
        let total_links: usize = self.outgoing_links.values().map(|links| links.len()).sum();
        let total_broken_links: usize = self.broken_links.values().map(|links| links.len()).sum();
        
        LinkIndexStats {
            total_notes,
            total_links,
            total_broken_links,
            orphaned_notes: self.orphaned_notes.len(),
        }
    }

    /// 根据笔记ID获取标题
    fn get_note_title(&self, note_id: &str) -> Option<String> {
        for (title, id) in &self.title_to_id {
            if id == note_id {
                return Some(title.clone());
            }
        }
        None
    }
}

/// 链接索引统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkIndexStats {
    pub total_notes: usize,
    pub total_links: usize,
    pub total_broken_links: usize,
    pub orphaned_notes: usize,
}

impl Default for LinkIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_link_index_basic_operations() {
        let mut index = LinkIndex::new();
        
        // 注册笔记
        index.register_note(
            "note1".to_string(),
            PathBuf::from("/path/note1.md"),
            "笔记1".to_string(),
        );
        index.register_note(
            "note2".to_string(),
            PathBuf::from("/path/note2.md"),
            "笔记2".to_string(),
        );
        
        // 创建链接
        let links = vec![WikiLink {
            raw: "[[笔记2]]".to_string(),
            target: "笔记2".to_string(),
            alias: None,
            anchor: None,
            is_embed: false,
            range: 0..8,
            line_number: 1,
        }];
        
        index.update_note_links("note1", links).unwrap();
        
        // 测试反向链接
        let backlinks = index.get_backlinks("note2");
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_note_id, "note1");
        
        // 测试正向链接
        let outgoing = index.get_outgoing_links("note1");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0], "note2");
    }
}