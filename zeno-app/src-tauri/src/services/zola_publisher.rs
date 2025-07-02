use crate::models::publisher::*;
use crate::models::note::Frontmatter;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tokio::time::Instant;
use regex::Regex;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde_yaml;
use pulldown_cmark::{Parser, html};

/// Zola 静态网站发布器
pub struct ZolaPublisher {
    config: ZolaConfig,
    site_path: PathBuf,
    template_engine: TemplateEngine,
}

impl ZolaPublisher {
    pub fn new(config: ZolaConfig, site_path: PathBuf) -> Result<Self> {
        let template_engine = TemplateEngine::new(&site_path)?;
        
        Ok(Self {
            config,
            site_path,
            template_engine,
        })
    }
    
    /// 初始化 Zola 站点结构
    pub async fn initialize_site(&self) -> Result<()> {
        let site_path = &self.site_path;
        
        // 创建 Zola 目录结构
        fs::create_dir_all(site_path.join("content")).await?;
        fs::create_dir_all(site_path.join("templates")).await?;
        fs::create_dir_all(site_path.join("static")).await?;
        fs::create_dir_all(site_path.join("sass")).await?;
        fs::create_dir_all(site_path.join("themes")).await?;
        
        // 生成 config.toml
        self.generate_config_file().await?;
        
        // 创建基础模板
        self.create_base_templates().await?;
        
        // 创建基础样式
        self.create_base_styles().await?;
        
        Ok(())
    }
    
    /// 发布笔记集合到静态网站
    pub async fn publish_notes(&self, notes: Vec<crate::models::note::Note>) -> Result<PublishResult> {
        let start_time = Instant::now();
        let mut published_files = Vec::new();
        let mut errors = Vec::new();
        
        // 清理旧内容
        self.clean_content_directory().await?;
        
        // 处理每个笔记
        for note in &notes {
            match self.process_note(note).await {
                Ok(file_path) => {
                    published_files.push(file_path);
                    log::info!("Published note: {}", note.title);
                }
                Err(e) => {
                    errors.push(PublishError {
                        file_path: note.path.clone(),
                        error_type: PublishErrorType::ContentProcessingError,
                        message: e.to_string(),
                    });
                    log::error!("Failed to publish note {}: {}", note.title, e);
                }
            }
        }
        
        // 处理资源文件
        if let Err(e) = self.copy_assets().await {
            errors.push(PublishError {
                file_path: self.site_path.join("static"),
                error_type: PublishErrorType::AssetCopyError,
                message: e.to_string(),
            });
        }
        
        // 构建网站
        let build_result = self.build_site().await?;
        let build_time = start_time.elapsed().as_secs_f64();
        
        // 计算站点大小
        let total_size = self.calculate_site_size().await?;
        
        Ok(PublishResult {
            success: errors.is_empty(),
            published_files,
            errors,
            build_output: build_result.output,
            site_url: Some(self.config.base_url.clone()),
            build_time,
            total_pages: notes.len(),
            total_size,
        })
    }
    
    /// 处理单个笔记
    async fn process_note(&self, note: &crate::models::note::Note) -> Result<PathBuf> {
        // 确定输出路径
        let output_path = self.determine_output_path(note)?;
        
        // 转换内容格式
        let zola_content = self.convert_to_zola_format(note).await?;
        
        // 处理图片和资源
        let processed_content = self.process_embedded_assets(&zola_content, note).await?;
        
        // 写入文件
        let full_path = self.site_path.join("content").join(&output_path);
        fs::create_dir_all(full_path.parent().unwrap()).await?;
        fs::write(&full_path, processed_content).await?;
        
        Ok(output_path)
    }
    
