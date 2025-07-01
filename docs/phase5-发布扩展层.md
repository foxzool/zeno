# Phase 5: 发布扩展层开发计划

## 阶段概述

实现Noto的发布生态系统，包括多平台发布、导入导出功能和插件系统框架。这个阶段将Noto从个人知识管理工具扩展为完整的内容创作发布平台。

**预计时间**: 4-6周  
**优先级**: 中高 (产品差异化)  
**前置条件**: Phase 4知识网络层完成

## 目标与交付物

### 主要目标
- 实现一键发布到多个平台
- 建立完善的导入导出系统
- 构建可扩展的插件架构
- 提供内容格式转换和适配

### 交付物
- 静态网站生成系统 (Zola集成)
- 第三方平台发布适配器
- 数据导入导出工具
- 插件系统框架和API

## 详细任务清单

### 5.1 静态网站生成

**任务描述**: 集成Zola实现高质量的静态网站生成

**Zola集成架构**:
```rust
// services/publisher/zola.rs
use std::process::Command;
use tokio::fs;
use serde_yaml;

pub struct ZolaPublisher {
    config: ZolaConfig,
    template_engine: TemplateEngine,
    asset_processor: AssetProcessor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZolaConfig {
    pub site_path: PathBuf,
    pub base_url: String,
    pub title: String,
    pub description: String,
    pub theme: String,
    pub build_drafts: bool,
    pub generate_rss: bool,
    pub taxonomies: Vec<Taxonomy>,
    pub markdown: MarkdownConfig,
    pub search: SearchConfig,
}

impl ZolaPublisher {
    pub async fn new(config: ZolaConfig) -> Result<Self> {
        let template_engine = TemplateEngine::new(&config.site_path)?;
        let asset_processor = AssetProcessor::new(&config.site_path)?;
        
        Ok(Self {
            config,
            template_engine,
            asset_processor,
        })
    }
    
    pub async fn initialize_site(&self) -> Result<()> {
        // 创建 Zola 项目结构
        let site_path = &self.config.site_path;
        
        fs::create_dir_all(site_path.join("content")).await?;
        fs::create_dir_all(site_path.join("templates")).await?;
        fs::create_dir_all(site_path.join("static")).await?;
        fs::create_dir_all(site_path.join("sass")).await?;
        fs::create_dir_all(site_path.join("themes")).await?;
        
        // 生成 config.toml
        self.generate_config_file().await?;
        
        // 安装主题
        if !self.config.theme.is_empty() {
            self.install_theme(&self.config.theme).await?;
        }
        
        // 创建基础模板
        self.create_base_templates().await?;
        
        Ok(())
    }
    
    pub async fn publish_notes(&self, notes: Vec<Note>) -> Result<PublishResult> {
        let mut published_files = Vec::new();
        let mut errors = Vec::new();
        
        // 清理旧内容
        self.clean_content_directory().await?;
        
        // 处理每个笔记
        for note in notes {
            match self.process_note(&note).await {
                Ok(file_path) => {
                    published_files.push(file_path);
                    log::info!("Published note: {}", note.title);
                }
                Err(e) => {
                    errors.push(PublishError {
                        note_id: note.id.clone(),
                        error: e.to_string(),
                    });
                    log::error!("Failed to publish note {}: {}", note.title, e);
                }
            }
        }
        
        // 处理资源文件
        self.copy_assets().await?;
        
        // 构建网站
        let build_result = self.build_site().await?;
        
        Ok(PublishResult {
            success: errors.is_empty(),
            published_files,
            errors,
            build_output: build_result.output,
            site_url: self.get_site_url(),
        })
    }
    
    async fn process_note(&self, note: &Note) -> Result<PathBuf> {
        // 确定输出路径
        let output_path = self.determine_output_path(note)?;
        
        // 转换内容格式
        let zola_content = self.convert_to_zola_format(note).await?;
        
        // 处理图片和资源
        let processed_content = self.process_embedded_assets(&zola_content, note).await?;
        
        // 写入文件
        let full_path = self.config.site_path.join("content").join(&output_path);
        fs::create_dir_all(full_path.parent().unwrap()).await?;
        fs::write(&full_path, processed_content).await?;
        
        Ok(output_path)
    }
    
    async fn convert_to_zola_format(&self, note: &Note) -> Result<String> {
        let mut content = String::new();
        
        // 生成 TOML frontmatter
        content.push_str("+++\n");
        content.push_str(&format!("title = \"{}\"\n", escape_toml(&note.title)));
        content.push_str(&format!("date = {}\n", note.frontmatter.date.format("%Y-%m-%d")));
        
        if let Some(description) = &note.frontmatter.description {
            content.push_str(&format!("description = \"{}\"\n", escape_toml(description)));
        }
        
        // 处理标签
        if !note.frontmatter.tags.is_empty() {
            content.push_str("\\n[taxonomies]\\n");
            content.push_str(&format!("tags = {:?}\\n", note.frontmatter.tags));
        }
        
        // 处理分类
        if !note.frontmatter.categories.is_empty() {
            content.push_str(&format!("categories = {:?}\\n", note.frontmatter.categories));
        }
        
        // 额外的 Zola 配置
        content.push_str(&format!("slug = \"{}\"\\n", slugify(&note.title)));
        content.push_str(&format!("draft = {}\\n", note.frontmatter.status == NoteStatus::Draft));
        
        if let Some(template) = &note.frontmatter.template {
            content.push_str(&format!("template = \"{}\"\\n", template));
        }
        
        content.push_str("+++\\n\\n");
        
        // 处理正文内容
        let processed_body = self.process_content_for_zola(&note.content).await?;
        content.push_str(&processed_body);
        
        Ok(content)
    }
    
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
        
        // 处理自定义语法
        processed = self.process_custom_syntax(&processed);
        
        Ok(processed)
    }
    
    async fn build_site(&self) -> Result<BuildResult> {
        let output = Command::new("zola")
            .current_dir(&self.config.site_path)
            .arg("build")
            .arg("--output-dir")
            .arg("public")
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Zola build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(BuildResult {
            success: true,
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            build_time: std::time::Instant::now(),
        })
    }
}
```

