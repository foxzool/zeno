use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions, ConnectOptions};
use std::str::FromStr;
use std::time::Duration;
use crate::error::Result;

/// 创建 SQLite 连接池
pub async fn create_connection_pool(database_url: &str) -> Result<SqlitePool> {
    // 解析连接选项
    let mut options = SqliteConnectOptions::from_str(database_url)?;
    
    // 配置连接选项
    options = options
        .create_if_missing(true)                    // 如果数据库文件不存在则创建
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)  // 使用 WAL 模式提高并发性能
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal) // 平衡性能和安全性
        .foreign_keys(true)                         // 启用外键约束
        .busy_timeout(Duration::from_secs(30))      // 设置忙等超时
        .pragma("cache_size", "1000")               // 设置缓存大小 (页数)
        .pragma("temp_store", "memory")             // 临时表存储在内存中
        .pragma("mmap_size", "268435456")           // 设置内存映射大小 (256MB)
        .pragma("optimize", "0x10002")              // 启用查询优化
        .disable_statement_logging();               // 禁用语句日志以提高性能

    // 创建连接池
    let pool = SqlitePool::connect_with(options).await?;
    
    // 设置连接池配置
    sqlx::query("PRAGMA optimize").execute(&pool).await?;
    
    Ok(pool)
}

/// 测试数据库连接
pub async fn test_connection(pool: &SqlitePool) -> Result<()> {
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    Ok(())
}

/// 获取数据库信息
pub async fn get_database_info(pool: &SqlitePool) -> Result<DatabaseInfo> {
    // 数据库版本
    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(pool)
        .await?;

    // 页面大小
    let page_size: i32 = sqlx::query_scalar("PRAGMA page_size")
        .fetch_one(pool)
        .await?;

    // 页面数量
    let page_count: i32 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(pool)
        .await?;

    // WAL 模式检查
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;

    // 外键检查
    let foreign_keys: bool = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;

    // 缓存大小
    let cache_size: i32 = sqlx::query_scalar("PRAGMA cache_size")
        .fetch_one(pool)
        .await?;

    Ok(DatabaseInfo {
        version,
        page_size,
        page_count,
        database_size: (page_size * page_count) as u64,
        journal_mode,
        foreign_keys_enabled: foreign_keys,
        cache_size,
    })
}

/// 数据库信息结构
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub version: String,
    pub page_size: i32,
    pub page_count: i32,
    pub database_size: u64,
    pub journal_mode: String,
    pub foreign_keys_enabled: bool,
    pub cache_size: i32,
}

impl DatabaseInfo {
    /// 格式化数据库大小
    pub fn formatted_size(&self) -> String {
        format_bytes(self.database_size)
    }

    /// 检查配置是否最优
    pub fn is_optimized(&self) -> bool {
        self.journal_mode.to_lowercase() == "wal" && 
        self.foreign_keys_enabled &&
        self.cache_size > 0
    }
}

/// 格式化字节大小
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 执行数据库性能分析
pub async fn analyze_performance(pool: &SqlitePool) -> Result<PerformanceMetrics> {
    // 获取查询计划缓存信息
    let stmt_cache_hit: Option<i32> = sqlx::query_scalar("SELECT stmt_cache_hit FROM pragma_stats")
        .fetch_optional(pool)
        .await?;

    let stmt_cache_miss: Option<i32> = sqlx::query_scalar("SELECT stmt_cache_miss FROM pragma_stats")
        .fetch_optional(pool)
        .await?;

    // 计算缓存命中率
    let cache_hit_rate = if let (Some(hit), Some(miss)) = (stmt_cache_hit, stmt_cache_miss) {
        if hit + miss > 0 {
            (hit as f64 / (hit + miss) as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    // 获取页面统计
    let page_cache_hit: i32 = sqlx::query_scalar("PRAGMA cache_size")
        .fetch_one(pool)
        .await?;

    // 执行简单查询测量响应时间
    let start = std::time::Instant::now();
    sqlx::query("SELECT COUNT(*) FROM sqlite_master").fetch_one(pool).await?;
    let query_time = start.elapsed();

    Ok(PerformanceMetrics {
        cache_hit_rate,
        page_cache_size: page_cache_hit,
        avg_query_time: query_time,
    })
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cache_hit_rate: f64,
    pub page_cache_size: i32,
    pub avg_query_time: Duration,
}

impl PerformanceMetrics {
    /// 格式化查询时间
    pub fn formatted_query_time(&self) -> String {
        if self.avg_query_time.as_millis() < 1 {
            format!("{:.2}μs", self.avg_query_time.as_micros())
        } else {
            format!("{:.2}ms", self.avg_query_time.as_millis())
        }
    }

    /// 检查性能是否良好
    pub fn is_healthy(&self) -> bool {
        self.cache_hit_rate > 80.0 && self.avg_query_time.as_millis() < 100
    }
}

/// 连接池健康检查
pub async fn health_check(pool: &SqlitePool) -> Result<HealthStatus> {
    let start = std::time::Instant::now();
    
    // 测试基本连接
    match test_connection(pool).await {
        Ok(_) => {},
        Err(e) => return Ok(HealthStatus {
            is_healthy: false,
            response_time: start.elapsed(),
            error_message: Some(e.to_string()),
            active_connections: 0,
            max_connections: pool.options().get_max_connections(),
        }),
    }

    let response_time = start.elapsed();
    
    // 获取连接池状态
    let active_connections = pool.size();
    let max_connections = pool.options().get_max_connections();

    // 检查响应时间
    let is_healthy = response_time.as_millis() < 1000 && active_connections < max_connections;

    Ok(HealthStatus {
        is_healthy,
        response_time,
        error_message: None,
        active_connections,
        max_connections,
    })
}

/// 健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time: Duration,
    pub error_message: Option<String>,
    pub active_connections: u32,
    pub max_connections: u32,
}

impl HealthStatus {
    /// 格式化响应时间
    pub fn formatted_response_time(&self) -> String {
        if self.response_time.as_millis() < 1 {
            format!("{:.2}μs", self.response_time.as_micros())
        } else {
            format!("{}ms", self.response_time.as_millis())
        }
    }

    /// 获取连接池使用率
    pub fn connection_usage_rate(&self) -> f64 {
        if self.max_connections == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.max_connections as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_connection_pool() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        assert!(test_connection(&pool).await.is_ok());
    }

    #[tokio::test]
    async fn test_database_info() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        let info = get_database_info(&pool).await.unwrap();
        
        assert!(!info.version.is_empty());
        assert!(info.page_size > 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        let health = health_check(&pool).await.unwrap();
        
        assert!(health.is_healthy);
        assert!(health.error_message.is_none());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(512), "512 B");
    }
}