    /// 将笔记转换为 Zola 格式
    pub async fn convert_to_zola_format(&self, note: &crate::models::note::Note) -> Result<String> {
        let mut content = String::new();
        
        // 生成 TOML frontmatter
        content.push_str("+++\n");
        content.push_str(&format!("title = \"{}\"\n", escape_toml(&note.title)));
        content.push_str(&format!("date = {}\n", note.created_at.format("%Y-%m-%d")));
        
        // 获取frontmatter，如果为None则使用默认值
        let default_frontmatter = Frontmatter::default();
        let frontmatter = note.frontmatter.as_ref().unwrap_or(&default_frontmatter);
        
        if let Some(description) = &frontmatter.description {
            content.push_str(&format!("description = \"{}\"\n", escape_toml(description)));
        }
        
        // 处理标签和分类
        if !frontmatter.tags.is_empty() {
            content.push_str("\n[taxonomies]\n");
            content.push_str(&format!("tags = {:?}\n", frontmatter.tags));
        }
        
        if !frontmatter.categories.is_empty() {
            content.push_str(&format!("categories = {:?}\n", frontmatter.categories));
        }
        
        // 额外的 Zola 配置
        content.push_str(&format!("slug = \"{}\"\n", slugify(&note.title)));
        content.push_str(&format!("draft = {}\n", frontmatter.status == crate::models::note::NoteStatus::Draft));
        content.push_str(&format!("updated = {}\n", note.modified_at.format("%Y-%m-%d")));
        
        // 添加自定义字段
        for (key, value) in &frontmatter.extra {
            content.push_str(&format!("{} = {}\n", key, serde_json::to_string(value)?));
        }
        
        content.push_str("+++\n\n");
        
        // 处理正文内容
        let processed_body = self.process_content_for_zola(&note.content).await?;
        content.push_str(&processed_body);
        
        Ok(content)
    }
    
    /// 为 Zola 处理内容
    async fn process_content_for_zola(&self, content: &str) -> Result<String> {
        let mut processed = content.to_string();
        
        // 转换 Wiki 链接为 Zola 链接
        processed = self.convert_wiki_links(&processed).await?;
        
        // 处理嵌入式内容
        processed = self.process_embeds(&processed).await?;
        
        // 处理数学公式
        processed = self.process_math_blocks(&processed);
        
        // 处理代码块
        processed = self.process_code_blocks(&processed);
        
        Ok(processed)
    }
    
    /// 转换 Wiki 链接为 Zola 内部链接
    async fn convert_wiki_links(&self, content: &str) -> Result<String> {
        let wiki_link_regex = Regex::new(r"\[\[([^\]]+?)\]\]")?;
        let mut result = content.to_string();
        
        for captures in wiki_link_regex.captures_iter(content) {
            let full_match = &captures[0];
            let link_target = &captures[1];
            
            // 解析链接目标（可能包含别名）
            let (target, alias) = if link_target.contains('|') {
                let parts: Vec<&str> = link_target.splitn(2, '|').collect();
                (parts[0].trim(), Some(parts[1].trim()))
            } else {
                (link_target, None)
            };
            
            // 转换为 Zola 链接格式
            let zola_link = if let Some(alias_text) = alias {
                format!("[{}](../{})", alias_text, slugify(target))
            } else {
                format!("[{}](../{})", target, slugify(target))
            };
            
            result = result.replace(full_match, &zola_link);
        }
        
        Ok(result)
    }
    
    /// 处理嵌入内容
    async fn process_embeds(&self, content: &str) -> Result<String> {
        let embed_regex = Regex::new(r"!\[\[([^\]]+?)\]\]")?;
        let mut result = content.to_string();
        
        for captures in embed_regex.captures_iter(content) {
            let full_match = &captures[0];
            let embed_target = &captures[1];
            
            // 根据文件类型处理嵌入
            if embed_target.ends_with(".png") || embed_target.ends_with(".jpg") || 
               embed_target.ends_with(".jpeg") || embed_target.ends_with(".gif") {
                // 图片嵌入
                let img_tag = format!("![{}]({})", embed_target, embed_target);
                result = result.replace(full_match, &img_tag);
            } else {
                // 文档嵌入（暂时替换为链接）
                let link = format!("[{}](../{})", embed_target, slugify(embed_target));
                result = result.replace(full_match, &link);
            }
        }
        
        Ok(result)
    }
    
    /// 处理数学公式块
    fn process_math_blocks(&self, content: &str) -> String {
        let math_block_regex = Regex::new(r"\$\$([^$]+?)\$\$").unwrap();
        let inline_math_regex = Regex::new(r"\$([^$]+?)\$").unwrap();
        
        let mut result = content.to_string();
        
        // 处理块级数学公式
        result = math_block_regex.replace_all(&result, |caps: &regex::Captures| {
            format!("$${}$$", &caps[1])
        }).to_string();
        
        // 处理行内数学公式
        result = inline_math_regex.replace_all(&result, |caps: &regex::Captures| {
            format!("${}$", &caps[1])
        }).to_string();
        
        result
    }
    
    /// 处理代码块
    fn process_code_blocks(&self, content: &str) -> String {
        // Zola 原生支持 Markdown 代码块，无需特殊处理
        content.to_string()
    }
    
