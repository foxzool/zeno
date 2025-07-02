use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 层次化标签结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HierarchicalTag {
    /// 标签名称（完整路径，如 "编程/Rust/异步"）
    pub name: String,
    /// 标签层级（从 0 开始）
    pub level: usize,
    /// 父标签（如果是根标签则为 None）
    pub parent: Option<String>,
    /// 子标签列表
    pub children: Vec<String>,
    /// 使用此标签的笔记数量
    pub note_count: usize,
    /// 标签颜色（可选）
    pub color: Option<String>,
}

/// 标签层次结构管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagHierarchy {
    /// 所有标签的映射表
    tags: HashMap<String, HierarchicalTag>,
    /// 根标签列表
    root_tags: HashSet<String>,
}

impl TagHierarchy {
    /// 创建新的标签层次结构
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            root_tags: HashSet::new(),
        }
    }

    /// 解析标签字符串，支持层次化格式
    /// 支持格式：
    /// - "编程" -> 根标签
    /// - "编程/Rust" -> 两级标签
    /// - "编程/Rust/异步" -> 三级标签
    pub fn parse_tag(&mut self, tag_str: &str) -> Vec<String> {
        let parts: Vec<&str> = tag_str.split('/').collect();
        let mut created_tags = Vec::new();
        let mut current_path = String::new();
        let mut parent: Option<String> = None;

        for (level, part) in parts.iter().enumerate() {
            let trimmed_part = part.trim();
            if trimmed_part.is_empty() {
                continue;
            }

            if level == 0 {
                current_path = trimmed_part.to_string();
            } else {
                current_path = format!("{}/{}", current_path, trimmed_part);
            }

            // 如果标签不存在，创建它
            if !self.tags.contains_key(&current_path) {
                let tag = HierarchicalTag {
                    name: current_path.clone(),
                    level,
                    parent: parent.clone(),
                    children: Vec::new(),
                    note_count: 0,
                    color: None,
                };

                self.tags.insert(current_path.clone(), tag);

                // 如果是根标签，添加到根标签集合
                if level == 0 {
                    self.root_tags.insert(current_path.clone());
                }

                // 更新父标签的子标签列表
                if let Some(parent_name) = &parent {
                    if let Some(parent_tag) = self.tags.get_mut(parent_name) {
                        if !parent_tag.children.contains(&current_path) {
                            parent_tag.children.push(current_path.clone());
                        }
                    }
                }
            }

            created_tags.push(current_path.clone());
            parent = Some(current_path.clone());
        }

        created_tags
    }

    /// 更新标签的笔记计数
    pub fn update_note_count(&mut self, tag_name: &str, delta: i32) {
        if let Some(tag) = self.tags.get_mut(tag_name) {
            if delta < 0 && tag.note_count >= (-delta) as usize {
                tag.note_count -= (-delta) as usize;
            } else if delta > 0 {
                tag.note_count += delta as usize;
            }
        }
    }

    /// 递增标签使用计数
    pub fn increment_tag_usage(&mut self, tag_name: &str) {
        self.update_note_count(tag_name, 1);
    }

    /// 递减标签使用计数
    pub fn decrement_tag_usage(&mut self, tag_name: &str) {
        self.update_note_count(tag_name, -1);
    }

    /// 获取标签
    pub fn get_tag(&self, tag_name: &str) -> Option<&HierarchicalTag> {
        self.tags.get(tag_name)
    }

    /// 获取所有根标签
    pub fn get_root_tags(&self) -> Vec<&HierarchicalTag> {
        self.root_tags
            .iter()
            .filter_map(|name| self.tags.get(name))
            .collect()
    }

    /// 获取标签的子标签
    pub fn get_children(&self, tag_name: &str) -> Vec<&HierarchicalTag> {
        if let Some(tag) = self.tags.get(tag_name) {
            tag.children
                .iter()
                .filter_map(|child_name| self.tags.get(child_name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取标签的所有祖先标签
    pub fn get_ancestors(&self, tag_name: &str) -> Vec<&HierarchicalTag> {
        let mut ancestors = Vec::new();
        let mut current = tag_name;

        while let Some(tag) = self.tags.get(current) {
            if let Some(parent_name) = &tag.parent {
                if let Some(parent_tag) = self.tags.get(parent_name) {
                    ancestors.push(parent_tag);
                    current = parent_name;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        ancestors.reverse(); // 从根标签开始
        ancestors
    }

    /// 获取标签的所有后代标签
    pub fn get_descendants(&self, tag_name: &str) -> Vec<&HierarchicalTag> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![tag_name];

        while let Some(current) = to_visit.pop() {
            if let Some(tag) = self.tags.get(current) {
                for child_name in &tag.children {
                    if let Some(child_tag) = self.tags.get(child_name) {
                        descendants.push(child_tag);
                        to_visit.push(&child_tag.name);
                    }
                }
            }
        }

        descendants
    }

    /// 获取所有标签
    pub fn get_all_tags(&self) -> Vec<&HierarchicalTag> {
        self.tags.values().collect()
    }

    /// 搜索标签
    pub fn search_tags(&self, query: &str) -> Vec<&HierarchicalTag> {
        let query_lower = query.to_lowercase();
        self.tags
            .values()
            .filter(|tag| {
                tag.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// 获取热门标签（按使用次数排序）
    pub fn get_popular_tags(&self, limit: usize) -> Vec<&HierarchicalTag> {
        let mut tags: Vec<&HierarchicalTag> = self.tags.values().collect();
        tags.sort_by(|a, b| b.note_count.cmp(&a.note_count));
        tags.truncate(limit);
        tags
    }

    /// 建议相关标签
    pub fn suggest_related_tags(&self, tag_name: &str) -> Vec<&HierarchicalTag> {
        let mut suggestions = Vec::new();

        if let Some(tag) = self.tags.get(tag_name) {
            // 添加同级标签（兄弟标签）
            if let Some(parent_name) = &tag.parent {
                let siblings = self.get_children(parent_name);
                for sibling in siblings {
                    if sibling.name != tag_name {
                        suggestions.push(sibling);
                    }
                }
            }

            // 添加子标签
            suggestions.extend(self.get_children(tag_name));

            // 添加父标签
            if let Some(parent_name) = &tag.parent {
                if let Some(parent_tag) = self.tags.get(parent_name) {
                    suggestions.push(parent_tag);
                }
            }
        }

        suggestions
    }

    /// 清理未使用的标签
    pub fn cleanup_unused_tags(&mut self) {
        let unused_tags: Vec<String> = self
            .tags
            .iter()
            .filter(|(_, tag)| tag.note_count == 0)
            .map(|(name, _)| name.clone())
            .collect();

        for tag_name in unused_tags {
            self.remove_tag(&tag_name);
        }
    }

    /// 移除标签
    fn remove_tag(&mut self, tag_name: &str) {
        if let Some(tag) = self.tags.remove(tag_name) {
            // 从根标签集合中移除
            self.root_tags.remove(tag_name);

            // 从父标签的子标签列表中移除
            if let Some(parent_name) = &tag.parent {
                if let Some(parent_tag) = self.tags.get_mut(parent_name) {
                    parent_tag.children.retain(|child| child != tag_name);
                }
            }

            // 移除所有子标签
            let children = tag.children.clone();
            for child_name in children {
                self.remove_tag(&child_name);
            }
        }
    }

    /// 获取标签统计信息
    pub fn get_statistics(&self) -> TagStatistics {
        TagStatistics {
            total_tags: self.tags.len(),
            root_tags: self.root_tags.len(),
            max_depth: self.calculate_max_depth(),
            avg_children: self.calculate_avg_children(),
            most_used_tag: self.get_most_used_tag(),
        }
    }

    fn calculate_max_depth(&self) -> usize {
        self.tags.values().map(|tag| tag.level).max().unwrap_or(0) + 1
    }

    fn calculate_avg_children(&self) -> f64 {
        if self.tags.is_empty() {
            return 0.0;
        }

        let total_children: usize = self.tags.values().map(|tag| tag.children.len()).sum();
        total_children as f64 / self.tags.len() as f64
    }

    fn get_most_used_tag(&self) -> Option<String> {
        self.tags
            .values()
            .max_by_key(|tag| tag.note_count)
            .map(|tag| tag.name.clone())
    }
}

/// 标签统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TagStatistics {
    pub total_tags: usize,
    pub root_tags: usize,
    pub max_depth: usize,
    pub avg_children: f64,
    pub most_used_tag: Option<String>,
}

impl Default for TagHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tag() {
        let mut hierarchy = TagHierarchy::new();
        let tags = hierarchy.parse_tag("编程");
        
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "编程");
        
        let tag = hierarchy.get_tag("编程").unwrap();
        assert_eq!(tag.level, 0);
        assert_eq!(tag.parent, None);
    }

    #[test]
    fn test_parse_hierarchical_tag() {
        let mut hierarchy = TagHierarchy::new();
        let tags = hierarchy.parse_tag("编程/Rust/异步");
        
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], "编程");
        assert_eq!(tags[1], "编程/Rust");
        assert_eq!(tags[2], "编程/Rust/异步");
        
        // 检查层级关系
        let rust_tag = hierarchy.get_tag("编程/Rust").unwrap();
        assert_eq!(rust_tag.level, 1);
        assert_eq!(rust_tag.parent, Some("编程".to_string()));
        
        let async_tag = hierarchy.get_tag("编程/Rust/异步").unwrap();
        assert_eq!(async_tag.level, 2);
        assert_eq!(async_tag.parent, Some("编程/Rust".to_string()));
    }

    #[test]
    fn test_tag_relationships() {
        let mut hierarchy = TagHierarchy::new();
        hierarchy.parse_tag("编程/Rust/异步");
        hierarchy.parse_tag("编程/Rust/所有权");
        hierarchy.parse_tag("编程/Python");
        
        // 检查子标签
        let rust_children = hierarchy.get_children("编程/Rust");
        assert_eq!(rust_children.len(), 2);
        
        // 检查祖先标签
        let ancestors = hierarchy.get_ancestors("编程/Rust/异步");
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].name, "编程");
        assert_eq!(ancestors[1].name, "编程/Rust");
    }
}