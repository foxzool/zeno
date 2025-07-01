use crate::error::{Error, Result};
use crate::db::{Database, models::*};
use crate::parser::MarkdownParser;
use sqlx::{SqlitePool, Row};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use async_trait::async_trait;

/// 笔记仓库接口
#[async_trait]
pub trait NoteRepository: Send + Sync {
    // 笔记 CRUD 操作
    async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>>;
    async fn get_note_by_path(&self, path: &str) -> Result<Option<Note>>;
    async fn save_note(&self, note: &Note) -> Result<()>;
    async fn delete_note(&self, id: &str) -> Result<()>;
    async fn list_notes(&self, filter: Option<NoteFilter>) -> Result<Vec<Note>>;
    
    // 标签操作
    async fn get_all_tags(&self) -> Result<Vec<Tag>>;
    async fn get_notes_by_tag(&self, tag_name: &str) -> Result<Vec<Note>>;
    async fn create_tag(&self, tag: &Tag) -> Result<i64>;
    async fn update_tag(&self, tag: &Tag) -> Result<()>;
    async fn delete_tag(&self, tag_id: i64) -> Result<()>;
    
    // 分类操作
    async fn get_all_categories(&self) -> Result<Vec<Category>>;
    async fn get_category_tree(&self) -> Result<Vec<TreeNode>>;
    async fn get_notes_by_category(&self, category_id: i64) -> Result<Vec<Note>>;
    async fn create_category(&self, category: &Category) -> Result<i64>;
    async fn update_category(&self, category: &Category) -> Result<()>;
    async fn delete_category(&self, category_id: i64) -> Result<()>;
    
    // 链接操作
    async fn get_outbound_links(&self, note_id: &str) -> Result<Vec<Link>>;
    async fn get_inbound_links(&self, note_id: &str) -> Result<Vec<Link>>;
    async fn create_link(&self, link: &Link) -> Result<i64>;
    async fn delete_links_for_note(&self, note_id: &str) -> Result<()>;
    
