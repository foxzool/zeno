use anyhow::Result;
use crate::models::Note;

/// 发布器接口
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    /// 发布笔记
    async fn publish(&self, note: &Note) -> Result<PublishResult>;
    
    /// 取消发布
    async fn unpublish(&self, note: &Note) -> Result<()>;
    
    /// 检查发布状态
    async fn check_status(&self, note: &Note) -> Result<PublishStatus>;
}

/// 发布结果
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// 发布是否成功
    pub success: bool,
    /// 发布URL
    pub url: Option<String>,
    /// 发布消息
    pub message: String,
}

/// 发布状态
#[derive(Debug, Clone)]
pub enum PublishStatus {
    /// 未发布
    NotPublished,
    /// 已发布
    Published {
        url: String,
        published_at: chrono::DateTime<chrono::Utc>,
    },
    /// 发布失败
    Failed {
        error: String,
    },
}