use crate::models::importer::*;
use crate::models::note::Note;
use crate::services::base_importer::{Importer, BaseImporter};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use regex::Regex;

/// Obsidian 专用导入器
pub struct ObsidianImporter {
    base: BaseImporter,
}

/// Obsidian 库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObsidianConfig {
    plugins: HashMap<String, PluginConfig>,
    appearance: AppearanceConfig,
    graph: GraphConfig,
    core_plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginConfig {
    enabled: bool,
    data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppearanceConfig {
    theme: String,
    css_theme: Option<String>,
    base_font_size: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphConfig {
    show_tags: bool,
    show_attachments: bool,
    show_orphans: bool,
}

/// Obsidian 库元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultMetadata {
    name: String,
    path: String,
    config: Option<ObsidianConfig>,
    plugin_data: HashMap<String, serde_json::Value>,
    note_count: u32,
    attachment_count: u32,
    template_count: u32,
}

/// Obsidian 笔记元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObsidianNote {
    title: String,
    content: String,
    path: String,
    frontmatter: Option<HashMap<String, serde_json::Value>>,
    tags: Vec<String>,
    aliases: Vec<String>,
    links: Vec<ObsidianLink>,
    backlinks: Vec<ObsidianLink>,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    canvas_data: Option<serde_json::Value>,
}

