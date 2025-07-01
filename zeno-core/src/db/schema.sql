-- ============================================================================
-- Zeno 数据库 Schema
-- 版本: 1.0.0
-- 创建时间: 2025-07-01
-- 描述: 知识管理系统的核心数据库结构
-- ============================================================================

-- 启用外键约束
PRAGMA foreign_keys = ON;

-- ============================================================================
-- 核心数据表
-- ============================================================================

-- 笔记表
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,                    -- 笔记唯一标识符 (UUID)
    title TEXT NOT NULL,                    -- 笔记标题
    file_path TEXT NOT NULL UNIQUE,         -- 文件路径 (相对于工作区)
    content TEXT NOT NULL DEFAULT '',       -- 原始 Markdown 内容
    html_content TEXT DEFAULT '',           -- 渲染后的 HTML 内容
    word_count INTEGER DEFAULT 0,           -- 字数统计
    reading_time INTEGER DEFAULT 0,         -- 预估阅读时间(分钟)
    
    -- 时间戳
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- 状态和元数据
    status TEXT DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived', 'deleted')),
    
    -- frontmatter 数据 (JSON 格式)
    frontmatter TEXT DEFAULT '{}',
    
    -- 文件系统元数据
    file_size INTEGER DEFAULT 0,
    file_hash TEXT DEFAULT '',              -- 文件内容哈希，用于检测变化
    
    -- 索引字段
    search_vector TEXT DEFAULT ''           -- 搜索向量缓存
);

-- 标签表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,              -- 标签名称
    color TEXT DEFAULT '#6B7280',           -- 标签颜色
    description TEXT DEFAULT '',            -- 标签描述
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    usage_count INTEGER DEFAULT 0           -- 使用次数
);

-- 笔记-标签关联表 (多对多)
CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 链接表 (双向链接和引用)
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL,                -- 源笔记 ID
    target_id TEXT NOT NULL,                -- 目标笔记 ID
    link_type TEXT NOT NULL DEFAULT 'reference' CHECK (link_type IN ('reference', 'embed', 'citation')),
    anchor_text TEXT DEFAULT '',            -- 链接锚文本
    
    -- 链接位置信息
    source_line INTEGER DEFAULT 0,          -- 源文件中的行号
    source_column INTEGER DEFAULT 0,        -- 源文件中的列号
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (source_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES notes(id) ON DELETE CASCADE,
    UNIQUE(source_id, target_id, anchor_text, source_line)
);

-- 分类表
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,              -- 分类名称
    parent_id INTEGER DEFAULT NULL,         -- 父分类 ID (支持层级结构)
    description TEXT DEFAULT '',            -- 分类描述
    color TEXT DEFAULT '#6B7280',           -- 分类颜色
    icon TEXT DEFAULT 'folder',             -- 分类图标
    sort_order INTEGER DEFAULT 0,           -- 排序顺序
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- 笔记-分类关联表
CREATE TABLE IF NOT EXISTS note_categories (
    note_id TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (note_id, category_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- ============================================================================
-- 全文搜索虚拟表 (FTS5)
-- ============================================================================

-- 全文搜索索引
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    content,
    tags,
    categories,
    
    -- 配置选项
    content='notes',                         -- 关联到 notes 表
    content_rowid='rowid',                   -- 使用 notes 表的 rowid
    tokenize='porter unicode61 remove_diacritics 1'  -- 分词器配置
);

-- ============================================================================
-- 系统配置和元数据表
-- ============================================================================

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT DEFAULT '',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 搜索历史表
CREATE TABLE IF NOT EXISTS search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    result_count INTEGER DEFAULT 0,
    search_time REAL DEFAULT 0.0,           -- 搜索耗时(秒)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 文件监控日志表
CREATE TABLE IF NOT EXISTS file_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('created', 'modified', 'deleted', 'renamed')),
    old_path TEXT DEFAULT NULL,             -- 重命名时的旧路径
    processed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 索引优化
-- ============================================================================

-- 笔记表索引
CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
CREATE INDEX IF NOT EXISTS idx_notes_status ON notes(status);
CREATE INDEX IF NOT EXISTS idx_notes_modified ON notes(modified_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_created ON notes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_file_path ON notes(file_path);
CREATE INDEX IF NOT EXISTS idx_notes_file_hash ON notes(file_hash);

-- 链接表索引
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_id);
CREATE INDEX IF NOT EXISTS idx_links_type ON links(link_type);

-- 标签相关索引
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
CREATE INDEX IF NOT EXISTS idx_tags_usage ON tags(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_note_tags_note ON note_tags(note_id);
CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag_id);

-- 分类相关索引
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_sort ON categories(sort_order);
CREATE INDEX IF NOT EXISTS idx_note_categories_note ON note_categories(note_id);
CREATE INDEX IF NOT EXISTS idx_note_categories_category ON note_categories(category_id);

-- 文件事件索引
CREATE INDEX IF NOT EXISTS idx_file_events_path ON file_events(file_path);
CREATE INDEX IF NOT EXISTS idx_file_events_type ON file_events(event_type);
CREATE INDEX IF NOT EXISTS idx_file_events_processed ON file_events(processed);
CREATE INDEX IF NOT EXISTS idx_file_events_created ON file_events(created_at);