    /// 确定笔记的输出路径
    fn determine_output_path(&self, note: &crate::models::note::Note) -> Result<PathBuf> {
        let slug = slugify(&note.title);
        
        // 根据分类组织目录结构
        let mut path = PathBuf::new();
        
        if let Some(frontmatter) = &note.frontmatter {
            if !frontmatter.categories.is_empty() {
                for category in &frontmatter.categories {
                    path.push(slugify(category));
                }
            }
        }
        
        path.push(format!("{}.md", slug));
        Ok(path)
    }
    
    /// 处理嵌入的资源文件
    async fn process_embedded_assets(&self, content: &str, note: &crate::models::note::Note) -> Result<String> {
        // 查找内容中引用的图片
        let img_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)")?;
        let mut result = content.to_string();
        
        for captures in img_regex.captures_iter(content) {
            let _alt_text = &captures[1];
            let img_path = &captures[2];
            
            // 如果是相对路径，复制到静态资源目录
            if !img_path.starts_with("http") {
                let source_path = note.path.parent().unwrap().join(img_path);
                if source_path.exists() {
                    let filename = source_path.file_name().unwrap();
                    let dest_path = self.site_path.join("static").join("images").join(filename);
                    
                    fs::create_dir_all(dest_path.parent().unwrap()).await?;
                    fs::copy(&source_path, &dest_path).await?;
                    
                    // 更新内容中的路径
                    let new_path = format!("/images/{}", filename.to_string_lossy());
                    result = result.replace(img_path, &new_path);
                }
            }
        }
        