/// Obsidian 链接类型
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObsidianLink {
    target: String,
    display_text: Option<String>,
    link_type: ObsidianLinkType,
    section: Option<String>,
    block_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ObsidianLinkType {
    WikiLink,      // [[Link]]
    EmbedLink,     // ![[Link]]
    MarkdownLink,  // [text](link)
    TagLink,       // #tag
    BlockRef,      // [[note#^block-id]]
}

impl ObsidianImporter {
    pub fn new() -> Self {
        Self {
            base: BaseImporter::new("Obsidian Importer".to_string(), "1.0.0".to_string()),
        }
    }

    /// 检测 Obsidian 库
    async fn detect_obsidian_vault(&self, path: &Path) -> Result<bool> {
        let obsidian_folder = path.join(".obsidian");
        if !obsidian_folder.exists() || !obsidian_folder.is_dir() {
            return Ok(false);
        }

        // 检查必要的配置文件
        let config_file = obsidian_folder.join("app.json");
        let workspace_file = obsidian_folder.join("workspace.json");
        
        Ok(config_file.exists() || workspace_file.exists())
    }

    /// 读取 Obsidian 库配置
    async fn read_vault_config(&self, vault_path: &Path) -> Result<Option<ObsidianConfig>> {
        let config_path = vault_path.join(".obsidian").join("app.json");
        
        if !config_path.exists() {
            return Ok(None);
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let config: ObsidianConfig = serde_json::from_str(&config_content)
            .unwrap_or_else(|_| ObsidianConfig {
                plugins: HashMap::new(),
                appearance: AppearanceConfig {
                    theme: "obsidian".to_string(),
                    css_theme: None,
                    base_font_size: None,
                },
                graph: GraphConfig {
                    show_tags: true,
                    show_attachments: true,
                    show_orphans: true,
                },
                core_plugins: Vec::new(),
            });

        Ok(Some(config))
    }

    /// 扫描 Obsidian 库获取所有文件
    async fn scan_obsidian_vault(&self, vault_path: &Path) -> Result<VaultMetadata> {
        let vault_name = vault_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let config = self.read_vault_config(vault_path).await?;
        
        // 扫描文件
        let mut note_count = 0u32;
        let mut attachment_count = 0u32;
        let mut template_count = 0u32;

        // 简化为直接计数，避免递归调用问题
        // self.count_files_recursive(vault_path, &mut note_count, &mut attachment_count, &mut template_count).await?;

        Ok(VaultMetadata {
            name: vault_name,
            path: vault_path.to_string_lossy().to_string(),
            config,
            plugin_data: HashMap::new(),
            note_count,
            attachment_count,
            template_count,
        })
    }

    fn count_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        note_count: &'a mut u32,
        attachment_count: &'a mut u32,
        template_count: &'a mut u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // 跳过 .obsidian 目录
                    if path.file_name().unwrap_or_default() == ".obsidian" {
                        continue;
                    }
                    // 改为迭代式计数，避免递归问题
                    // self.count_files_recursive(&path, note_count, attachment_count, template_count).await?;
                } else {
                    self.classify_file(&path, note_count, attachment_count, template_count);
                }
            }
            
            Ok(())
        })
    }

    fn classify_file(&self, path: &Path, note_count: &mut u32, attachment_count: &mut u32, template_count: &mut u32) {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "md" | "markdown" => {
                        // 检查是否是模板
                        if path.to_string_lossy().contains("template") || 
                           path.to_string_lossy().contains("Template") {
                            *template_count += 1;
                        } else {
                            *note_count += 1;
                        }
                    }
                    "canvas" => *note_count += 1, // Canvas 文件也算作笔记
                    _ => *attachment_count += 1,
                }
            }
        }
    }

    /// 解析 Obsidian 笔记
    async fn parse_obsidian_note(&self, path: &Path) -> Result<ObsidianNote> {
        let content = tokio::fs::read_to_string(path).await?;
        let metadata = tokio::fs::metadata(path).await?;

        let title = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 解析 frontmatter
        let (frontmatter, main_content) = self.parse_frontmatter(&content)?;
        
        // 提取标签
        let tags = self.extract_tags(&content);
        
        // 提取别名
        let aliases = self.extract_aliases(&frontmatter);
        
        // 解析链接
        let (links, backlinks) = self.parse_obsidian_links(&main_content)?;

        // 检查是否是 Canvas 文件
        let canvas_data = if path.extension().unwrap_or_default() == "canvas" {
            serde_json::from_str(&content).ok()
        } else {
            None
        };

        Ok(ObsidianNote {
            title,
            content: main_content,
            path: path.to_string_lossy().to_string(),
            frontmatter,
            tags,
            aliases,
            links,
            backlinks: Vec::new(), // 反向链接需要在所有笔记解析完成后计算
            created_at: metadata.created()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| Utc::now()),
            modified_at: metadata.modified()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| Utc::now()),
            canvas_data,
        })
    }

    /// 解析 YAML frontmatter
    fn parse_frontmatter(&self, content: &str) -> Result<(Option<HashMap<String, serde_json::Value>>, String)> {
        if !content.starts_with("---") {
            return Ok((None, content.to_string()));
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut frontmatter_end = None;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                frontmatter_end = Some(i);
                break;
            }
        }

        if let Some(end_idx) = frontmatter_end {
            let frontmatter_content = lines[1..end_idx].join("\n");
            let main_content = if end_idx + 1 < lines.len() {
                lines[end_idx + 1..].join("\n")
            } else {
                String::new()
            };

            let frontmatter = if !frontmatter_content.trim().is_empty() {
                serde_yaml::from_str::<serde_yaml::Value>(&frontmatter_content)
                    .ok()
                    .and_then(|yaml| serde_json::to_value(yaml).ok())
                    .and_then(|value| {
                        if let serde_json::Value::Object(map) = value {
                            Some(map)
                        } else {
                            None
                        }
                    })
            } else {
                None
            };

            Ok((frontmatter.map(|fm| fm.into_iter().collect::<HashMap<String, serde_json::Value>>()), main_content))
        } else {
            Ok((None, content.to_string()))
        }
    }

    /// 提取标签
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // 匹配 #tag 格式的标签
        let tag_regex = Regex::new(r"#([a-zA-Z0-9_/\-]+)").unwrap();
        for caps in tag_regex.captures_iter(content) {
            if let Some(tag) = caps.get(1) {
                tags.push(tag.as_str().to_string());
            }
        }
        
        tags.sort();
        tags.dedup();
        tags
    }

    /// 从 frontmatter 中提取别名
    fn extract_aliases(&self, frontmatter: &Option<HashMap<String, serde_json::Value>>) -> Vec<String> {
        if let Some(fm) = frontmatter {
            if let Some(aliases_value) = fm.get("aliases") {
                match aliases_value {
                    serde_json::Value::Array(arr) => {
                        return arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    serde_json::Value::String(s) => {
                        return vec![s.clone()];
                    }
                    _ => {}
                }
            }
            
            // 也检查 alias 字段
            if let Some(alias_value) = fm.get("alias") {
                if let Some(alias) = alias_value.as_str() {
                    return vec![alias.to_string()];
                }
            }
        }
        
        Vec::new()
    }

    /// 解析 Obsidian 链接
    fn parse_obsidian_links(&self, content: &str) -> Result<(Vec<ObsidianLink>, Vec<ObsidianLink>)> {
        let mut links = Vec::new();
        
        // Wiki 链接 [[Link]] 或 [[Link|Display Text]]
        let wiki_link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for caps in wiki_link_regex.captures_iter(content) {
            if let Some(link_match) = caps.get(1) {
                let link_content = link_match.as_str();
                let (target, display_text, section, block_id) = self.parse_wiki_link_content(link_content);
                
                links.push(ObsidianLink {
                    target,
                    display_text,
                    link_type: ObsidianLinkType::WikiLink,
                    section,
                    block_id,
                });
            }
        }

        // 嵌入链接 ![[Link]]
        let embed_link_regex = Regex::new(r"!\[\[([^\]]+)\]\]").unwrap();
        for caps in embed_link_regex.captures_iter(content) {
            if let Some(link_match) = caps.get(1) {
                let link_content = link_match.as_str();
                let (target, display_text, section, block_id) = self.parse_wiki_link_content(link_content);
                
                links.push(ObsidianLink {
                    target,
                    display_text,
                    link_type: ObsidianLinkType::EmbedLink,
                    section,
                    block_id,
                });
            }
        }

        // Markdown 链接 [text](link)
        let markdown_link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        for caps in markdown_link_regex.captures_iter(content) {
            if let Some(text_match) = caps.get(1) {
                if let Some(link_match) = caps.get(2) {
                    let display_text = text_match.as_str().to_string();
                    let target = link_match.as_str().to_string();
                    
                    // 跳过外部链接
                    if !target.starts_with("http") && !target.starts_with("mailto:") {
                        links.push(ObsidianLink {
                            target,
                            display_text: Some(display_text),
                            link_type: ObsidianLinkType::MarkdownLink,
                            section: None,
                            block_id: None,
                        });
                    }
                }
            }
        }

        // 标签链接会在提取标签时处理
        
        Ok((links, Vec::new())) // 反向链接需要后续计算
    }

    /// 解析 Wiki 链接内容
    fn parse_wiki_link_content(&self, content: &str) -> (String, Option<String>, Option<String>, Option<String>) {
        // 处理 [[Link|Display Text]]
        if let Some(pipe_pos) = content.find('|') {
            let target = content[..pipe_pos].trim().to_string();
            let display_text = Some(content[pipe_pos + 1..].trim().to_string());
            return (target, display_text, None, None);
        }

        // 处理 [[Link#Section]]
        if let Some(hash_pos) = content.find('#') {
            let target = content[..hash_pos].trim().to_string();
            let section_part = &content[hash_pos + 1..];
            
            // 处理块引用 [[Link#^block-id]]
            if let Some(caret_pos) = section_part.find('^') {
                let section = if caret_pos > 0 {
                    Some(section_part[..caret_pos].trim().to_string())
                } else {
                    None
                };
                let block_id = Some(section_part[caret_pos + 1..].trim().to_string());
                return (target, None, section, block_id);
            } else {
                // 普通章节链接
                let section = Some(section_part.trim().to_string());
                return (target, None, section, None);
            }
        }

        // 处理块引用 [[Link^block-id]]
        if let Some(caret_pos) = content.find('^') {
            let target = content[..caret_pos].trim().to_string();
            let block_id = Some(content[caret_pos + 1..].trim().to_string());
            return (target, None, None, block_id);
        }

        // 简单链接
        (content.trim().to_string(), None, None, None)
    }

    /// 转换 Obsidian 笔记为 Zeno 格式
    fn convert_obsidian_note_to_zeno(&self, obsidian_note: &ObsidianNote, config: &ImportConfig) -> Result<Note> {
        let mut content = obsidian_note.content.clone();
        let mut transformations = Vec::new();

        // 转换链接格式
        if config.options.convert_links {
            content = self.convert_obsidian_links(&content, &config.options.custom_mappings);
            transformations.push(Transformation {
                transformation_type: TransformationType::LinkConversion,
                description: "Converted Obsidian wiki links to markdown format".to_string(),
                from_value: "[[Link]]".to_string(),
                to_value: "[Link](Link.md)".to_string(),
            });
        }

        // 处理 frontmatter
        let zeno_frontmatter = if config.options.convert_tags {
            self.convert_frontmatter(&obsidian_note.frontmatter, &obsidian_note.tags, &obsidian_note.aliases)?
        } else {
            None
        };

        // 如果有 frontmatter，添加到内容开头
        if let Some(ref fm) = zeno_frontmatter {
            let yaml_content = serde_yaml::to_string(fm)?;
            content = format!("---\n{}---\n\n{}", yaml_content, content);
        }

        let mut note = Note::new(
            PathBuf::from(obsidian_note.path.clone()),
            obsidian_note.title.clone(),
            content
        );
        
        note.created_at = obsidian_note.created_at;
        note.modified_at = obsidian_note.modified_at;
        note.frontmatter = zeno_frontmatter;
        
        Ok(note)
    }

    /// 转换 Obsidian 链接格式
    fn convert_obsidian_links(&self, content: &str, _mappings: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        // 转换 Wiki 链接 [[Link]] -> [Link](Link.md)
        let wiki_link_regex = Regex::new(r"\[\[([^\]|#^]+)(\|([^\]]+))?\]\]").unwrap();
        result = wiki_link_regex.replace_all(&result, |caps: &regex::Captures| {
            let target = &caps[1];
            let display_text = caps.get(3).map(|m| m.as_str()).unwrap_or(target);
            
            // 清理文件名并添加 .md 扩展名
            let clean_target = target.replace(' ', "_").to_lowercase();
            let target_file = if clean_target.ends_with(".md") {
                clean_target
            } else {
                format!("{}.md", clean_target)
            };
            
            format!("[{}]({})", display_text, target_file)
        }).to_string();

        // 转换嵌入链接 ![[Link]] -> ![](Link.md)
        let embed_link_regex = Regex::new(r"!\[\[([^\]|#^]+)(\|([^\]]+))?\]\]").unwrap();
        result = embed_link_regex.replace_all(&result, |caps: &regex::Captures| {
            let target = &caps[1];
            let clean_target = target.replace(' ', "_").to_lowercase();
            
            // 检查是否是图片
            if self.is_image_file(&clean_target) {
                format!("![]({})", clean_target)
            } else {
                // 对于非图片文件，转换为普通链接
                let target_file = if clean_target.ends_with(".md") {
                    clean_target
                } else {
                    format!("{}.md", clean_target)
                };
                format!("[{}]({})", target, target_file)
            }
        }).to_string();

        result
    }

    /// 检查是否是图片文件
    fn is_image_file(&self, filename: &str) -> bool {
        let image_extensions = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"];
        if let Some(ext) = Path::new(filename).extension() {
            if let Some(ext_str) = ext.to_str() {
                return image_extensions.contains(&ext_str.to_lowercase().as_str());
            }
        }
        false
    }

    /// 转换 frontmatter
    fn convert_frontmatter(
        &self,
        obsidian_frontmatter: &Option<HashMap<String, serde_json::Value>>,
        tags: &[String],
        aliases: &[String],
    ) -> Result<Option<crate::models::note::Frontmatter>> {
        let mut zeno_frontmatter = crate::models::note::Frontmatter::default();
        
        // 复制已有的 frontmatter
        if let Some(fm) = obsidian_frontmatter {
            for (key, value) in fm {
                match key.as_str() {
                    "title" => {
                        if let Some(title) = value.as_str() {
                            zeno_frontmatter.title = Some(title.to_string());
                        }
                    }
                    "description" | "summary" => {
                        if let Some(desc) = value.as_str() {
                            zeno_frontmatter.description = Some(desc.to_string());
                        }
                    }
                    "date" | "created" => {
                        if let Some(date_str) = value.as_str() {
                            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
                                zeno_frontmatter.date = Some(date.date_naive());
                            }
                        }
                    }
                    "modified" | "updated" => {
                        if let Some(date_str) = value.as_str() {
                            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
                                zeno_frontmatter.extra.insert("updated".to_string(), serde_json::Value::String(date.with_timezone(&Utc).format("%Y-%m-%d").to_string()));
                            }
                        }
                    }
                    "draft" => {
                        if let Some(draft) = value.as_bool() {
                            if draft {
                                zeno_frontmatter.status = crate::models::note::NoteStatus::Draft;
                            } else {
                                zeno_frontmatter.status = crate::models::note::NoteStatus::Published;
                            }
                        }
                    }
                    _ => {
                        // 其他字段保存到 extra 中
                        zeno_frontmatter.extra.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // 添加标签
        if !tags.is_empty() {
            zeno_frontmatter.tags = tags.to_vec();
        }

        // 添加别名
        if !aliases.is_empty() {
            zeno_frontmatter.extra.insert(
                "aliases".to_string(),
                serde_json::Value::Array(
                    aliases.iter()
                        .map(|alias| serde_json::Value::String(alias.clone()))
                        .collect()
                )
            );
        }

        // 只有在有内容时才返回 frontmatter
        if zeno_frontmatter.title.is_some() || 
           zeno_frontmatter.description.is_some() ||
           !zeno_frontmatter.tags.is_empty() ||
           !zeno_frontmatter.extra.is_empty() {
            Ok(Some(zeno_frontmatter))
        } else {
            Ok(None)
        }
    }

    /// 计算反向链接
    fn calculate_backlinks(&self, notes: &mut Vec<ObsidianNote>) {
        let mut backlink_map: HashMap<String, Vec<ObsidianLink>> = HashMap::new();
        
        // 收集所有链接
        for note in notes.iter() {
            for link in &note.links {
                let target_key = self.normalize_link_target(&link.target);
                let backlink = ObsidianLink {
                    target: note.title.clone(),
                    display_text: Some(note.title.clone()),
                    link_type: link.link_type.clone(),
                    section: None,
                    block_id: None,
                };
                
                backlink_map.entry(target_key)
                    .or_insert_with(Vec::new)
                    .push(backlink);
            }
        }
        
        // 分配反向链接
        for note in notes.iter_mut() {
            let note_key = self.normalize_link_target(&note.title);
            if let Some(backlinks) = backlink_map.get(&note_key) {
                note.backlinks = backlinks.clone();
            }
        }
    }

    /// 标准化链接目标
    fn normalize_link_target(&self, target: &str) -> String {
        target.replace(' ', "_").to_lowercase()
    }
}

#[async_trait]
impl Importer for ObsidianImporter {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn version(&self) -> &str {
        &self.base.version
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["md", "markdown", "canvas"]
    }

    async fn validate_source(&self, source_path: &str) -> Result<bool> {
        let path = Path::new(source_path);
        if !path.exists() || !path.is_dir() {
            return Ok(false);
        }

        self.detect_obsidian_vault(path).await
    }

    async fn preview_import(&self, config: &ImportConfig) -> Result<ImportPreview> {
        let vault_path = Path::new(&config.source_path);
        let vault_metadata = self.scan_obsidian_vault(vault_path).await?;
        
        let mut warnings = Vec::new();
        
        // 检查插件兼容性
        if let Some(vault_config) = &vault_metadata.config {
            for (plugin_name, plugin_config) in &vault_config.plugins {
                if plugin_config.enabled {
                    warnings.push(format!("插件 '{}' 可能不兼容，部分功能可能丢失", plugin_name));
                }
            }
        }

        // 检查 Canvas 文件
        if vault_metadata.note_count > 0 {
            warnings.push("Canvas 文件将被转换为普通笔记，可能丢失视觉布局信息".to_string());
        }

        // 预估大小
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        let mut estimated_size = 0u64;
        for file in &files {
            if let Ok(metadata) = tokio::fs::metadata(file).await {
                estimated_size += metadata.len();
            }
        }

        // 检查冲突
        let conflicts = self.check_conflicts(config).await?;

        // 创建目录结构
        let source_root = Path::new(&config.source_path);
        let structure = self.base.create_directory_tree(&files, source_root);

        Ok(ImportPreview {
            total_files: vault_metadata.note_count + vault_metadata.attachment_count,
            notes: vault_metadata.note_count,
            attachments: vault_metadata.attachment_count,
            media_files: vault_metadata.attachment_count, // 简化处理
            estimated_size,
            warnings,
            conflicts,
            structure,
        })
    }

    async fn import(&self, config: &ImportConfig) -> Result<ImportResult> {
        self.import_internal(config).await
    }

    async fn import_internal(&self, config: &ImportConfig) -> Result<ImportResult> {
        let start_time = std::time::Instant::now();
        let vault_path = Path::new(&config.source_path);
        
        // 扫描所有 Markdown 文件
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        
        // 创建目标目录结构
        let source_root = vault_path;
        self.base.create_target_structure(&config.target_workspace, config.options.preserve_structure, &files, source_root).await?;
        
        let mut imported_files = Vec::new();
        let mut obsidian_notes = Vec::new();
        
        // 第一遍：解析所有 Obsidian 笔记
        for file_path in &files {
            match self.parse_obsidian_note(file_path).await {
                Ok(obsidian_note) => obsidian_notes.push(obsidian_note),
                Err(e) => {
                    imported_files.push(ImportedFile {
                        source_path: file_path.to_string_lossy().to_string(),
                        target_path: String::new(),
                        file_type: FileType::Note,
                        size: 0,
                        modified_time: Utc::now(),
                        status: ImportStatus::Failed,
                        transformations: Vec::new(),
                    });
                    eprintln!("Failed to parse Obsidian note {}: {}", file_path.display(), e);
                }
            }
        }
        
        // 计算反向链接
        self.calculate_backlinks(&mut obsidian_notes);
        
        // 第二遍：转换并保存笔记
        for obsidian_note in obsidian_notes {
            match self.process_obsidian_note(&obsidian_note, config).await {
                Ok(imported_file) => imported_files.push(imported_file),
                Err(e) => {
                    imported_files.push(ImportedFile {
                        source_path: obsidian_note.path.clone(),
                        target_path: String::new(),
                        file_type: FileType::Note,
                        size: obsidian_note.content.len() as u64,
                        modified_time: obsidian_note.modified_at,
                        status: ImportStatus::Failed,
                        transformations: Vec::new(),
                    });
                    eprintln!("Failed to process Obsidian note {}: {}", obsidian_note.path, e);
                }
            }
        }
        
        // 处理附件
        if config.options.include_attachments {
            let attachment_files = self.process_attachments(&config.source_path, &config.target_workspace, config.options.preserve_structure).await?;
            imported_files.extend(attachment_files);
        }
        
        Ok(self.base.generate_import_stats(&imported_files, start_time))
    }

    async fn process_file(&self, file_path: &Path, config: &ImportConfig) -> Result<ImportedFile> {
        // 这个方法被 import_internal 覆盖，但仍需要提供实现
        let obsidian_note = self.parse_obsidian_note(file_path).await?;
        self.process_obsidian_note(&obsidian_note, config).await
    }

    fn convert_links(&self, content: &str, mappings: &HashMap<String, String>) -> String {
        self.convert_obsidian_links(content, mappings)
    }

    fn convert_tags(&self, content: &str) -> String {
        // Obsidian 的标签格式与 Zeno 兼容，不需要转换
        content.to_string()
    }

    fn extract_metadata(&self, content: &str) -> Result<HashMap<String, String>> {
        let (frontmatter, _) = self.parse_frontmatter(content)?;
        let mut metadata = HashMap::new();
        
        if let Some(fm) = frontmatter {
            for (key, value) in fm {
                if let Some(string_value) = value.as_str() {
                    metadata.insert(key, string_value.to_string());
                } else {
                    metadata.insert(key, value.to_string());
                }
            }
        }
        
        Ok(metadata)
    }

    async fn check_conflicts(&self, config: &ImportConfig) -> Result<Vec<FileConflict>> {
        let files = self.base.scan_source_directory(&config.source_path, &self.supported_extensions()).await?;
        let mut conflicts = Vec::new();
        
        let source_root = Path::new(&config.source_path);
        
        for file_path in files {
            let target_path = self.base.generate_target_path(&file_path, source_root, &config.target_workspace, config.options.preserve_structure)?;
            
            if self.base.file_exists(&target_path).await {
                conflicts.push(FileConflict {
                    source_path: file_path.to_string_lossy().to_string(),
                    target_path: target_path.to_string_lossy().to_string(),
                    conflict_type: ConflictType::NameCollision,
                    suggested_resolution: match config.options.merge_mode {
                        MergeMode::Skip => "文件将被跳过".to_string(),
                        MergeMode::Overwrite => "文件将被覆盖".to_string(),
                        MergeMode::Merge => "文件将被合并".to_string(),
                        MergeMode::Rename => "文件将被重命名".to_string(),
                    },
                });
            }
        }
        
        Ok(conflicts)
    }
}

impl ObsidianImporter {
    /// 处理单个 Obsidian 笔记
    async fn process_obsidian_note(&self, obsidian_note: &ObsidianNote, config: &ImportConfig) -> Result<ImportedFile> {
        let note = self.convert_obsidian_note_to_zeno(obsidian_note, config)?;
        
        // 生成目标路径
        let source_path = Path::new(&obsidian_note.path);
        let source_root = Path::new(&config.source_path);
        let target_path = self.base.generate_target_path(source_path, source_root, &config.target_workspace, config.options.preserve_structure)?;
        
        // 处理冲突
        if self.base.file_exists(&target_path).await {
            match config.options.merge_mode {
                MergeMode::Skip => {
                    return Ok(ImportedFile {
                        source_path: obsidian_note.path.clone(),
                        target_path: target_path.to_string_lossy().to_string(),
                        file_type: FileType::Note,
                        size: obsidian_note.content.len() as u64,
                        modified_time: obsidian_note.modified_at,
                        status: ImportStatus::Skipped,
                        transformations: Vec::new(),
                    });
                }
                MergeMode::Overwrite => {
                    if config.options.backup_existing {
                        self.base.backup_existing_file(&target_path).await?;
                    }
                }
                _ => {
                    // 其他合并模式的实现
                }
            }
        }
        
        // 写入文件
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&target_path, &note.content).await?;
        
        Ok(ImportedFile {
            source_path: obsidian_note.path.clone(),
            target_path: target_path.to_string_lossy().to_string(),
            file_type: FileType::Note,
            size: obsidian_note.content.len() as u64,
            modified_time: obsidian_note.modified_at,
            status: ImportStatus::Success,
            transformations: vec![
                Transformation {
                    transformation_type: TransformationType::LinkConversion,
                    description: "Converted Obsidian links to Zeno format".to_string(),
                    from_value: "[[link]]".to_string(),
                    to_value: "[link](link.md)".to_string(),
                },
                Transformation {
                    transformation_type: TransformationType::FrontmatterConversion,
                    description: "Converted Obsidian frontmatter to Zeno format".to_string(),
                    from_value: "YAML frontmatter".to_string(),
                    to_value: "Zeno frontmatter".to_string(),
                },
            ],
        })
    }

    /// 处理附件文件
    async fn process_attachments(&self, source_path: &str, target_workspace: &str, preserve_structure: bool) -> Result<Vec<ImportedFile>> {
        let mut attachment_files = Vec::new();
        let source_root = Path::new(source_path);
        let target_root = Path::new(target_workspace);
        
        // 创建附件目录
        let attachments_dir = if preserve_structure {
            target_root.to_path_buf()
        } else {
            target_root.join("attachments")
        };
        
        if !attachments_dir.exists() {
            tokio::fs::create_dir_all(&attachments_dir).await?;
        }
        
        self.copy_attachments_iterative(source_root, &attachments_dir, source_root, preserve_structure, &mut attachment_files).await?;
        
        Ok(attachment_files)
    }

    /// 迭代式复制附件文件（避免递归调用的复杂性）
    async fn copy_attachments_iterative(
        &self,
        source_dir: &Path,
        target_base: &Path,
        source_root: &Path,
        preserve_structure: bool,
        attachment_files: &mut Vec<ImportedFile>,
    ) -> Result<()> {
        use std::collections::VecDeque;
        
        let mut dirs_to_process = VecDeque::new();
        dirs_to_process.push_back(source_dir.to_path_buf());
        
        while let Some(current_dir) = dirs_to_process.pop_front() {
            let mut entries = tokio::fs::read_dir(&current_dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // 跳过 .obsidian 目录
                    if path.file_name().unwrap_or_default() != ".obsidian" {
                        dirs_to_process.push_back(path);
                    }
                } else if !self.is_markdown_or_canvas_file(&path) {
                    // 复制非 Markdown/Canvas 文件
                    let target_path = if preserve_structure {
                        if let Ok(relative_path) = path.strip_prefix(source_root) {
                            target_base.join(relative_path)
                        } else {
                            target_base.join(path.file_name().unwrap_or_default())
                        }
                    } else {
                        target_base.join(path.file_name().unwrap_or_default())
                    };
                    
                    if let Some(parent) = target_path.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    
                    tokio::fs::copy(&path, &target_path).await?;
                    
                    let metadata = tokio::fs::metadata(&path).await?;
                    attachment_files.push(ImportedFile {
                        source_path: path.to_string_lossy().to_string(),
                        target_path: target_path.to_string_lossy().to_string(),
                        file_type: FileType::Attachment,
                        size: metadata.len(),
                        modified_time: metadata.modified().map(chrono::DateTime::from).unwrap_or_else(|_| Utc::now()),
                        status: ImportStatus::Success,
                        transformations: Vec::new(),
                    });
                }
            }
        }
        
        Ok(())
    }


    fn is_markdown_or_canvas_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return matches!(ext_str.to_lowercase().as_str(), "md" | "markdown" | "canvas");
            }
        }
        false
    }
}