    // 搜索操作
    async fn search_notes(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    async fn get_related_notes(&self, note_id: &str, limit: Option<usize>) -> Result<Vec<Note>>;
    
    // 统计操作
    async fn get_statistics(&self) -> Result<Statistics>;
}

/// 笔记过滤器
#[derive(Debug, Clone, Default)]
pub struct NoteFilter {
    pub status: Option<NoteStatus>,
    pub tags: Vec<String>,
    pub categories: Vec<i64>,
    pub date_range: Option<DateRange>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// SQLite 笔记仓库实现
pub struct SqliteNoteRepository {
    pool: SqlitePool,
    parser: MarkdownParser,
}

impl SqliteNoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            parser: MarkdownParser::new(),
        }
    }

    /// 从数据库结果构建笔记详情
    async fn build_note_with_relations(&self, note: Note) -> Result<(Note, Vec<Tag>, Vec<Category>, Vec<Link>, Vec<Link>)> {
        let tags = self.get_tags_for_note(&note.id).await?;
        let categories = self.get_categories_for_note(&note.id).await?;
        let outbound_links = self.get_outbound_links(&note.id).await?;
        let inbound_links = self.get_inbound_links(&note.id).await?;

        Ok((note, tags, categories, outbound_links, inbound_links))
    }

    /// 获取笔记的标签
    async fn get_tags_for_note(&self, note_id: &str) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT t.* FROM tags t
            JOIN note_tags nt ON t.id = nt.tag_id
            WHERE nt.note_id = ?
            ORDER BY t.name
            "#
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// 获取笔记的分类
    async fn get_categories_for_note(&self, note_id: &str) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            r#"
            SELECT c.* FROM categories c
            JOIN note_categories nc ON c.id = nc.category_id
            WHERE nc.note_id = ?
            ORDER BY c.name
            "#
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// 更新笔记标签关联
    async fn update_note_tags(&self, note_id: &str, tag_names: &[String]) -> Result<()> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 删除现有关联
        sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        // 创建或获取标签并建立关联
        for tag_name in tag_names {
            // 确保标签存在
            let tag_id: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM tags WHERE name = ?"
            )
            .bind(tag_name)
            .fetch_optional(&mut *tx)
            .await?;

            let tag_id = if let Some(id) = tag_id {
                id
            } else {
                // 创建新标签
                sqlx::query_scalar(
                    "INSERT INTO tags (name) VALUES (?) RETURNING id"
                )
                .bind(tag_name)
                .fetch_one(&mut *tx)
                .await?
            };

            // 建立关联
            sqlx::query(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?, ?)"
            )
            .bind(note_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 更新笔记分类关联
    async fn update_note_categories(&self, note_id: &str, category_ids: &[i64]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 删除现有关联
        sqlx::query("DELETE FROM note_categories WHERE note_id = ?")
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        // 建立新关联
        for category_id in category_ids {
            sqlx::query(
                "INSERT INTO note_categories (note_id, category_id) VALUES (?, ?)"
            )
            .bind(note_id)
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 更新笔记链接
    async fn update_note_links(&self, note_id: &str, links: &[Link]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 删除现有链接
        sqlx::query("DELETE FROM links WHERE source_id = ?")
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        // 插入新链接
        for link in links {
            sqlx::query(
                r#"
                INSERT INTO links (source_id, target_id, link_type, anchor_text, source_line, source_column)
                VALUES (?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&link.source_id)
            .bind(&link.target_id)
            .bind(&link.link_type)
            .bind(&link.anchor_text)
            .bind(link.source_line)
            .bind(link.source_column)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl NoteRepository for SqliteNoteRepository {
    async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>> {
        let note = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE id = ? AND status != 'deleted'"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(note)
    }

    async fn get_note_by_path(&self, path: &str) -> Result<Option<Note>> {
        let note = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE file_path = ? AND status != 'deleted'"
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        Ok(note)
    }

    async fn save_note(&self, note: &Note) -> Result<()> {
        // 检查笔记是否已存在
        let existing = self.get_note_by_id(&note.id).await?;
        
        if existing.is_some() {
            // 更新现有笔记
            sqlx::query(
                r#"
                UPDATE notes SET
                    title = ?, file_path = ?, content = ?, html_content = ?,
                    word_count = ?, reading_time = ?, modified_at = CURRENT_TIMESTAMP,
                    status = ?, frontmatter = ?, file_size = ?, file_hash = ?
                WHERE id = ?
                "#
            )
            .bind(&note.title)
            .bind(&note.file_path)
            .bind(&note.content)
            .bind(&note.html_content)
            .bind(note.word_count)
            .bind(note.reading_time)
            .bind(&note.status)
            .bind(&note.frontmatter)
            .bind(note.file_size)
            .bind(&note.file_hash)
            .bind(&note.id)
            .execute(&self.pool)
            .await?;
        } else {
            // 插入新笔记
            sqlx::query(
                r#"
                INSERT INTO notes (
                    id, title, file_path, content, html_content, word_count, reading_time,
                    created_at, modified_at, status, frontmatter, file_size, file_hash
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&note.id)
            .bind(&note.title)
            .bind(&note.file_path)
            .bind(&note.content)
            .bind(&note.html_content)
            .bind(note.word_count)
            .bind(note.reading_time)
            .bind(note.created_at)
            .bind(note.modified_at)
            .bind(&note.status)
            .bind(&note.frontmatter)
            .bind(note.file_size)
            .bind(&note.file_hash)
            .execute(&self.pool)
            .await?;
        }

        // 解析 frontmatter 并更新关联数据
        if let Ok(frontmatter) = note.get_frontmatter() {
            // 更新标签
            if !frontmatter.tags.is_empty() {
                self.update_note_tags(&note.id, &frontmatter.tags).await?;
            }

            // 更新分类（这里简化处理，实际需要根据分类名称查找ID）
            // TODO: 实现分类名称到ID的映射
        }

        Ok(())
    }

    async fn delete_note(&self, id: &str) -> Result<()> {
        // 软删除：将状态设置为 deleted
        sqlx::query(
            "UPDATE notes SET status = 'deleted', modified_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_notes(&self, filter: Option<NoteFilter>) -> Result<Vec<Note>> {
        let filter = filter.unwrap_or_default();
        
        let mut query = "SELECT * FROM notes WHERE status != 'deleted'".to_string();
        let mut bindings = Vec::new();

        // 应用过滤条件
        if let Some(status) = &filter.status {
            query.push_str(" AND status = ?");
            bindings.push(status.to_string());
        }

        // 添加排序
        query.push_str(" ORDER BY modified_at DESC");

        // 添加分页
        if let Some(limit) = filter.limit {
            query.push_str(" LIMIT ?");
            bindings.push(limit.to_string());
            
            if let Some(offset) = filter.offset {
                query.push_str(" OFFSET ?");
                bindings.push(offset.to_string());
            }
        }

        let mut sql_query = sqlx::query_as::<_, Note>(&query);
        for binding in bindings {
            sql_query = sql_query.bind(binding);
        }

        let notes = sql_query.fetch_all(&self.pool).await?;
        Ok(notes)
    }

    async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT * FROM tags ORDER BY usage_count DESC, name ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    async fn get_notes_by_tag(&self, tag_name: &str) -> Result<Vec<Note>> {
        let notes = sqlx::query_as::<_, Note>(
            r#"
            SELECT n.* FROM notes n
            JOIN note_tags nt ON n.id = nt.note_id
            JOIN tags t ON nt.tag_id = t.id
            WHERE t.name = ? AND n.status != 'deleted'
            ORDER BY n.modified_at DESC
            "#
        )
        .bind(tag_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(notes)
    }

    async fn create_tag(&self, tag: &Tag) -> Result<i64> {
        let id = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name, color, description)
            VALUES (?, ?, ?)
            RETURNING id
            "#
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update_tag(&self, tag: &Tag) -> Result<()> {
        sqlx::query(
            "UPDATE tags SET name = ?, color = ?, description = ? WHERE id = ?"
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.description)
        .bind(tag.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_tag(&self, tag_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 删除标签关联
        sqlx::query("DELETE FROM note_tags WHERE tag_id = ?")
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;

        // 删除标签
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_all_categories(&self) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories ORDER BY sort_order ASC, name ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    async fn get_category_tree(&self) -> Result<Vec<TreeNode>> {
        let categories = self.get_all_categories().await?;
        let mut tree = Vec::new();
        let mut category_map: HashMap<i64, Vec<Category>> = HashMap::new();

        // 按父ID分组
        for category in categories {
            let parent_id = category.parent_id.unwrap_or(0);
            category_map.entry(parent_id).or_default().push(category);
        }

        // 构建树结构
        fn build_tree_nodes(
            parent_id: i64,
            category_map: &HashMap<i64, Vec<Category>>,
        ) -> Vec<TreeNode> {
            category_map
                .get(&parent_id)
                .map(|categories| {
                    categories
                        .iter()
                        .map(|category| TreeNode {
                            category: category.clone(),
                            children: build_tree_nodes(category.id, category_map),
                            note_count: 0, // TODO: 计算笔记数量
                        })
                        .collect()
                })
                .unwrap_or_default()
        }

        tree = build_tree_nodes(0, &category_map);
        Ok(tree)
    }

    async fn get_notes_by_category(&self, category_id: i64) -> Result<Vec<Note>> {
        let notes = sqlx::query_as::<_, Note>(
            r#"
            SELECT n.* FROM notes n
            JOIN note_categories nc ON n.id = nc.note_id
            WHERE nc.category_id = ? AND n.status != 'deleted'
            ORDER BY n.modified_at DESC
            "#
        )
        .bind(category_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(notes)
    }

    async fn create_category(&self, category: &Category) -> Result<i64> {
        let id = sqlx::query_scalar(
            r#"
            INSERT INTO categories (name, parent_id, description, color, icon, sort_order)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(&category.name)
        .bind(category.parent_id)
        .bind(&category.description)
        .bind(&category.color)
        .bind(&category.icon)
        .bind(category.sort_order)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update_category(&self, category: &Category) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE categories 
            SET name = ?, parent_id = ?, description = ?, color = ?, icon = ?, sort_order = ?
            WHERE id = ?
            "#
        )
        .bind(&category.name)
        .bind(category.parent_id)
        .bind(&category.description)
        .bind(&category.color)
        .bind(&category.icon)
        .bind(category.sort_order)
        .bind(category.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_category(&self, category_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 删除分类关联
        sqlx::query("DELETE FROM note_categories WHERE category_id = ?")
            .bind(category_id)
            .execute(&mut *tx)
            .await?;

        // 更新子分类的父ID为NULL
        sqlx::query("UPDATE categories SET parent_id = NULL WHERE parent_id = ?")
            .bind(category_id)
            .execute(&mut *tx)
            .await?;

        // 删除分类
        sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(category_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_outbound_links(&self, note_id: &str) -> Result<Vec<Link>> {
        let links = sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE source_id = ? ORDER BY created_at ASC"
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }

    async fn get_inbound_links(&self, note_id: &str) -> Result<Vec<Link>> {
        let links = sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE target_id = ? ORDER BY created_at ASC"
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }

    async fn create_link(&self, link: &Link) -> Result<i64> {
        let id = sqlx::query_scalar(
            r#"
            INSERT INTO links (source_id, target_id, link_type, anchor_text, source_line, source_column)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(&link.source_id)
        .bind(&link.target_id)
        .bind(&link.link_type)
        .bind(&link.anchor_text)
        .bind(link.source_line)
        .bind(link.source_column)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn delete_links_for_note(&self, note_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM links WHERE source_id = ? OR target_id = ?")
            .bind(note_id)
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn search_notes(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // 使用 FTS5 进行全文搜索
        let search_sql = r#"
            SELECT n.*, 
                   bm25(notes_fts) as score,
                   snippet(notes_fts, 1, '<mark>', '</mark>', '...', 32) as highlight
            FROM notes_fts
            JOIN notes n ON notes_fts.rowid = n.rowid
            WHERE notes_fts MATCH ? AND n.status != 'deleted'
            ORDER BY bm25(notes_fts)
            LIMIT ?
        "#;

        let limit = query.limit.unwrap_or(50);
        let rows = sqlx::query(search_sql)
            .bind(&query.query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let note = Note {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                content: row.get("content"),
                html_content: row.get("html_content"),
                word_count: row.get("word_count"),
                reading_time: row.get("reading_time"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                indexed_at: row.get("indexed_at"),
                status: row.get("status"),
                frontmatter: row.get("frontmatter"),
                file_size: row.get("file_size"),
                file_hash: row.get("file_hash"),
                search_vector: row.get("search_vector"),
            };

            let score: f64 = row.get("score");
            let highlight: String = row.get("highlight");

            results.push(SearchResult {
                note,
                score: score as f32,
                highlights: vec![highlight],
                matched_fields: vec!["content".to_string()],
            });
        }

        Ok(results)
    }

    async fn get_related_notes(&self, note_id: &str, limit: Option<usize>) -> Result<Vec<Note>> {
        let limit = limit.unwrap_or(10);
        
        // 通过标签和链接查找相关笔记
        let notes = sqlx::query_as::<_, Note>(
            r#"
            SELECT DISTINCT n.*, 
                   (CASE WHEN l.target_id IS NOT NULL THEN 3 ELSE 0 END +
                    CASE WHEN shared_tags.cnt IS NOT NULL THEN shared_tags.cnt ELSE 0 END) as relevance_score
            FROM notes n
            LEFT JOIN links l ON n.id = l.target_id AND l.source_id = ?
            LEFT JOIN (
                SELECT n2.id, COUNT(*) as cnt
                FROM notes n2
                JOIN note_tags nt2 ON n2.id = nt2.note_id
                WHERE nt2.tag_id IN (
                    SELECT nt1.tag_id FROM note_tags nt1 WHERE nt1.note_id = ?
                )
                AND n2.id != ?
                GROUP BY n2.id
            ) shared_tags ON n.id = shared_tags.id
            WHERE n.id != ? AND n.status != 'deleted'
            AND (l.target_id IS NOT NULL OR shared_tags.cnt IS NOT NULL)
            ORDER BY relevance_score DESC
            LIMIT ?
            "#
        )
        .bind(note_id)
        .bind(note_id)
        .bind(note_id)
        .bind(note_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(notes)
    }

    async fn get_statistics(&self) -> Result<Statistics> {
        // 这里重用 Database 的 get_statistics 方法
        let db = Database::from_pool(self.pool.clone());
        db.get_statistics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::create_connection_pool;

    async fn setup_test_db() -> SqlitePool {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = create_connection_pool(&db_url).await.unwrap();
        let db = Database::new(&db_path).await.unwrap();
        db.initialize().await.unwrap();
        
        pool
    }

    #[tokio::test]
    async fn test_note_crud() {
        let pool = setup_test_db().await;
        let repo = SqliteNoteRepository::new(pool);

        let mut note = Note::new("测试笔记".to_string(), "test.md".to_string(), "# 测试\n\n内容".to_string());
        
        // 创建笔记
        repo.save_note(&note).await.unwrap();
        
        // 读取笔记
        let saved_note = repo.get_note_by_id(&note.id).await.unwrap().unwrap();
        assert_eq!(saved_note.title, note.title);
        
        // 更新笔记
        note.title = "更新后的标题".to_string();
        repo.save_note(&note).await.unwrap();
        
        let updated_note = repo.get_note_by_id(&note.id).await.unwrap().unwrap();
        assert_eq!(updated_note.title, "更新后的标题");
        
        // 删除笔记
        repo.delete_note(&note.id).await.unwrap();
        let deleted_note = repo.get_note_by_id(&note.id).await.unwrap();
        assert!(deleted_note.is_none());
    }

    #[tokio::test]
    async fn test_tag_operations() {
        let pool = setup_test_db().await;
        let repo = SqliteNoteRepository::new(pool);

        let tag = Tag::new("测试标签".to_string());
        
        // 创建标签
        let tag_id = repo.create_tag(&tag).await.unwrap();
        assert!(tag_id > 0);
        
        // 获取所有标签
        let tags = repo.get_all_tags().await.unwrap();
        assert!(!tags.is_empty());
    }

    #[tokio::test]
    async fn test_search() {
        let pool = setup_test_db().await;
        let repo = SqliteNoteRepository::new(pool);

        // 创建测试笔记
        let note = Note::new("搜索测试".to_string(), "search_test.md".to_string(), "这是一个用于搜索的测试笔记".to_string());
        repo.save_note(&note).await.unwrap();

        // 搜索
        let query = SearchQuery {
            query: "搜索".to_string(),
            limit: Some(10),
            offset: None,
            filters: SearchFilters::default(),
        };

        let results = repo.search_notes(&query).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].note.title.contains("搜索"));
    }
}