**主题系统设计**:
```rust
// services/theme_manager.rs
pub struct ThemeManager {
    themes_dir: PathBuf,
    custom_themes: HashMap<String, ThemeConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub min_zola_version: String,
    pub demo_url: Option<String>,
    pub screenshot: Option<String>,
    pub features: Vec<String>,
    pub variables: HashMap<String, ThemeVariable>,
}

impl ThemeManager {
    pub async fn install_theme(&self, theme_name: &str) -> Result<()> {
        // 从官方仓库或 GitHub 安装主题
        if self.is_official_theme(theme_name) {
            self.install_official_theme(theme_name).await
        } else if theme_name.contains('/') {
            // GitHub 仓库格式: user/repo
            self.install_github_theme(theme_name).await
        } else {
            // 本地主题
            self.install_local_theme(theme_name).await
        }
    }
    
    pub async fn customize_theme(&self, theme_name: &str, customizations: ThemeCustomization) -> Result<()> {
        let theme_dir = self.themes_dir.join(theme_name);
        
        // 修改样式变量
        if !customizations.variables.is_empty() {
            self.update_theme_variables(&theme_dir, &customizations.variables).await?;
        }
        
        // 覆盖模板
        for (template_name, template_content) in &customizations.templates {
            let template_path = theme_dir.join("templates").join(template_name);
            fs::write(template_path, template_content).await?;
        }
        
        // 添加自定义 CSS
        if let Some(custom_css) = &customizations.custom_css {
            let css_path = theme_dir.join("sass").join("custom.scss");
            fs::write(css_path, custom_css).await?;
        }
        
        Ok(())
    }
}
```

**验收标准**:
- [ ] 网站构建时间 < 30秒 (1000篇文章)
- [ ] 生成的网站SEO友好
- [ ] 支持多种主题切换
- [ ] 自动部署到托管平台
- [ ] 响应式设计和移动端适配

### 5.2 第三方平台发布

**任务描述**: 实现多平台发布适配器

**发布适配器架构**:
```rust
// services/publisher/mod.rs
#[async_trait]
pub trait Publisher: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    async fn publish(&self, content: &PublishContent, options: &PublishOptions) -> Result<PublishResult>;
    async fn update(&self, content_id: &str, content: &PublishContent) -> Result<PublishResult>;
    async fn delete(&self, content_id: &str) -> Result<()>;
    async fn get_status(&self, content_id: &str) -> Result<PublishStatus>;
    async fn list_published(&self) -> Result<Vec<PublishedContent>>;
    
    fn platform_name(&self) -> &'static str;
    fn supported_formats(&self) -> Vec<ContentFormat>;
    fn max_content_length(&self) -> Option<usize>;
    fn supports_images(&self) -> bool;
    fn supports_formatting(&self) -> Vec<FormattingFeature>;
}

pub struct PublisherRegistry {
    publishers: HashMap<String, Box<dyn Publisher>>,
}

impl PublisherRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            publishers: HashMap::new(),
        };
        
        // 注册内置发布器
        registry.register("wechat", Box::new(WeChatPublisher::new()));
        registry.register("zhihu", Box::new(ZhihuPublisher::new()));
        registry.register("medium", Box::new(MediumPublisher::new()));
        registry.register("dev_to", Box::new(DevToPublisher::new()));
        registry.register("hashnode", Box::new(HashnodePublisher::new()));
        
        registry
    }
    
    pub async fn publish_to_platforms(
        &self,
        content: &PublishContent,
        platforms: Vec<String>,
        options: PublishOptions,
    ) -> Result<Vec<PublishResult>> {
        let mut results = Vec::new();
        
        // 并行发布到多个平台
        let futures: Vec<_> = platforms.iter().map(|platform| {
            async move {
                if let Some(publisher) = self.publishers.get(platform) {
                    let platform_content = self.adapt_content_for_platform(content, platform).await?;
                    publisher.publish(&platform_content, &options).await
                } else {
                    Err(anyhow!("Unknown platform: {}", platform))
                }
            }
        }).collect();
        
        let platform_results = futures::future::join_all(futures).await;
        
        for (i, result) in platform_results.into_iter().enumerate() {
            match result {
                Ok(publish_result) => results.push(publish_result),
                Err(e) => {
                    results.push(PublishResult {
                        platform: platforms[i].clone(),
                        success: false,
                        error: Some(e.to_string()),
                        ..Default::default()
                    });
                }
            }
        }
        
        Ok(results)
    }
}
```

