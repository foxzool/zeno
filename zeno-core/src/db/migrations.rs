use sqlx::{SqlitePool, Row};
use crate::error::Result;

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // 确保迁移表存在
    create_migration_table(pool).await?;
    
    // 获取当前版本
    let current_version = get_current_version(pool).await?;
    
    // 执行所有需要的迁移
    let migrations = get_migrations();
    
    for migration in migrations {
        if migration.version > current_version {
            log::info!("执行迁移: {} - {}", migration.version, migration.description);
            
            // 开始事务
            let mut tx = pool.begin().await?;
            
            // 执行迁移脚本
            for sql in &migration.up_sql {
                sqlx::query(sql).execute(&mut *tx).await.map_err(|e| {
                    log::error!("迁移失败 (版本 {}): {}", migration.version, e);
                    e
                })?;
            }
            
            // 更新版本记录
            sqlx::query(
                "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
            )
            .bind(migration.version)
            .bind(&migration.description)
            .execute(&mut *tx)
            .await?;
            
            // 提交事务
            tx.commit().await?;
            
            log::info!("迁移完成: 版本 {}", migration.version);
        }
    }
    
    Ok(())
}

/// 创建迁移表
async fn create_migration_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

/// 获取当前数据库版本
async fn get_current_version(pool: &SqlitePool) -> Result<i32> {
    let version = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(version)
}

/// 迁移定义
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i32,
    pub description: String,
    pub up_sql: Vec<String>,
    pub down_sql: Vec<String>,
}

/// 获取所有迁移
fn get_migrations() -> Vec<Migration> {
    vec![
        // 版本 1: 初始Schema
        Migration {
            version: 1,
            description: "初始数据库结构".to_string(),
            up_sql: vec![
                include_str!("schema.sql").to_string(),
            ],
            down_sql: vec![
                "DROP TABLE IF EXISTS notes_fts;".to_string(),
                "DROP TABLE IF EXISTS file_events;".to_string(),
                "DROP TABLE IF EXISTS search_history;".to_string(),
                "DROP TABLE IF EXISTS system_config;".to_string(),
                "DROP TABLE IF EXISTS note_categories;".to_string(),
                "DROP TABLE IF EXISTS categories;".to_string(),
                "DROP TABLE IF EXISTS links;".to_string(),
                "DROP TABLE IF EXISTS note_tags;".to_string(),
                "DROP TABLE IF EXISTS tags;".to_string(),
                "DROP TABLE IF EXISTS notes;".to_string(),
            ],
        },
        
        // 版本 2: 性能优化
        Migration {
            version: 2,
            description: "性能优化和索引改进".to_string(),
            up_sql: vec![
                // 添加复合索引
                "CREATE INDEX IF NOT EXISTS idx_notes_status_modified ON notes(status, modified_at DESC);".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_links_source_type ON links(source_id, link_type);".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_note_tags_composite ON note_tags(note_id, tag_id);".to_string(),
                
                // 优化 FTS 配置
                "INSERT INTO notes_fts(notes_fts, rank) VALUES('rank', 'bm25(10.0, 5.0, 1.0)');".to_string(),
                
                // 添加统计表
                r#"
                CREATE TABLE IF NOT EXISTS statistics_cache (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    expires_at TIMESTAMP
                );
                "#.to_string(),
            ],
            down_sql: vec![
                "DROP TABLE IF EXISTS statistics_cache;".to_string(),
                "DROP INDEX IF EXISTS idx_notes_status_modified;".to_string(),
                "DROP INDEX IF EXISTS idx_links_source_type;".to_string(),
                "DROP INDEX IF EXISTS idx_note_tags_composite;".to_string(),
            ],
        },
        
        // 版本 3: 增加用户配置
        Migration {
            version: 3,
            description: "用户配置和主题支持".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE IF NOT EXISTS user_preferences (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    key TEXT NOT NULL UNIQUE,
                    value TEXT NOT NULL,
                    type TEXT DEFAULT 'string' CHECK (type IN ('string', 'number', 'boolean', 'json')),
                    description TEXT DEFAULT '',
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#.to_string(),
                
                // 插入默认配置
                r#"
                INSERT OR IGNORE INTO user_preferences (key, value, type, description) VALUES
                ('theme', 'light', 'string', '界面主题'),
                ('font_size', '14', 'number', '字体大小'),
                ('auto_save', 'true', 'boolean', '自动保存'),
                ('editor_mode', 'wysiwyg', 'string', '编辑器模式'),
                ('backup_enabled', 'true', 'boolean', '自动备份开关'),
                ('backup_interval', '24', 'number', '备份间隔(小时)');
                "#.to_string(),
                
                // 添加配置更新触发器
                r#"
                CREATE TRIGGER IF NOT EXISTS update_user_preferences_timestamp
                AFTER UPDATE ON user_preferences
                FOR EACH ROW
                BEGIN
                    UPDATE user_preferences SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
                END;
                "#.to_string(),
            ],
            down_sql: vec![
                "DROP TRIGGER IF EXISTS update_user_preferences_timestamp;".to_string(),
                "DROP TABLE IF EXISTS user_preferences;".to_string(),
            ],
        },
        
        // 版本 4: 工作空间支持
        Migration {
            version: 4,
            description: "多工作空间支持".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE IF NOT EXISTS workspaces (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL UNIQUE,
                    description TEXT DEFAULT '',
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    is_active BOOLEAN DEFAULT FALSE
                );
                "#.to_string(),
                
                // 添加工作空间字段到笔记表
                "ALTER TABLE notes ADD COLUMN workspace_id TEXT DEFAULT NULL;".to_string(),
                
                // 创建默认工作空间
                r#"
                INSERT OR IGNORE INTO workspaces (id, name, path, description, is_active)
                VALUES ('default', '默认工作空间', '.', '默认工作空间', TRUE);
                "#.to_string(),
                
                // 更新现有笔记的工作空间
                "UPDATE notes SET workspace_id = 'default' WHERE workspace_id IS NULL;".to_string(),
                
                // 添加工作空间索引
                "CREATE INDEX IF NOT EXISTS idx_notes_workspace ON notes(workspace_id);".to_string(),
            ],
            down_sql: vec![
                "DROP INDEX IF EXISTS idx_notes_workspace;".to_string(),
                "ALTER TABLE notes DROP COLUMN workspace_id;".to_string(),
                "DROP TABLE IF EXISTS workspaces;".to_string(),
            ],
        },
        
        // 版本 5: 插件系统支持
        Migration {
            version: 5,
            description: "插件系统和扩展支持".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE IF NOT EXISTS plugins (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    version TEXT NOT NULL,
                    enabled BOOLEAN DEFAULT TRUE,
                    config TEXT DEFAULT '{}',
                    install_path TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#.to_string(),
                
                r#"
                CREATE TABLE IF NOT EXISTS plugin_data (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    plugin_id TEXT NOT NULL,
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
                    UNIQUE(plugin_id, key)
                );
                "#.to_string(),
                
                "CREATE INDEX IF NOT EXISTS idx_plugin_data_plugin ON plugin_data(plugin_id);".to_string(),
            ],
            down_sql: vec![
                "DROP INDEX IF EXISTS idx_plugin_data_plugin;".to_string(),
                "DROP TABLE IF EXISTS plugin_data;".to_string(),
                "DROP TABLE IF EXISTS plugins;".to_string(),
            ],
        },
    ]
}

