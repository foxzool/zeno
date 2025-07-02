use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 微信公众号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatConfig {
    /// 公众号名称
    pub account_name: String,
    /// AppID
    pub app_id: String,
    /// AppSecret (加密存储)
    pub app_secret: String,
    /// 访问令牌 (临时)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// 令牌过期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<DateTime<Utc>>,
    /// 是否启用
    pub enabled: bool,
    /// 默认发布设置
    pub default_settings: WeChatPublishSettings,
}

/// 微信公众号发布设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatPublishSettings {
    /// 是否立即发布
    pub publish_immediately: bool,
    /// 是否打开评论
    pub open_comment: bool,
    /// 是否仅粉丝可评论
    pub only_fans_can_comment: bool,
    /// 封面图片地址
    pub thumb_media_id: Option<String>,
    /// 作者
    pub author: String,
    /// 原文链接
    pub content_source_url: Option<String>,
    /// 摘要
    pub digest: Option<String>,
    /// 是否显示封面
    pub show_cover_pic: bool,
    /// 自定义字段
    pub extra_fields: HashMap<String, String>,
}

impl Default for WeChatConfig {
    fn default() -> Self {
        Self {
            account_name: String::new(),
            app_id: String::new(),
            app_secret: String::new(),
            access_token: None,
            token_expires_at: None,
            enabled: false,
            default_settings: WeChatPublishSettings::default(),
        }
    }
}

impl Default for WeChatPublishSettings {
    fn default() -> Self {
        Self {
            publish_immediately: false,
            open_comment: true,
            only_fans_can_comment: false,
            thumb_media_id: None,
            author: String::new(),
            content_source_url: None,
            digest: None,
            show_cover_pic: true,
            extra_fields: HashMap::new(),
        }
    }
}

/// 微信公众号素材类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    /// 图片
    Image,
    /// 视频
    Video,
    /// 语音
    Voice,
    /// 缩略图
    Thumb,
}

/// 上传的素材信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// 素材类型
    pub media_type: MediaType,
    /// 媒体文件 ID
    pub media_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 本地文件路径
    pub local_path: String,
    /// 文件大小
    pub file_size: u64,
    /// 文件名
    pub filename: String,
}

/// 图文消息素材
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    /// 标题
    pub title: String,
    /// 作者
    pub author: String,
    /// 图文消息的摘要
    pub digest: String,
    /// 图文消息的具体内容，支持HTML标签
    pub content: String,
    /// 图文消息的原文地址
    pub content_source_url: Option<String>,
    /// 缩略图的media_id
    pub thumb_media_id: String,
    /// 是否显示封面，0为false，1为true
    pub show_cover_pic: bool,
    /// 是否打开评论，0不打开，1打开
    pub need_open_comment: bool,
    /// 是否粉丝才可评论，0所有人可评论，1粉丝才可评论
    pub only_fans_can_comment: bool,
}

/// 微信发布结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatPublishResult {
    /// 是否成功
    pub success: bool,
    /// 媒体文件 ID
    pub media_id: Option<String>,
    /// 发布时间
    pub published_at: DateTime<Utc>,
    /// 文章 URL (预览链接)
    pub preview_url: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 发布的笔记标题
    pub note_title: String,
    /// 处理时间 (毫秒)
    pub processing_time_ms: u64,
}

/// 微信 API 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatApiError {
    /// 错误码
    pub errcode: i32,
    /// 错误信息
    pub errmsg: String,
}

/// 获取访问令牌的响应
#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

/// 上传媒体文件的响应
#[derive(Debug, Deserialize)]
pub struct UploadMediaResponse {
    pub media_id: Option<String>,
    pub created_at: Option<i64>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

/// 新增永久图文素材的响应
#[derive(Debug, Deserialize)]
pub struct AddNewsResponse {
    pub media_id: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

/// 微信公众号统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatStats {
    /// 总发布文章数
    pub total_articles: u32,
    /// 本月发布数
    pub monthly_articles: u32,
    /// 今日发布数
    pub daily_articles: u32,
    /// 总媒体文件数
    pub total_media_files: u32,
    /// 媒体文件总大小 (字节)
    pub total_media_size: u64,
    /// 平均发布时间 (毫秒)
    pub average_publish_time: u64,
    /// 最后发布时间
    pub last_publish_time: Option<DateTime<Utc>>,
    /// 令牌状态
    pub token_status: TokenStatus,
}

/// 令牌状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenStatus {
    /// 有效
    Valid,
    /// 即将过期 (30分钟内)
    Expiring,
    /// 已过期
    Expired,
    /// 未配置
    NotConfigured,
    /// 配置错误
    ConfigError,
}

impl Default for WeChatStats {
    fn default() -> Self {
        Self {
            total_articles: 0,
            monthly_articles: 0,
            daily_articles: 0,
            total_media_files: 0,
            total_media_size: 0,
            average_publish_time: 0,
            last_publish_time: None,
            token_status: TokenStatus::NotConfigured,
        }
    }
}

/// 微信内容转换选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatContentOptions {
    /// 是否转换 Markdown 到 HTML
    pub convert_markdown: bool,
    /// 是否处理图片
    pub process_images: bool,
    /// 是否上传本地图片到微信
    pub upload_local_images: bool,
    /// 是否转换代码块
    pub convert_code_blocks: bool,
    /// 是否保留原始格式
    pub preserve_formatting: bool,
    /// 最大图片宽度
    pub max_image_width: u32,
    /// 图片质量 (1-100)
    pub image_quality: u8,
    /// 是否添加水印
    pub add_watermark: bool,
    /// 水印文本
    pub watermark_text: String,
}

impl Default for WeChatContentOptions {
    fn default() -> Self {
        Self {
            convert_markdown: true,
            process_images: true,
            upload_local_images: true,
            convert_code_blocks: true,
            preserve_formatting: false,
            max_image_width: 1080,
            image_quality: 80,
            add_watermark: false,
            watermark_text: String::new(),
        }
    }
}