        Ok(result)
    }
    
    /// 生成 config.toml 文件
    async fn generate_config_file(&self) -> Result<()> {
        let config_content = format!(
            r#"# The URL the site will be built for
base_url = "{}"

# The site title and description
title = "{}"
description = "{}"

# The default author
author = "{}"

# The default language
default_language = "{}"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = {}

# Whether to generate a RSS feed automatically
generate_rss = {}

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = {}

# Configuration of the link checker
[link_checker]
# Skip link checking for external URLs that start with these prefixes
skip_prefixes = ["http://[2001:db8::]/*"]

# Skip anchor checking for external URLs that start with these prefixes
skip_anchor_prefixes = ["https://caniuse.com/"]

# Various slugification strategies
[slugify]
paths = "on"
taxonomies = "on"
anchors = "on"

# Optional translation object
[translations]

# When set to "true", the generated HTML files are minified
minify_html = false

# A list of glob patterns specifying asset files to ignore when the content
# directory is processed. Defaults to none, which means that all asset files are
# copied over to the `public` directory.
ignored_content = []

# When set to "true", a feed is automatically generated.
generate_feed = {}

# The filename to use for the feed. Used as the template filename, too.
feed_filename = "atom.xml"

# The number of articles to include in the feed. All items are included if
# this limit is not set (the default).
# feed_limit = 20

# When set to "true", files in the `static` directory are hard-linked. Useful for large
# static files. Note that for this to work, both `static` and output directory need to be on
# the same filesystem. Note that the theme's `static` files are always copied.
hard_link_static = false

# The taxonomies to be rendered for the site and their configuration of the default languages
# Example:
#     taxonomies = [
#       {{name = "tags", feed = true}}, # each tag will have its own feed
#       {{name = "tags"}}, # you can have taxonomies with the same name in multiple languages
#       {{name = "categories", paginate_by = 5}},  # 5 items per page for a term
#       {{name = "authors"}}, # Basic definition: no feed or pagination
#     ]
#
taxonomies = [
    {{name = "tags", feed = true}},
    {{name = "categories", feed = true}},
]

# When set to "true", the Sass files in the `sass` directory in the site root are compiled.
# Sass files in theme directories are always compiled.
compile_sass = {}

# Markdown configuration
[markdown]
# Whether to do syntax highlighting
# Theme can be customised by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = {}

# A list of directories used to search for additional `.sublime-syntax` and `.tmTheme` files.
extra_syntaxes_and_themes = []

# The theme to use for code highlighting.
# See below for list of allowed values.
highlight_theme = "{}"

# When set to "true", emoji aliases translated to their corresponding
# Unicode emoji equivalent in the rendered Markdown files. (e.g.: :smile: => 😄)
render_emoji = {}

# Whether external links are to be opened in a new tab
# If this is true, a `rel="noopener"` will always be added for security reasons
external_links_target_blank = {}

# Whether to set rel="nofollow" for all external links
external_links_no_follow = {}

# Whether to set rel="noreferrer" for all external links
external_links_no_referrer = {}

# Whether smart punctuation is enabled (changing quotes, dashes, dots etc in their typographic form)
# For example, `...` into `…`, `"quote"` into `"curly"` etc
smart_punctuation = {}

# Configuration of the search functionality
# If `build_search_index` is set to true, a search index is built from the pages and section
# content for `default_language`.
[search]
# Whether to include the title of the page/section in the index
include_title = true
# Whether to include the description of the page/section in the index
include_description = false
# Whether to include the path of the page/section in the index
include_path = false
# Whether to include the rendered content of the page/section in the index
include_content = true
# At which character to truncate the content when indexing. Useful if you have a lot of pages and the index would
# become too big to load on the site. Defaults to not being set.
# truncate_content_length = 100

# Optional translation object for the default language
# Example:
#     default_language = "fr"
#
#     [translations]
#     title = "Un titre"
#
[translations]

# Configuration of the Rust code syntax highlighting
[extra]
# Put all your custom variables here
"#,
            self.config.base_url,
            self.config.title,
            self.config.description,
            self.config.author,
            self.config.default_language,
            self.config.compile_sass,
            self.config.generate_rss,
            self.config.build_search_index,
            self.config.generate_rss,
            self.config.compile_sass,
            self.config.markdown.highlight_code,
            self.config.markdown.highlight_theme,
            self.config.markdown.render_emoji,
            self.config.markdown.external_links_target_blank,
            self.config.markdown.external_links_no_follow,
            self.config.markdown.external_links_no_referrer,
            self.config.markdown.smart_punctuation,
        );
        
        let config_path = self.site_path.join("config.toml");
        fs::write(config_path, config_content).await?;
        
        Ok(())
    }
    
    /// 创建基础模板
    async fn create_base_templates(&self) -> Result<()> {
        let templates_dir = self.site_path.join("templates");
        
        // base.html 基础模板
        let base_template = include_str!("../templates/base.html");
        fs::write(templates_dir.join("base.html"), base_template).await?;
        
        // index.html 首页模板
        let index_template = include_str!("../templates/index.html");
        fs::write(templates_dir.join("index.html"), index_template).await?;
        
        // page.html 页面模板
        let page_template = include_str!("../templates/page.html");
        fs::write(templates_dir.join("page.html"), page_template).await?;
        
        // section.html 分区模板
        let section_template = include_str!("../templates/section.html");
        fs::write(templates_dir.join("section.html"), section_template).await?;
        
        Ok(())
    }
    
    /// 创建基础样式
    async fn create_base_styles(&self) -> Result<()> {
        let sass_dir = self.site_path.join("sass");
        fs::create_dir_all(&sass_dir).await?;
        
        let main_scss = include_str!("../styles/main.scss");
        fs::write(sass_dir.join("main.scss"), main_scss).await?;
        
        Ok(())
    }
    
    /// 清理内容目录
    async fn clean_content_directory(&self) -> Result<()> {
        let content_dir = self.site_path.join("content");
        if content_dir.exists() {
            fs::remove_dir_all(&content_dir).await?;
            fs::create_dir_all(&content_dir).await?;
        }
        Ok(())
    }
    
    /// 复制资源文件
    async fn copy_assets(&self) -> Result<()> {
        // 这里可以添加复制其他资源文件的逻辑
        Ok(())
    }
    
    /// 构建站点
    async fn build_site(&self) -> Result<BuildResult> {
        let output = Command::new("zola")
            .current_dir(&self.site_path)
            .arg("build")
            .arg("--output-dir")
            .arg("public")
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Zola build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(BuildResult {
            success: true,
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
    
    /// 计算站点大小
    async fn calculate_site_size(&self) -> Result<u64> {
        let public_dir = self.site_path.join("public");
        if !public_dir.exists() {
            return Ok(0);
        }
        
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&public_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += self.calculate_dir_size(&entry.path())?;
            }
        }
        
        Ok(total_size)
    }
    
    fn calculate_dir_size(&self, dir: &Path) -> Result<u64> {
        use std::fs;
        let mut total_size = 0u64;
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += self.calculate_dir_size(&entry.path())?;
            }
        }
        
        Ok(total_size)
    }
}

/// 构建结果
#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub output: String,
}

/// 模板引擎
pub struct TemplateEngine {
    _site_path: PathBuf,
}

impl TemplateEngine {
    pub fn new(site_path: &Path) -> Result<Self> {
        Ok(Self {
            _site_path: site_path.to_path_buf(),
        })
    }
}

/// 工具函数
fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}