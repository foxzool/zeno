use crate::models::wechat::*;
use crate::models::note::Note;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::time::Instant;
use tokio::fs;
use regex::Regex;
use chrono::{DateTime, Utc};
use pulldown_cmark::{Parser, html, Options, Event, Tag, CowStr, TagEnd};
use std::collections::HashMap;

/// 微信公众号发布器
pub struct WeChatPublisher {
    config: WeChatConfig,
    client: reqwest::Client,
}

impl WeChatPublisher {
    /// 创建新的微信发布器实例
    pub fn new(config: WeChatConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// 发布笔记到微信公众号
    pub async fn publish_note(
        &mut self,
        note: &Note,
        settings: Option<WeChatPublishSettings>,
    ) -> Result<WeChatPublishResult> {
        let start_time = Instant::now();

        // 使用提供的设置或默认设置
        let publish_settings = settings.unwrap_or_else(|| self.config.default_settings.clone());

        // 确保访问令牌有效
        self.ensure_valid_token().await?;

        // 转换笔记内容为微信格式
        let converted_content = self.convert_note_to_wechat_format(note).await?;

        // 处理图片上传
        let processed_content = self.process_images_in_content(&converted_content, note).await?;

        // 创建图文消息
        let news_item = self.create_news_item(note, &processed_content, &publish_settings).await?;

        // 上传图文素材
        let result = self.upload_news_material(&news_item).await?;

        let publish_result = WeChatPublishResult {
            success: result.media_id.is_some(),
            media_id: result.media_id,
            published_at: Utc::now(),
            preview_url: None, // 微信不提供直接预览链接
            error_message: result.errmsg,
            note_title: note.title.clone(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        if publish_result.success {
            log::info!("Successfully published note '{}' to WeChat", note.title);
        } else {
            log::error!(
                "Failed to publish note '{}' to WeChat: {}",
                note.title,
                publish_result.error_message.as_deref().unwrap_or("Unknown error")
            );
        }

        Ok(publish_result)
    }

    /// 批量发布笔记
    pub async fn publish_notes(
        &mut self,
        notes: Vec<&Note>,
        settings: Option<WeChatPublishSettings>,
    ) -> Result<Vec<WeChatPublishResult>> {
        let mut results = Vec::new();

        for note in notes {
            match self.publish_note(note, settings.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    let error_result = WeChatPublishResult {
                        success: false,
                        media_id: None,
                        published_at: Utc::now(),
                        preview_url: None,
                        error_message: Some(e.to_string()),
                        note_title: note.title.clone(),
                        processing_time_ms: 0,
                    };
                    results.push(error_result);
                }
            }

            // 避免 API 频率限制，添加延迟
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(results)
    }

    /// 确保访问令牌有效
    async fn ensure_valid_token(&mut self) -> Result<()> {
        // 检查当前令牌是否有效
        if let Some(expires_at) = self.config.token_expires_at {
            if Utc::now() < expires_at.checked_sub_signed(chrono::Duration::minutes(5)).unwrap_or(expires_at) {
                // 令牌仍然有效（提前5分钟刷新）
                return Ok(());
            }
        }

        // 获取新的访问令牌
        self.refresh_access_token().await
    }

    /// 刷新访问令牌
    async fn refresh_access_token(&mut self) -> Result<()> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.config.app_id,
            self.config.app_secret
        );

        let response = self.client.get(&url).send().await?;
        let token_response: AccessTokenResponse = response.json().await?;

        if let Some(access_token) = token_response.access_token {
            self.config.access_token = Some(access_token);
            
            if let Some(expires_in) = token_response.expires_in {
                self.config.token_expires_at = Some(
                    Utc::now() + chrono::Duration::seconds(expires_in)
                );
            }
            
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to get access token: {} ({})",
                token_response.errmsg.unwrap_or_else(|| "Unknown error".to_string()),
                token_response.errcode.unwrap_or(0)
            ))
        }
    }

    /// 将笔记转换为微信公众号格式
    async fn convert_note_to_wechat_format(&self, note: &Note) -> Result<String> {
        let mut content = note.content.clone();

        // 转换 Wiki 链接为普通文本
        content = self.convert_wiki_links(&content);

        // 转换 Markdown 为 HTML
        content = self.convert_markdown_to_html(&content);

        // 处理代码块
        content = self.process_code_blocks(&content);

        // 处理数学公式
        content = self.process_math_formulas(&content);

        // 添加样式和格式
        content = self.apply_wechat_styling(&content);

        Ok(content)
    }

    /// 转换 Wiki 链接为普通文本
    fn convert_wiki_links(&self, content: &str) -> String {
        let wiki_link_regex = Regex::new(r"\[\[([^\]]+?)\]\]").unwrap();
        
        wiki_link_regex.replace_all(content, |caps: &regex::Captures| {
            let link_text = &caps[1];
            
            // 如果包含别名，使用别名
            if let Some(pipe_pos) = link_text.find('|') {
                let alias = &link_text[pipe_pos + 1..].trim();
                format!("**{}**", alias)
            } else {
                format!("**{}**", link_text)
            }
        }).to_string()
    }

    /// 转换 Markdown 为 HTML
    fn convert_markdown_to_html(&self, content: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(content, options);
        
        // 自定义事件处理，适配微信格式
        let events: Vec<Event> = parser.map(|event| {
            match event {
                Event::Start(Tag::CodeBlock(_)) => {
                    Event::Html(CowStr::Borrowed("<section style=\"background: #f7f7f7; padding: 10px; border-left: 4px solid #42b983; margin: 10px 0; border-radius: 4px; font-family: 'SF Mono', Monaco, monospace;\">"))
                }
                Event::End(TagEnd::CodeBlock) => {
                    Event::Html(CowStr::Borrowed("</section>"))
                }
                Event::Start(Tag::BlockQuote) => {
                    Event::Html(CowStr::Borrowed("<section style=\"background: #f9f9f9; padding: 15px; border-left: 4px solid #ddd; margin: 15px 0; color: #666; font-style: italic;\">"))
                }
                Event::End(TagEnd::BlockQuote) => {
                    Event::Html(CowStr::Borrowed("</section>"))
                }
                _ => event,
            }
        }).collect();

        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        html_output
    }

    /// 处理代码块样式
    fn process_code_blocks(&self, content: &str) -> String {
        let code_regex = Regex::new(r"<pre><code[^>]*>(.*?)</code></pre>").unwrap();
        
        code_regex.replace_all(content, |caps: &regex::Captures| {
            let code_content = &caps[1];
            format!(
                r#"<section style="background: #2d3748; color: #e2e8f0; padding: 15px; border-radius: 8px; margin: 15px 0; font-family: 'SF Mono', Monaco, monospace; font-size: 14px; line-height: 1.5; overflow-x: auto;">{}</section>"#,
                code_content
            )
        }).to_string()
    }

    /// 处理数学公式
    fn process_math_formulas(&self, content: &str) -> String {
        // 将数学公式转换为图片或移除
        let math_regex = Regex::new(r"\$\$([^$]+)\$\$|\$([^$]+)\$").unwrap();
        
        math_regex.replace_all(content, |caps: &regex::Captures| {
            let formula = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("");
            format!("<em>数学公式: {}</em>", formula)
        }).to_string()
    }

    /// 应用微信样式
    fn apply_wechat_styling(&self, content: &str) -> String {
        let mut styled_content = content.to_string();

        // 为段落添加样式
        styled_content = styled_content.replace(
            "<p>",
            r#"<p style="margin: 10px 0; line-height: 1.8; color: #333;">"#
        );

        // 为标题添加样式
        for i in 1..=6 {
            let from = format!("<h{}>", i);
            let size = match i {
                1 => "24px",
                2 => "20px",
                3 => "18px",
                4 => "16px",
                _ => "14px",
            };
            let to = format!(
                r#"<h{} style="color: #2c3e50; font-size: {}; margin: 20px 0 10px 0; font-weight: bold; border-bottom: 2px solid #42b983; padding-bottom: 8px;">"#,
                i, size
            );
            styled_content = styled_content.replace(&from, &to);
        }

        // 为链接添加样式
        styled_content = styled_content.replace(
            "<a ",
            r#"<a style="color: #42b983; text-decoration: none;" "#
        );

        // 为强调文本添加样式
        styled_content = styled_content.replace(
            "<strong>",
            r#"<strong style="color: #e74c3c; font-weight: bold;">"#
        );

        styled_content = styled_content.replace(
            "<em>",
            r#"<em style="color: #8e44ad; font-style: italic;">"#
        );

        styled_content
    }

    /// 处理内容中的图片
    async fn process_images_in_content(&self, content: &str, note: &Note) -> Result<String> {
        let img_regex = Regex::new(r#"<img[^>]+src="([^"]+)"[^>]*>"#).unwrap();
        let mut processed_content = content.to_string();

        for captures in img_regex.captures_iter(content) {
            let img_src = &captures[1];
            
            // 如果是本地图片，上传到微信
            if !img_src.starts_with("http") {
                if let Ok(media_info) = self.upload_local_image(img_src, note).await {
                    // 替换为微信图片链接
                    let wechat_img = format!(
                        r#"<img src="https://mmbiz.qpic.cn/mmbiz_png/{}/0?wx_fmt=png" style="max-width: 100%; height: auto; display: block; margin: 15px auto;">"#,
                        media_info.media_id
                    );
                    processed_content = processed_content.replace(&captures[0], &wechat_img);
                }
            } else {
                // 为外部图片添加样式
                let styled_img = format!(
                    r#"<img src="{}" style="max-width: 100%; height: auto; display: block; margin: 15px auto;">"#,
                    img_src
                );
                processed_content = processed_content.replace(&captures[0], &styled_img);
            }
        }

        Ok(processed_content)
    }

    /// 上传本地图片到微信
    async fn upload_local_image(&self, img_path: &str, note: &Note) -> Result<MediaInfo> {
        let full_path = if Path::new(img_path).is_absolute() {
            img_path.to_string()
        } else {
            note.path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(img_path)
                .to_string_lossy()
                .to_string()
        };

        let file_data = fs::read(&full_path).await?;
        let filename = Path::new(&full_path)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("image.png"))
            .to_string_lossy()
            .to_string();

        self.upload_media(&file_data, &filename, MediaType::Image).await
    }

    /// 上传媒体文件到微信
    pub async fn upload_media(&self, file_data: &[u8], filename: &str, media_type: MediaType) -> Result<MediaInfo> {
        let access_token = self.config.access_token.as_ref()
            .ok_or_else(|| anyhow!("Access token not available"))?;

        let type_str = match media_type {
            MediaType::Image => "image",
            MediaType::Video => "video",
            MediaType::Voice => "voice",
            MediaType::Thumb => "thumb",
        };

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/media/upload?access_token={}&type={}",
            access_token, type_str
        );

        let form = reqwest::multipart::Form::new()
            .part("media", reqwest::multipart::Part::bytes(file_data.to_vec())
                .file_name(filename.to_string()));

        let response = self.client.post(&url)
            .multipart(form)
            .send()
            .await?;

        let upload_response: UploadMediaResponse = response.json().await?;

        if let Some(media_id) = upload_response.media_id {
            Ok(MediaInfo {
                media_type,
                media_id,
                created_at: Utc::now(),
                expires_at: Utc::now() + chrono::Duration::days(3), // 临时素材3天过期
                local_path: filename.to_string(),
                file_size: file_data.len() as u64,
                filename: filename.to_string(),
            })
        } else {
            Err(anyhow!(
                "Failed to upload media: {} ({})",
                upload_response.errmsg.unwrap_or_else(|| "Unknown error".to_string()),
                upload_response.errcode.unwrap_or(0)
            ))
        }
    }

    /// 创建图文消息
    async fn create_news_item(
        &self,
        note: &Note,
        content: &str,
        settings: &WeChatPublishSettings,
    ) -> Result<NewsItem> {
        // 生成摘要
        let digest = settings.digest.clone().unwrap_or_else(|| {
            self.generate_digest(&note.content)
        });

        // 获取或上传封面图片
        let thumb_media_id = if let Some(ref thumb_id) = settings.thumb_media_id {
            thumb_id.clone()
        } else {
            // 使用默认封面或从内容中提取第一张图片
            self.get_default_thumb_media_id().await?
        };

        Ok(NewsItem {
            title: note.title.clone(),
            author: settings.author.clone(),
            digest,
            content: content.to_string(),
            content_source_url: settings.content_source_url.clone(),
            thumb_media_id,
            show_cover_pic: settings.show_cover_pic,
            need_open_comment: settings.open_comment,
            only_fans_can_comment: settings.only_fans_can_comment,
        })
    }

    /// 生成文章摘要
    fn generate_digest(&self, content: &str) -> String {
        // 移除 Markdown 格式，提取纯文本
        let plain_text = self.strip_markdown(content);
        
        // 取前120个字符作为摘要
        if plain_text.len() <= 120 {
            plain_text
        } else {
            format!("{}...", &plain_text[..120])
        }
    }

    /// 移除 Markdown 格式
    fn strip_markdown(&self, content: &str) -> String {
        let parser = Parser::new(content);
        let mut plain_text = String::new();

        for event in parser {
            match event {
                Event::Text(text) => plain_text.push_str(&text),
                Event::Code(code) => plain_text.push_str(&code),
                Event::SoftBreak | Event::HardBreak => plain_text.push(' '),
                _ => {}
            }
        }

        // 清理多余的空白字符
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        whitespace_regex.replace_all(&plain_text, " ").trim().to_string()
    }

    /// 获取默认封面图片 media_id
    async fn get_default_thumb_media_id(&self) -> Result<String> {
        // 这里可以上传一个默认的封面图片
        // 为了简化，返回一个占位符，实际使用时需要上传真实图片
        Ok("default_thumb_media_id".to_string())
    }

    /// 上传图文素材
    async fn upload_news_material(&self, news_item: &NewsItem) -> Result<AddNewsResponse> {
        let access_token = self.config.access_token.as_ref()
            .ok_or_else(|| anyhow!("Access token not available"))?;

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/material/add_news?access_token={}",
            access_token
        );

        let articles = vec![news_item];
        let payload = serde_json::json!({
            "articles": articles
        });

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;

        let add_news_response: AddNewsResponse = response.json().await?;
        Ok(add_news_response)
    }

    /// 获取微信公众号统计信息
    pub async fn get_stats(&self) -> Result<WeChatStats> {
        // 这里可以实现统计逻辑，从本地数据库或微信 API 获取统计信息
        // 为了简化，返回默认值
        Ok(WeChatStats {
            total_articles: 0,
            monthly_articles: 0,
            daily_articles: 0,
            total_media_files: 0,
            total_media_size: 0,
            average_publish_time: 0,
            last_publish_time: None,
            token_status: self.get_token_status(),
        })
    }

    /// 获取令牌状态
    fn get_token_status(&self) -> TokenStatus {
        if self.config.app_id.is_empty() || self.config.app_secret.is_empty() {
            return TokenStatus::NotConfigured;
        }

        if let Some(expires_at) = self.config.token_expires_at {
            let now = Utc::now();
            if now > expires_at {
                TokenStatus::Expired
            } else if now > expires_at.checked_sub_signed(chrono::Duration::minutes(30)).unwrap_or(expires_at) {
                TokenStatus::Expiring
            } else {
                TokenStatus::Valid
            }
        } else {
            TokenStatus::Expired
        }
    }

    /// 测试微信配置
    pub async fn test_configuration(&mut self) -> Result<bool> {
        match self.refresh_access_token().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 预览转换后的内容
    pub async fn preview_converted_content(&self, note: &Note) -> Result<String> {
        self.convert_note_to_wechat_format(note).await
    }
}