**微信公众号发布器**:
```rust
// services/publisher/wechat.rs
pub struct WeChatPublisher {
    client: reqwest::Client,
    app_id: String,
    app_secret: String,
    access_token: Option<String>,
    token_expires_at: Option<Instant>,
}

impl WeChatPublisher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            app_id: std::env::var("WECHAT_APP_ID").unwrap_or_default(),
            app_secret: std::env::var("WECHAT_APP_SECRET").unwrap_or_default(),
            access_token: None,
            token_expires_at: None,
        }
    }
    
    async fn ensure_access_token(&mut self) -> Result<&str> {
        if let Some(token) = &self.access_token {
            if let Some(expires_at) = self.token_expires_at {
                if Instant::now() < expires_at {
                    return Ok(token);
                }
            }
        }
        
        // 获取新的 access_token
        let response = self.client
            .get("https://api.weixin.qq.com/cgi-bin/token")
            .query(&[
                ("grant_type", "client_credential"),
                ("appid", &self.app_id),
                ("secret", &self.app_secret),
            ])
            .send()
            .await?;
        
        let token_response: TokenResponse = response.json().await?;
        
        if let Some(errcode) = token_response.errcode {
            return Err(anyhow!("WeChat API error {}: {}", errcode, 
                token_response.errmsg.unwrap_or_default()));
        }
        
        self.access_token = Some(token_response.access_token.unwrap());
        self.token_expires_at = Some(Instant::now() + Duration::from_secs(7000)); // 提前200秒过期
        
        Ok(self.access_token.as_ref().unwrap())
    }
    
    async fn upload_media(&mut self, media_path: &Path) -> Result<String> {
        let access_token = self.ensure_access_token().await?;
        
        let file = tokio::fs::File::open(media_path).await?;
        let file_size = file.metadata().await?.len();
        
        // 检查文件大小限制
        if file_size > 10 * 1024 * 1024 { // 10MB
            return Err(anyhow!("File too large for WeChat: {} bytes", file_size));
        }
        
        let form = reqwest::multipart::Form::new()
            .part("media", reqwest::multipart::Part::stream(file)
                .file_name(media_path.file_name().unwrap().to_string_lossy().to_string())
                .mime_str("image/jpeg")?);
        
        let response = self.client
            .post(&format!(
                "https://api.weixin.qq.com/cgi-bin/media/upload?access_token={}&type=image",
                access_token
            ))
            .multipart(form)
            .send()
            .await?;
        
        let upload_response: MediaUploadResponse = response.json().await?;
        
        if let Some(errcode) = upload_response.errcode {
            return Err(anyhow!("Media upload failed {}: {}", errcode,
                upload_response.errmsg.unwrap_or_default()));
        }
        
        Ok(upload_response.media_id.unwrap())
    }
    
    fn format_content_for_wechat(&self, content: &PublishContent) -> Result<String> {
        let mut formatted = String::new();
        
        // 添加标题
        formatted.push_str(&format!("# {}\\n\\n", content.title));
        
        // 处理正文
        let mut body = content.body.clone();
        
        // 转换 Markdown 到微信富文本
        body = self.convert_markdown_to_wechat_html(&body)?;
        
        // 处理图片
        body = self.process_images_for_wechat(&body)?;
        
        // 添加标签
        if !content.tags.is_empty() {
            body.push_str("\\n\\n---\\n\\n");
            body.push_str(&format!("标签: {}\\n", content.tags.join(" · ")));
        }
        
        formatted.push_str(&body);
        Ok(formatted)
    }
}

#[async_trait]
impl Publisher for WeChatPublisher {
    async fn publish(&self, content: &PublishContent, options: &PublishOptions) -> Result<PublishResult> {
        let formatted_content = self.format_content_for_wechat(content)?;
        
        // 创建草稿
        let draft_response = self.create_draft(&formatted_content).await?;
        
        // 如果设置了自动发布，则立即发布
        if options.auto_publish {
            self.publish_draft(&draft_response.media_id).await?;
        }
        
        Ok(PublishResult {
            platform: "wechat".to_string(),
            success: true,
            content_id: Some(draft_response.media_id),
            url: None, // 微信公众号没有直接URL
            published_at: Some(Utc::now()),
            ..Default::default()
        })
    }
    
    fn platform_name(&self) -> &'static str {
        "微信公众号"
    }
    
    fn supported_formats(&self) -> Vec<ContentFormat> {
        vec![ContentFormat::Html, ContentFormat::Markdown]
    }
    
    fn max_content_length(&self) -> Option<usize> {
        Some(20000) // 微信公众号字数限制
    }
    
    fn supports_images(&self) -> bool {
        true
    }
    
    fn supports_formatting(&self) -> Vec<FormattingFeature> {
        vec![
            FormattingFeature::Bold,
            FormattingFeature::Italic,
            FormattingFeature::Headers,
            FormattingFeature::Lists,
            FormattingFeature::Links,
            FormattingFeature::Images,
        ]
    }
}
```

**内容适配引擎**:
```rust
// services/content_adapter.rs
pub struct ContentAdapter {
    platform_configs: HashMap<String, PlatformConfig>,
}

#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub max_title_length: Option<usize>,
    pub max_content_length: Option<usize>,
    pub supported_formats: Vec<ContentFormat>,
    pub image_restrictions: ImageRestrictions,
    pub custom_formatting: HashMap<String, String>,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

impl ContentAdapter {
    pub async fn adapt_content(
        &self,
        content: &PublishContent,
        platform: &str,
    ) -> Result<PublishContent> {
        let config = self.platform_configs.get(platform)
            .ok_or_else(|| anyhow!("Unknown platform: {}", platform))?;
        
        let mut adapted = content.clone();
        
        // 适配标题长度
        if let Some(max_len) = config.max_title_length {
            if adapted.title.len() > max_len {
                adapted.title = format!("{}...", &adapted.title[..max_len-3]);
            }
        }
        
        // 适配内容长度
        if let Some(max_len) = config.max_content_length {
            if adapted.body.len() > max_len {
                adapted.body = self.smart_truncate(&adapted.body, max_len)?;
            }
        }
        
        // 转换格式
        adapted.body = self.convert_format(&adapted.body, &config.supported_formats[0])?;
        
        // 处理图片
        adapted.body = self.adapt_images(&adapted.body, &config.image_restrictions).await?;
        
        // 应用平台特定格式化
        for (pattern, replacement) in &config.custom_formatting {
            adapted.body = adapted.body.replace(pattern, replacement);
        }
        
        Ok(adapted)
    }
    
    fn smart_truncate(&self, content: &str, max_length: usize) -> Result<String> {
        if content.len() <= max_length {
            return Ok(content.to_string());
        }
        
        // 尝试在段落边界截断
        let paragraphs: Vec<&str> = content.split("\\n\\n").collect();
        let mut result = String::new();
        
        for paragraph in paragraphs {
            if result.len() + paragraph.len() + 2 <= max_length {
                if !result.is_empty() {
                    result.push_str("\\n\\n");
                }
                result.push_str(paragraph);
            } else {
                break;
            }
        }
        
        // 如果仍然太长，在句子边界截断
        if result.is_empty() || result.len() < max_length / 2 {
            result = content.chars().take(max_length - 3).collect();
            result.push_str("...");
        } else {
            result.push_str("\\n\\n...");
        }
        
        Ok(result)
    }
}
```

