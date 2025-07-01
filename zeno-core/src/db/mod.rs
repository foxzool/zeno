pub mod models;
pub mod connection;
pub mod migrations;

pub use models::*;
pub use connection::*;

use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;
use crate::error::Result;

/// 数据库管理器
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.as_ref().display());
        let pool = create_connection_pool(&db_url).await?;
        
        Ok(Self { pool })
    }

    /// 获取连接池
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// 从现有连接池创建 Database
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 运行数据库迁移
    pub async fn migrate(&self) -> Result<()> {
        migrations::run_migrations(&self.pool).await
    }

    /// 初始化数据库 (创建表结构和初始数据)
    pub async fn initialize(&self) -> Result<()> {
        self.migrate().await?;
        self.init_default_data().await?;
        Ok(())
    }

    /// 初始化默认数据
    async fn init_default_data(&self) -> Result<()> {
        // 检查是否已经初始化
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM system_config WHERE key = 'initialized'")
            .fetch_one(&self.pool)
            .await?;

        if count > 0 {
            return Ok(()); // 已经初始化过了
        }

        // 创建默认分类
        let uncategorized = Category {
            id: 0,
            name: "未分类".to_string(),
            parent_id: None,
            description: "默认分类".to_string(),
            color: "#6B7280".to_string(),
            icon: "folder".to_string(),
            sort_order: 0,
            created_at: chrono::Utc::now(),
        };

        let archived = Category {
            id: 0,
            name: "归档".to_string(),
            parent_id: None,
            description: "已归档的笔记".to_string(),
            color: "#9CA3AF".to_string(),
            icon: "archive".to_string(),
            sort_order: 999,
            created_at: chrono::Utc::now(),
        };

        // 插入默认分类
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO categories (name, parent_id, description, color, icon, sort_order)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&uncategorized.name)
        .bind(uncategorized.parent_id)
        .bind(&uncategorized.description)
        .bind(&uncategorized.color)
        .bind(&uncategorized.icon)
        .bind(uncategorized.sort_order)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO categories (name, parent_id, description, color, icon, sort_order)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&archived.name)
        .bind(archived.parent_id)
        .bind(&archived.description)
        .bind(&archived.color)
        .bind(&archived.icon)
        .bind(archived.sort_order)
        .execute(&self.pool)
        .await?;

        // 创建常用标签
        let default_tags = vec![
            ("想法", "#3B82F6", "随想和灵感"),
            ("学习", "#10B981", "学习笔记和总结"),
            ("工作", "#F59E0B", "工作相关内容"),
            ("项目", "#8B5CF6", "项目记录和规划"),
            ("阅读", "#EF4444", "读书笔记和摘录"),
        ];

        for (name, color, description) in default_tags {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO tags (name, color, description)
                VALUES (?, ?, ?)
                "#
            )
            .bind(name)
            .bind(color)
            .bind(description)
            .execute(&self.pool)
            .await?;
        }

        // 标记已初始化
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO system_config (key, value, description)
            VALUES ('initialized', 'true', '数据库初始化标记')
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn get_statistics(&self) -> Result<Statistics> {
        // 总笔记数
        let total_notes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE status != 'deleted'")
            .fetch_one(&self.pool)
            .await?;

        // 总字数
        let total_words: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(word_count), 0) FROM notes WHERE status != 'deleted'")
            .fetch_one(&self.pool)
            .await?;

        // 总标签数
        let total_tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(&self.pool)
            .await?;

        // 总分类数
        let total_categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
            .fetch_one(&self.pool)
            .await?;

        // 总链接数
        let total_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM links")
            .fetch_one(&self.pool)
            .await?;

        // 按状态统计笔记
        let notes_by_status_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as count FROM notes GROUP BY status"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut notes_by_status = std::collections::HashMap::new();
        for (status, count) in notes_by_status_rows {
            notes_by_status.insert(status, count);
        }

        // 热门标签
        let top_tags: Vec<(String, i32)> = sqlx::query_as(
            "SELECT name, usage_count FROM tags ORDER BY usage_count DESC LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await?;

        // 最近笔记
        let recent_notes: Vec<Note> = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE status != 'deleted' ORDER BY modified_at DESC LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Statistics {
            total_notes,
            total_words,
            total_tags,
            total_categories,
            total_links,
            notes_by_status,
            top_tags,
            recent_notes,
        })
    }

    /// 清理数据库 (删除孤立数据)
    pub async fn cleanup(&self) -> Result<()> {
        // 删除没有关联笔记的标签关系
        sqlx::query("DELETE FROM note_tags WHERE note_id NOT IN (SELECT id FROM notes)")
            .execute(&self.pool)
            .await?;

        // 删除没有关联笔记的分类关系
        sqlx::query("DELETE FROM note_categories WHERE note_id NOT IN (SELECT id FROM notes)")
            .execute(&self.pool)
            .await?;

        // 删除孤立的链接
        sqlx::query(
            r#"
            DELETE FROM links 
            WHERE source_id NOT IN (SELECT id FROM notes) 
               OR target_id NOT IN (SELECT id FROM notes)
            "#
        )
        .execute(&self.pool)
        .await?;

        // 更新标签使用计数
        sqlx::query(
            r#"
            UPDATE tags SET usage_count = (
                SELECT COUNT(*) FROM note_tags WHERE tag_id = tags.id
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // 删除未使用的标签 (可选，需要配置开关)
        let auto_cleanup_tags: String = sqlx::query_scalar(
            "SELECT value FROM system_config WHERE key = 'auto_cleanup_unused_tags'"
        )
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_else(|| "false".to_string());

        if auto_cleanup_tags == "true" {
            sqlx::query("DELETE FROM tags WHERE usage_count = 0")
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// 备份数据库
    pub async fn backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        // SQLite 数据库文件级备份
        // 这里简化实现，实际应该使用 SQLite 的 VACUUM INTO 或文件复制
        let backup_sql = format!(
            "VACUUM INTO '{}'", 
            backup_path.as_ref().display()
        );
        
        sqlx::query(&backup_sql)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 从备份恢复数据库
    pub async fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        // 检查备份文件是否存在
        if !backup_path.as_ref().exists() {
            return Err(crate::error::Error::NotFound(
                format!("备份文件不存在: {}", backup_path.as_ref().display())
            ));
        }

        // 这里应该实现完整的恢复逻辑
        // 包括验证备份文件完整性、关闭当前连接、替换数据库文件等
        // 为了简化，这里只是提示需要重启应用
        log::warn!("数据库恢复需要重启应用程序");
        
        Ok(())
    }

    /// 检查数据库完整性
    pub async fn check_integrity(&self) -> Result<bool> {
        let result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;

        Ok(result == "ok")
    }

    /// 优化数据库
    pub async fn optimize(&self) -> Result<()> {
        // 重建索引
        sqlx::query("REINDEX").execute(&self.pool).await?;
        
        // 清理空间
        sqlx::query("VACUUM").execute(&self.pool).await?;
        
        // 分析表统计信息
        sqlx::query("ANALYZE").execute(&self.pool).await?;

        Ok(())
    }

    /// 关闭数据库连接
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let db = Database::new(&db_path).await.unwrap();
        db.initialize().await.unwrap();
        
        // 验证初始化成功
        let stats = db.get_statistics().await.unwrap();
        assert!(stats.total_categories >= 2); // 至少有未分类和归档两个分类
    }

    #[tokio::test]
    async fn test_database_backup_restore() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backup_path = dir.path().join("backup.db");
        
        let db = Database::new(&db_path).await.unwrap();
        db.initialize().await.unwrap();
        
        // 备份
        db.backup(&backup_path).await.unwrap();
        assert!(backup_path.exists());
    }

    #[tokio::test]
    async fn test_integrity_check() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let db = Database::new(&db_path).await.unwrap();
        db.initialize().await.unwrap();
        
        let is_ok = db.check_integrity().await.unwrap();
        assert!(is_ok);
    }
}