-- ============================================================================
-- 触发器 (自动维护数据一致性)
-- ============================================================================

-- 更新笔记修改时间
CREATE TRIGGER IF NOT EXISTS update_notes_modified_time
AFTER UPDATE ON notes
FOR EACH ROW
WHEN NEW.content != OLD.content OR NEW.title != OLD.title
BEGIN
    UPDATE notes SET modified_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- 维护标签使用次数
CREATE TRIGGER IF NOT EXISTS increment_tag_usage
AFTER INSERT ON note_tags
FOR EACH ROW
BEGIN
    UPDATE tags SET usage_count = usage_count + 1 WHERE id = NEW.tag_id;
END;

CREATE TRIGGER IF NOT EXISTS decrement_tag_usage
AFTER DELETE ON note_tags
FOR EACH ROW
BEGIN
    UPDATE tags SET usage_count = usage_count - 1 WHERE id = OLD.tag_id;
END;

-- 自动更新全文搜索索引
CREATE TRIGGER IF NOT EXISTS update_fts_insert
AFTER INSERT ON notes
FOR EACH ROW
BEGIN
    INSERT INTO notes_fts(rowid, title, content, tags, categories)
    SELECT 
        NEW.rowid,
        NEW.title,
        NEW.content,
        COALESCE((
            SELECT GROUP_CONCAT(t.name, ' ')
            FROM note_tags nt
            JOIN tags t ON nt.tag_id = t.id
            WHERE nt.note_id = NEW.id
        ), ''),
        COALESCE((
            SELECT GROUP_CONCAT(c.name, ' ')
            FROM note_categories nc
            JOIN categories c ON nc.category_id = c.id
            WHERE nc.note_id = NEW.id
        ), '');
END;

CREATE TRIGGER IF NOT EXISTS update_fts_update
AFTER UPDATE ON notes
FOR EACH ROW
WHEN NEW.title != OLD.title OR NEW.content != OLD.content
BEGIN
    UPDATE notes_fts 
    SET 
        title = NEW.title,
        content = NEW.content,
        tags = COALESCE((
            SELECT GROUP_CONCAT(t.name, ' ')
            FROM note_tags nt
            JOIN tags t ON nt.tag_id = t.id
            WHERE nt.note_id = NEW.id
        ), ''),
        categories = COALESCE((
            SELECT GROUP_CONCAT(c.name, ' ')
            FROM note_categories nc
            JOIN categories c ON nc.category_id = c.id
            WHERE nc.note_id = NEW.id
        ), '')
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER IF NOT EXISTS update_fts_delete
AFTER DELETE ON notes
FOR EACH ROW
BEGIN
    DELETE FROM notes_fts WHERE rowid = OLD.rowid;
END;

-- ============================================================================
-- 初始化数据
-- ============================================================================

-- 插入系统默认配置
INSERT OR IGNORE INTO system_config (key, value, description) VALUES
('db_version', '1.0.0', '数据库版本'),
('workspace_path', '', '工作区路径'),
('auto_backup', 'true', '自动备份开关'),
('backup_interval', '24', '备份间隔(小时)'),
('search_limit', '100', '搜索结果限制'),
('file_watch_enabled', 'true', '文件监控开关');

-- 创建默认分类
INSERT OR IGNORE INTO categories (name, description, icon, sort_order) VALUES
('未分类', '默认分类', 'folder', 0),
('归档', '已归档的笔记', 'archive', 999);

-- ============================================================================
-- 视图 (便于查询)
-- ============================================================================

-- 笔记详情视图 (包含标签和分类信息)
CREATE VIEW IF NOT EXISTS note_details AS
SELECT 
    n.*,
    COALESCE(GROUP_CONCAT(DISTINCT t.name), '') as tag_names,
    COALESCE(GROUP_CONCAT(DISTINCT c.name), '') as category_names,
    COUNT(DISTINCT l_out.id) as outbound_links,
    COUNT(DISTINCT l_in.id) as inbound_links
FROM notes n
LEFT JOIN note_tags nt ON n.id = nt.note_id
LEFT JOIN tags t ON nt.tag_id = t.id
LEFT JOIN note_categories nc ON n.id = nc.note_id
LEFT JOIN categories c ON nc.category_id = c.id
LEFT JOIN links l_out ON n.id = l_out.source_id
LEFT JOIN links l_in ON n.id = l_in.target_id
GROUP BY n.id;

-- 标签使用统计视图
CREATE VIEW IF NOT EXISTS tag_stats AS
SELECT 
    t.*,
    COUNT(nt.note_id) as actual_usage_count,
    MAX(n.modified_at) as last_used
FROM tags t
LEFT JOIN note_tags nt ON t.id = nt.tag_id
LEFT JOIN notes n ON nt.note_id = n.id
GROUP BY t.id;

-- 链接关系图视图
CREATE VIEW IF NOT EXISTS link_graph AS
SELECT 
    l.id,
    l.source_id,
    s.title as source_title,
    s.file_path as source_path,
    l.target_id,
    t.title as target_title,
    t.file_path as target_path,
    l.link_type,
    l.anchor_text,
    l.created_at
FROM links l
JOIN notes s ON l.source_id = s.id
JOIN notes t ON l.target_id = t.id
WHERE s.status != 'deleted' AND t.status != 'deleted';