**验收标准**:
- [ ] 至少支持3个主流平台发布
- [ ] 内容格式自动适配正确
- [ ] 图片上传和处理正常
- [ ] 发布状态跟踪准确
- [ ] 批量发布性能良好

### 5.3 导入导出功能

**任务描述**: 实现数据的导入导出和格式转换

**导入系统设计**:
```rust
// services/importer/mod.rs
#[async_trait]
pub trait Importer: Send + Sync {
    async fn can_import(&self, source: &ImportSource) -> bool;
    async fn import(&self, source: &ImportSource, options: &ImportOptions) -> Result<ImportResult>;
    fn source_name(&self) -> &'static str;
    fn supported_formats(&self) -> Vec<String>;
}

pub struct ImporterRegistry {
    importers: HashMap<String, Box<dyn Importer>>,
}

impl ImporterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            importers: HashMap::new(),
        };
        
        // 注册导入器
        registry.register("obsidian", Box::new(ObsidianImporter::new()));
        registry.register("notion", Box::new(NotionImporter::new()));
        registry.register("roam", Box::new(RoamImporter::new()));
        registry.register("logseq", Box::new(LogseqImporter::new()));
        registry.register("joplin", Box::new(JoplinImporter::new()));
        registry.register("markdown", Box::new(MarkdownImporter::new()));
        registry.register("html", Box::new(HtmlImporter::new()));
        registry.register("docx", Box::new(DocxImporter::new()));
        
        registry
    }
    
    pub async fn auto_detect_format(&self, source: &ImportSource) -> Result<String> {
        for (name, importer) in &self.importers {
            if importer.can_import(source).await {
                return Ok(name.clone());
            }
        }
        
        Err(anyhow!("Cannot detect import format"))
    }
    
    pub async fn import_with_progress(
        &self,
        source: &ImportSource,
        format: &str,
        options: ImportOptions,
        progress_callback: Box<dyn Fn(ImportProgress) + Send + Sync>,
    ) -> Result<ImportResult> {
        let importer = self.importers.get(format)
            .ok_or_else(|| anyhow!("Unknown import format: {}", format))?;
        
        // 启动进度监控
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
        
        // 在后台处理进度更新
        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = progress_rx.recv().await {
                progress_callback(progress);
            }
        });
        
        // 执行导入
        let result = importer.import(source, &options).await;
        
        // 清理
        drop(progress_tx);
        progress_handle.await?;
        
        result
    }
}
```