/// 回滚到指定版本
pub async fn rollback_to_version(pool: &SqlitePool, target_version: i32) -> Result<()> {
    let current_version = get_current_version(pool).await?;
    
    if target_version >= current_version {
        return Ok(()); // 无需回滚
    }
    
    let migrations = get_migrations();
    
    // 按版本倒序执行回滚
    for migration in migrations.iter().rev() {
        if migration.version > target_version && migration.version <= current_version {
            log::info!("回滚迁移: {} - {}", migration.version, migration.description);
            
            // 开始事务
            let mut tx = pool.begin().await?;
            
            // 执行回滚脚本
            for sql in &migration.down_sql {
                sqlx::query(sql).execute(&mut *tx).await.map_err(|e| {
                    log::error!("回滚失败 (版本 {}): {}", migration.version, e);
                    e
                })?;
            }
            
            // 删除版本记录
            sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
                .bind(migration.version)
                .execute(&mut *tx)
                .await?;
            
            // 提交事务
            tx.commit().await?;
            
            log::info!("回滚完成: 版本 {}", migration.version);
        }
    }
    
    Ok(())
}

/// 获取迁移历史
pub async fn get_migration_history(pool: &SqlitePool) -> Result<Vec<MigrationRecord>> {
    let records = sqlx::query_as::<_, MigrationRecord>(
        "SELECT version, description, applied_at FROM schema_migrations ORDER BY version DESC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(records)
}

/// 迁移记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MigrationRecord {
    pub version: i32,
    pub description: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

/// 检查数据库是否需要迁移
pub async fn needs_migration(pool: &SqlitePool) -> Result<bool> {
    let current_version = get_current_version(pool).await?;
    let latest_version = get_migrations().last().map(|m| m.version).unwrap_or(0);
    
    Ok(current_version < latest_version)
}

/// 验证迁移完整性
pub async fn validate_migrations(pool: &SqlitePool) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    
    // 检查表是否存在
    let required_tables = vec![
        "notes", "tags", "note_tags", "categories", "note_categories",
        "links", "notes_fts", "system_config", "file_events"
    ];
    
    for table in required_tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if !exists {
            errors.push(format!("缺少必需的表: {}", table));
        }
    }
    
    // 检查关键索引是否存在
    let required_indexes = vec![
        "idx_notes_title", "idx_notes_status", "idx_links_source", "idx_links_target"
    ];
    
    for index in required_indexes {
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?"
        )
        .bind(index)
        .fetch_one(pool)
        .await?;
        
        if !exists {
            errors.push(format!("缺少必需的索引: {}", index));
        }
    }
    
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::create_connection_pool;

    #[tokio::test]
    async fn test_migrations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        
        // 测试迁移
        run_migrations(&pool).await.unwrap();
        
        // 验证迁移
        let errors = validate_migrations(&pool).await.unwrap();
        assert!(errors.is_empty(), "迁移验证失败: {:?}", errors);
        
        // 检查版本
        let version = get_current_version(&pool).await.unwrap();
        assert!(version > 0);
    }

    #[tokio::test]
    async fn test_migration_history() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        run_migrations(&pool).await.unwrap();
        
        let history = get_migration_history(&pool).await.unwrap();
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_needs_migration() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        
        // 新数据库需要迁移
        assert!(needs_migration(&pool).await.unwrap());
        
        // 运行迁移后不需要
        run_migrations(&pool).await.unwrap();
        assert!(!needs_migration(&pool).await.unwrap());
    }
}