**Obsidian导入器实现**:
```rust
// services/importer/obsidian.rs
pub struct ObsidianImporter {
    link_resolver: ObsidianLinkResolver,
    attachment_processor: AttachmentProcessor,
}

impl ObsidianImporter {
    pub fn new() -> Self {
        Self {
            link_resolver: ObsidianLinkResolver::new(),
            attachment_processor: AttachmentProcessor::new(),
        }
    }
    
    async fn scan_vault(&self, vault_path: &Path) -> Result<ObsidianVault> {
        let mut vault = ObsidianVault::new();
        
        // 扫描配置文件
        let obsidian_dir = vault_path.join(".obsidian");
        if obsidian_dir.exists() {
            vault.config = self.load_obsidian_config(&obsidian_dir).await?;
        }
        
        // 扫描笔记文件
        let mut walker = WalkDir::new(vault_path);
        walker = walker.filter_entry(|entry| {
            // 跳过 .obsidian 目录
            !entry.path().components().any(|c| c.as_os_str() == ".obsidian")
        });
        
        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "md") {
                let note = self.parse_obsidian_note(path).await?;
                vault.notes.push(note);
            } else if self.is_attachment_file(path) {
                vault.attachments.push(path.to_path_buf());
            }
        }
        
        // 解析链接关系
        vault.links = self.extract_all_links(&vault.notes)?;
        
        Ok(vault)
    }
    
    async fn parse_obsidian_note(&self, path: &Path) -> Result<ObsidianNote> {
        let content = fs::read_to_string(path).await?;
        
        // 解析 frontmatter
        let (frontmatter, body) = if content.starts_with("---\\n") {
            let end_pos = content[4..].find("\\n---\\n")
                .ok_or_else(|| anyhow!("Invalid frontmatter"))?;
            
            let frontmatter_content = &content[4..end_pos + 4];
            let body_content = &content[end_pos + 8..];
            
            let frontmatter: serde_yaml::Value = serde_yaml::from_str(frontmatter_content)?;
            (Some(frontmatter), body_content.to_string())
        } else {
            (None, content)
        };
        
        // 提取 Obsidian 特有的元素
        let tags = self.extract_obsidian_tags(&body)?;
        let links = self.link_resolver.extract_links(&body)?;
        let embeds = self.extract_embeds(&body)?;
        let aliases = frontmatter.as_ref()
            .and_then(|fm| fm.get("aliases"))
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        Ok(ObsidianNote {
            path: path.to_path_buf(),
            title: path.file_stem().unwrap().to_string_lossy().to_string(),
            content: body,
            frontmatter,
            tags,
            links,
            embeds,
            aliases,
            created_at: self.get_file_created_time(path)?,
            modified_at: self.get_file_modified_time(path)?,
        })
    }
    
    async fn convert_to_noto_format(&self, obsidian_note: &ObsidianNote) -> Result<Note> {
        let mut content = obsidian_note.content.clone();
        
        // 转换 Obsidian 链接语法
        content = self.convert_obsidian_links(&content)?;
        
        // 转换嵌入语法
        content = self.convert_obsidian_embeds(&content)?;
        
        // 转换标签语法
        content = self.convert_obsidian_tags(&content)?;
        
        // 处理 Obsidian 特有的语法
        content = self.convert_obsidian_callouts(&content)?;
        content = self.convert_obsidian_dataview(&content)?;
        
        // 构建 Zeno frontmatter
        let mut frontmatter = Frontmatter {
            title: obsidian_note.title.clone(),
            date: obsidian_note.created_at.date_naive(),
            tags: obsidian_note.tags.clone(),
            categories: Vec::new(), // 从路径推断
            status: NoteStatus::Active,
            publish: None,
            extra: HashMap::new(),
        };
        
        // 从 Obsidian frontmatter 转换额外字段
        if let Some(obs_fm) = &obsidian_note.frontmatter {
            if let Some(categories) = obs_fm.get("categories") {
                if let Some(cats) = categories.as_sequence() {
                    frontmatter.categories = cats.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            
            // 保留其他自定义字段
            for (key, value) in obs_fm.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
                if let Some(key_str) = key.as_str() {
                    if !["title", "date", "tags", "categories"].contains(&key_str) {
                        frontmatter.extra.insert(key_str.to_string(), 
                            serde_json::to_value(value)?);
                    }
                }
            }
        }
        
        Ok(Note {
            id: Uuid::new_v4().to_string(),
            path: obsidian_note.path.clone(),
            title: frontmatter.title.clone(),
            content,
            frontmatter,
            created_at: obsidian_note.created_at,
            modified_at: obsidian_note.modified_at,
            checksum: self.calculate_checksum(&content),
        })
    }
}

#[async_trait]
impl Importer for ObsidianImporter {
    async fn can_import(&self, source: &ImportSource) -> bool {
        match source {
            ImportSource::Directory(path) => {
                // 检查是否为 Obsidian vault
                path.join(".obsidian").exists() || 
                self.contains_obsidian_files(path).await.unwrap_or(false)
            },
            ImportSource::Archive(path) => {
                // 检查压缩包内容
                self.archive_contains_obsidian_structure(path).await.unwrap_or(false)
            },
            _ => false,
        }
    }
    
    async fn import(&self, source: &ImportSource, options: &ImportOptions) -> Result<ImportResult> {
        let vault_path = match source {
            ImportSource::Directory(path) => path.clone(),
            ImportSource::Archive(path) => {
                // 解压到临时目录
                self.extract_archive(path, &options.temp_dir).await?
            },
            _ => return Err(anyhow!("Unsupported source type for Obsidian import")),
        };
        
        // 扫描 Obsidian vault
        let vault = self.scan_vault(&vault_path).await?;
        
        let mut imported_notes = Vec::new();
        let mut import_errors = Vec::new();
        let total_notes = vault.notes.len();
        
        // 转换笔记
        for (i, obsidian_note) in vault.notes.iter().enumerate() {
            match self.convert_to_noto_format(obsidian_note).await {
                Ok(note) => {
                    imported_notes.push(note);
                },
                Err(e) => {
                    import_errors.push(ImportError {
                        file_path: obsidian_note.path.clone(),
                        error: e.to_string(),
                    });
                }
            }
            
            // 报告进度
            if let Some(progress_callback) = &options.progress_callback {
                progress_callback(ImportProgress {
                    processed: i + 1,
                    total: total_notes,
                    current_file: obsidian_note.path.to_string_lossy().to_string(),
                    stage: ImportStage::ConvertingNotes,
                });
            }
        }
        
        // 处理附件
        let mut copied_attachments = 0;
        for attachment in &vault.attachments {
            if let Err(e) = self.copy_attachment(attachment, &options.target_dir).await {
                import_errors.push(ImportError {
                    file_path: attachment.clone(),
                    error: e.to_string(),
                });
            } else {
                copied_attachments += 1;
            }
        }
        
        Ok(ImportResult {
            success: import_errors.is_empty(),
            imported_notes_count: imported_notes.len(),
            imported_attachments_count: copied_attachments,
            errors: import_errors,
            notes: imported_notes,
            conversion_log: self.generate_conversion_log(&vault),
        })
    }
    
    fn source_name(&self) -> &'static str {
        "Obsidian"
    }
    
    fn supported_formats(&self) -> Vec<String> {
        vec!["obsidian".to_string(), "md".to_string()]
    }
}
```

**导出系统设计**:
```rust
// services/exporter/mod.rs
#[async_trait]
pub trait Exporter: Send + Sync {
    async fn export(&self, notes: Vec<Note>, options: &ExportOptions) -> Result<ExportResult>;
    fn format_name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
    fn supports_attachments(&self) -> bool;
}

pub struct ExporterRegistry {
    exporters: HashMap<String, Box<dyn Exporter>>,
}

impl ExporterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            exporters: HashMap::new(),
        };
        
        // 注册导出器
        registry.register("markdown", Box::new(MarkdownExporter::new()));
        registry.register("html", Box::new(HtmlExporter::new()));
        registry.register("pdf", Box::new(PdfExporter::new()));
        registry.register("epub", Box::new(EpubExporter::new()));
        registry.register("docx", Box::new(DocxExporter::new()));
        registry.register("json", Box::new(JsonExporter::new()));
        registry.register("obsidian", Box::new(ObsidianExporter::new()));
        
        registry
    }
}

// PDF 导出器示例
pub struct PdfExporter {
    chrome_path: Option<PathBuf>,
}

impl PdfExporter {
    pub fn new() -> Self {
        Self {
            chrome_path: Self::find_chrome_executable(),
        }
    }
    
    fn find_chrome_executable() -> Option<PathBuf> {
        // 在不同操作系统中查找 Chrome/Chromium
        let possible_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\\\Program Files\\\\Google\\\\Chrome\\\\Application\\\\chrome.exe",
                "C:\\\\Program Files (x86)\\\\Google\\\\Chrome\\\\Application\\\\chrome.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            ]
        } else {
            vec![
                "/usr/bin/google-chrome",
                "/usr/bin/chromium-browser",
                "/usr/bin/chromium",
            ]
        };
        
        possible_paths.into_iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
    }
}

#[async_trait]
impl Exporter for PdfExporter {
    async fn export(&self, notes: Vec<Note>, options: &ExportOptions) -> Result<ExportResult> {
        let chrome_path = self.chrome_path.as_ref()
            .ok_or_else(|| anyhow!("Chrome/Chromium not found"))?;
        
        let temp_dir = tempfile::tempdir()?;
        let mut exported_files = Vec::new();
        
        for note in notes {
            // 生成 HTML
            let html_content = self.generate_html_for_pdf(&note, options)?;
            let html_path = temp_dir.path().join(format!("{}.html", sanitize_filename(&note.title)));
            fs::write(&html_path, html_content).await?;
            
            // 使用 Chrome 转换为 PDF
            let pdf_path = options.output_dir.join(format!("{}.pdf", sanitize_filename(&note.title)));
            
            let output = Command::new(chrome_path)
                .args(&[
                    "--headless",
                    "--disable-gpu",
                    "--print-to-pdf-no-header",
                    "--print-to-pdf",
                    &pdf_path.to_string_lossy(),
                    &html_path.to_string_lossy(),
                ])
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(anyhow!("PDF generation failed: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
            
            exported_files.push(pdf_path);
        }
        
        Ok(ExportResult {
            success: true,
            exported_files,
            format: "PDF".to_string(),
            total_size: self.calculate_total_size(&exported_files).await?,
        })
    }
    
    fn format_name(&self) -> &'static str {
        "PDF"
    }
    
    fn file_extension(&self) -> &'static str {
        "pdf"
    }
    
    fn supports_attachments(&self) -> bool {
        true
    }
}
```

**验收标准**:
- [ ] 支持主流知识管理工具导入
- [ ] 导入准确率 > 95%
- [ ] 导出格式质量良好
- [ ] 大批量操作性能稳定
- [ ] 进度显示和错误处理完善

### 5.4 插件系统框架

**任务描述**: 建立可扩展的插件架构

**插件API设计**:
```rust
// plugin/api.rs
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> &str;
    
    // 生命周期方法
    fn on_load(&mut self, context: &PluginContext) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn on_enable(&mut self) -> Result<()>;
    fn on_disable(&mut self) -> Result<()>;
    
    // 功能扩展点
    fn content_processors(&self) -> Vec<Box<dyn ContentProcessor>>;
    fn command_handlers(&self) -> Vec<Box<dyn CommandHandler>>;
    fn ui_components(&self) -> Vec<UiComponent>;
    fn menu_items(&self) -> Vec<MenuItem>;
    fn settings_panels(&self) -> Vec<SettingsPanel>;
}

pub struct PluginContext {
    pub app_version: String,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub api: Arc<PluginApi>,
}

pub struct PluginApi {
    note_service: Arc<NoteService>,
    file_service: Arc<FileService>,
    search_service: Arc<SearchService>,
    publisher_registry: Arc<PublisherRegistry>,
    event_bus: Arc<EventBus>,
}

impl PluginApi {
    // 笔记操作 API
    pub async fn get_note(&self, id: &str) -> Result<Option<Note>> {
        self.note_service.get_note(id).await
    }
    
    pub async fn create_note(&self, note: Note) -> Result<String> {
        self.note_service.create_note(note).await
    }
    
    pub async fn update_note(&self, id: &str, note: Note) -> Result<()> {
        self.note_service.update_note(id, note).await
    }
    
    pub async fn delete_note(&self, id: &str) -> Result<()> {
        self.note_service.delete_note(id).await
    }
    
    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        self.search_service.search(query).await
    }
    
    // 事件系统 API
    pub fn subscribe<T: Event + 'static>(&self, handler: Box<dyn EventHandler<T>>) {
        self.event_bus.subscribe(handler);
    }
    
    pub fn emit<T: Event>(&self, event: T) {
        self.event_bus.emit(event);
    }
    
    // UI 操作 API
    pub fn show_notification(&self, message: &str, level: NotificationLevel) {
        self.emit(NotificationEvent {
            message: message.to_string(),
            level,
            duration: Duration::from_secs(5),
        });
    }
    
    pub fn open_modal(&self, component: UiComponent) {
        self.emit(ModalEvent::Open(component));
    }
    
    // 文件操作 API
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        self.file_service.read_file(path).await
    }
    
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        self.file_service.write_file(path, content).await
    }
    
    // 配置 API
    pub fn get_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        // 从配置中读取插件设置
        todo!()
    }
    
    pub fn set_setting<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        // 保存插件设置
        todo!()
    }
}
```

**插件管理器**:
```rust
// plugin/manager.rs
pub struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    plugin_configs: HashMap<String, PluginConfig>,
    plugin_api: Arc<PluginApi>,
    event_bus: Arc<EventBus>,
}

struct PluginInstance {
    plugin: Box<dyn Plugin>,
    metadata: PluginMetadata,
    status: PluginStatus,
    config: PluginConfig,
}

impl PluginManager {
    pub fn new(plugin_api: Arc<PluginApi>, event_bus: Arc<EventBus>) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_configs: HashMap::new(),
            plugin_api,
            event_bus,
        }
    }
    
    pub async fn load_plugin_from_path(&mut self, plugin_path: &Path) -> Result<String> {
        // 读取插件清单
        let manifest_path = plugin_path.join("plugin.toml");
        let manifest_content = fs::read_to_string(manifest_path).await?;
        let metadata: PluginMetadata = toml::from_str(&manifest_content)?;
        
        // 验证插件依赖
        self.validate_dependencies(&metadata)?;
        
        // 加载插件
        let plugin = if plugin_path.join("plugin.wasm").exists() {
            // WASM 插件
            self.load_wasm_plugin(plugin_path, &metadata).await?
        } else if plugin_path.join("plugin.js").exists() {
            // JavaScript 插件
            self.load_js_plugin(plugin_path, &metadata).await?
        } else {
            return Err(anyhow!("No valid plugin file found"));
        };
        
        let plugin_id = metadata.id.clone();
        
        // 创建插件上下文
        let context = PluginContext {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            config_dir: self.get_plugin_config_dir(&plugin_id),
            data_dir: self.get_plugin_data_dir(&plugin_id),
            api: self.plugin_api.clone(),
        };
        
        // 初始化插件
        let mut plugin_instance = PluginInstance {
            plugin,
            metadata: metadata.clone(),
            status: PluginStatus::Loaded,
            config: self.load_plugin_config(&plugin_id)?,
        };
        
        plugin_instance.plugin.on_load(&context)?;
        
        // 注册插件功能
        self.register_plugin_features(&plugin_instance)?;
        
        self.plugins.insert(plugin_id.clone(), plugin_instance);
        
        // 发出插件加载事件
        self.event_bus.emit(PluginEvent::Loaded {
            plugin_id: plugin_id.clone(),
            metadata,
        });
        
        Ok(plugin_id)
    }
    
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        if plugin.status == PluginStatus::Enabled {
            return Ok(());
        }
        
        plugin.plugin.on_enable()?;
        plugin.status = PluginStatus::Enabled;
        
        self.event_bus.emit(PluginEvent::Enabled {
            plugin_id: plugin_id.to_string(),
        });
        
        Ok(())
    }
    
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        if plugin.status == PluginStatus::Disabled {
            return Ok(());
        }
        
        plugin.plugin.on_disable()?;
        plugin.status = PluginStatus::Disabled;
        
        self.event_bus.emit(PluginEvent::Disabled {
            plugin_id: plugin_id.to_string(),
        });
        
        Ok(())
    }
    
    pub async fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let mut plugin = self.plugins.remove(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        // 清理插件注册的功能
        self.unregister_plugin_features(&plugin)?;
        
        // 调用插件卸载方法
        plugin.plugin.on_unload()?;
        
        self.event_bus.emit(PluginEvent::Unloaded {
            plugin_id: plugin_id.to_string(),
        });
        
        Ok(())
    }
    
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|instance| {
            PluginInfo {
                id: instance.metadata.id.clone(),
                name: instance.metadata.name.clone(),
                version: instance.metadata.version.clone(),
                description: instance.metadata.description.clone(),
                author: instance.metadata.author.clone(),
                status: instance.status.clone(),
                enabled: instance.status == PluginStatus::Enabled,
            }
        }).collect()
    }
    
    async fn load_wasm_plugin(&self, plugin_path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // 使用 wasmtime 加载 WASM 插件
        let wasm_path = plugin_path.join("plugin.wasm");
        let wasm_bytes = fs::read(wasm_path).await?;
        
        let plugin = WasmPlugin::new(wasm_bytes, metadata.clone())?;
        Ok(Box::new(plugin))
    }
    
    async fn load_js_plugin(&self, plugin_path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // 使用 deno_core 加载 JavaScript 插件
        let js_path = plugin_path.join("plugin.js");
        let js_content = fs::read_to_string(js_path).await?;
        
        let plugin = JavaScriptPlugin::new(js_content, metadata.clone())?;
        Ok(Box::new(plugin))
    }
}
```

**示例插件**:
```javascript
// plugins/word-count/plugin.js
class WordCountPlugin {
    constructor() {
        this.name = "Word Count";
        this.version = "1.0.0";
        this.description = "显示实时字数统计";
        this.author = "Zeno Team";
    }
    
    onLoad(context) {
        this.api = context.api;
        console.log("Word Count plugin loaded");
    }
    
    onEnable() {
        // 注册状态栏组件
        this.api.registerStatusBarItem({
            id: "word-count",
            text: "0 words",
            priority: 100,
            onClick: () => this.showDetailedStats(),
        });
        
        // 监听编辑器内容变化
        this.api.subscribe("editor.content-changed", (event) => {
            this.updateWordCount(event.content);
        });
        
        console.log("Word Count plugin enabled");
    }
    
    onDisable() {
        this.api.unregisterStatusBarItem("word-count");
        console.log("Word Count plugin disabled");
    }
    
    updateWordCount(content) {
        const wordCount = this.countWords(content);
        const charCount = content.length;
        
        this.api.updateStatusBarItem("word-count", {
            text: `${wordCount} words, ${charCount} chars`,
        });
    }
    
    countWords(text) {
        return text.trim() === "" ? 0 : text.trim().split(/\\s+/).length;
    }
    
    showDetailedStats() {
        this.api.getCurrentNote().then(note => {
            if (!note) return;
            
            const stats = this.calculateDetailedStats(note.content);
            
            this.api.openModal({
                title: "文档统计",
                content: this.renderStatsModal(stats),
                width: 400,
                height: 300,
            });
        });
    }
    
    calculateDetailedStats(content) {
        return {
            words: this.countWords(content),
            characters: content.length,
            charactersNoSpaces: content.replace(/\\s/g, "").length,
            paragraphs: content.split(/\\n\\s*\\n/).length,
            sentences: content.split(/[.!?]+/).length - 1,
            readingTime: Math.ceil(this.countWords(content) / 200), // 假设每分钟200词
        };
    }
    
    renderStatsModal(stats) {
        return `
            <div class="stats-modal">
                <div class="stat-item">
                    <span class="stat-label">字数:</span>
                    <span class="stat-value">${stats.words}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">字符数:</span>
                    <span class="stat-value">${stats.characters}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">字符数(不含空格):</span>
                    <span class="stat-value">${stats.charactersNoSpaces}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">段落数:</span>
                    <span class="stat-value">${stats.paragraphs}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">预计阅读时间:</span>
                    <span class="stat-value">${stats.readingTime} 分钟</span>
                </div>
            </div>
        `;
    }
}

// 导出插件实例
globalThis.plugin = new WordCountPlugin();
```

**插件市场设计**:
```typescript
// components/PluginMarket/index.tsx
export const PluginMarket: React.FC = () => {
    const [plugins, setPlugins] = useState<MarketPlugin[]>([]);
    const [searchQuery, setSearchQuery] = useState('');
    const [category, setCategory] = useState('all');
    const [loading, setLoading] = useState(true);
    
    useEffect(() => {
        loadMarketPlugins();
    }, []);
    
    const loadMarketPlugins = async () => {
        try {
            const response = await fetch('https://zeno-plugins.com/api/plugins');
            const marketPlugins = await response.json();
            setPlugins(marketPlugins);
        } catch (error) {
            console.error('Failed to load market plugins:', error);
        } finally {
            setLoading(false);
        }
    };
    
    const installPlugin = async (plugin: MarketPlugin) => {
        try {
            await invoke('install_plugin', { 
                pluginId: plugin.id,
                downloadUrl: plugin.downloadUrl 
            });
            
            // 更新本地插件列表
            await invoke('refresh_plugins');
            
            toast.success(`插件 ${plugin.name} 安装成功`);
        } catch (error) {
            toast.error(`安装失败: ${error}`);
        }
    };
    
    const filteredPlugins = plugins.filter(plugin => {
        const matchesSearch = plugin.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                            plugin.description.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesCategory = category === 'all' || plugin.category === category;
        
        return matchesSearch && matchesCategory;
    });
    
    return (
        <div className="plugin-market">
            <div className="market-header">
                <h2>插件市场</h2>
                <div className="market-filters">
                    <input
                        type="text"
                        placeholder="搜索插件..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                    />
                    <select value={category} onChange={(e) => setCategory(e.target.value)}>
                        <option value="all">所有分类</option>
                        <option value="editor">编辑器</option>
                        <option value="export">导出</option>
                        <option value="visualization">可视化</option>
                        <option value="productivity">效率工具</option>
                    </select>
                </div>
            </div>
            
            <div className="plugins-grid">
                {filteredPlugins.map(plugin => (
                    <PluginCard
                        key={plugin.id}
                        plugin={plugin}
                        onInstall={() => installPlugin(plugin)}
                    />
                ))}
            </div>
        </div>
    );
};
```

**验收标准**:
- [ ] 插件加载和卸载稳定
- [ ] API接口功能完整
- [ ] 插件沙箱安全隔离
- [ ] 插件市场集成正常
- [ ] 开发工具和文档完善

## 里程碑和验收

### 第1周里程碑
- Zola集成和静态网站生成
- 微信公众号发布适配器
- 基础导入导出框架

### 第2-3周里程碑
- 多平台发布适配器完成
- Obsidian/Notion导入器实现
- 插件系统核心架构

### 第4-5周里程碑
- 插件API和管理器完成
- 示例插件开发
- 插件市场集成

### 最终验收标准
- [ ] 至少支持3个发布平台
- [ ] 至少支持3种格式导入
- [ ] 插件系统稳定运行
- [ ] 性能指标达到要求
- [ ] 用户文档完整

## 下一步规划

### 产品完善
- 用户反馈收集和功能优化
- 性能监控和持续改进
- 社区建设和生态发展

### 技术演进
- AI功能集成探索
- 云同步和协作功能
- 移动端应用开发

---

**创建时间**: 2025-07-01  
**负责人**: 全栈开发团队  
**状态**: 规划中  
**依赖**: Phase 4 知识网络层完成