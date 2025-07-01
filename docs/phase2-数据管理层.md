# Phase 2: 数据管理层开发计划

## 阶段概述

实现Noto系统的核心数据处理能力，包括文件系统管理、Markdown解析、索引系统和存储抽象层。这是整个系统的基础，必须确保稳定可靠。

**预计时间**: 3-4周  
**优先级**: 最高 (核心基础)  
**前置条件**: Phase 1基础架构完成

## 目标与交付物

### 主要目标
- 实现稳定的文件监控和管理系统
- 建立高效的Markdown解析和处理能力
- 完成全文搜索和索引系统
- 建立数据存储抽象层

### 交付物
- 文件监控服务 (实时检测文件变化)
- Markdown解析器 (支持frontmatter和扩展语法)
- SQLite索引系统 (支持全文搜索)
- 存储抽象接口 (统一数据访问)

## 详细任务清单

### 2.1 文件系统管理

**任务描述**: 实现文件监控和基础文件操作

**核心功能**:
```rust
// services/file_watcher.rs
use notify::{DebouncedEvent, RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct FileWatcherService {
    base_path: PathBuf,
    watcher: Option<RecommendedWatcher>,
    event_sender: mpsc::Sender<FileEvent>,
}

impl FileWatcherService {
    pub fn new(base_path: PathBuf) -> Self;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    
    // 处理文件事件
    async fn handle_event(&self, event: DebouncedEvent) -> Result<()>;
}
```

**文件操作接口**:
```rust
// services/file_manager.rs
pub trait FileManager {
    async fn read_file(&self, path: &Path) -> Result<String>;
    async fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    async fn delete_file(&self, path: &Path) -> Result<()>;
    async fn move_file(&self, from: &Path, to: &Path) -> Result<()>;
    async fn list_files(&self, dir: &Path, recursive: bool) -> Result<Vec<PathBuf>>;
}
```

**实现要求**:
- 支持实时文件变化检测 (< 1秒延迟)
- 处理文件并发访问和锁定
- 支持大量文件的监控 (10k+ 文件)
- 跨平台兼容性 (Windows/macOS/Linux)

**验收标准**:
- [ ] 能够检测到文件的创建、修改、删除
- [ ] 文件操作在并发环境下的安全性
- [ ] 性能测试: 1000个文件变化在10秒内处理完成
- [ ] 内存使用稳定，无内存泄漏

### 2.2 Markdown解析器

**任务描述**: 实现强大的Markdown解析和处理能力

**解析器接口**:
```rust
// services/parser.rs
use markdown::{ParseOptions, CompileOptions};

pub struct MarkdownParser {
    options: ParseOptions,
}

#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub frontmatter: Frontmatter,
    pub content: String,
    pub html: String,
    pub toc: Vec<TocItem>,
    pub links: Vec<Link>,
    pub tags: Vec<String>,
    pub word_count: usize,
    pub reading_time: usize,
}

impl MarkdownParser {
    pub fn new() -> Self;
    pub fn parse(&self, content: &str) -> Result<ParsedNote>;
    pub fn extract_frontmatter(&self, content: &str) -> Result<Frontmatter>;
    pub fn extract_links(&self, content: &str) -> Result<Vec<Link>>;
    pub fn generate_toc(&self, content: &str) -> Result<Vec<TocItem>>;
}
```

**支持特性**:
- YAML frontmatter解析
- 双向链接语法 `[[链接]]`
- 数学公式支持 (KaTeX)
- 代码块语法高亮
- 表格、任务列表等扩展语法
- 自定义标签和元数据提取

**frontmatter结构**:
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frontmatter {
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub status: NoteStatus,
    pub publish: Option<PublishConfig>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

**验收标准**:
- [ ] 能够正确解析标准Markdown语法
- [ ] frontmatter提取和解析正确
- [ ] 双向链接识别和提取
- [ ] 性能测试: 10MB文档解析时间 < 100ms
- [ ] 边界情况处理 (恶意输入、格式错误等)

### 2.3 索引系统实现

**任务描述**: 建立高效的搜索和索引系统

**数据库Schema扩展**:
```sql
-- 全文搜索虚拟表
CREATE VIRTUAL TABLE notes_fts USING fts5(
    title,
    content,
    tags,
    content=notes,
    content_rowid=rowid,
    tokenize='porter unicode61 remove_diacritics 1'
);

-- 链接关系表
CREATE TABLE links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT NOT NULL DEFAULT 'reference',
    anchor_text TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_id) REFERENCES notes(id),
    FOREIGN KEY (target_id) REFERENCES notes(id),
    UNIQUE(source_id, target_id, anchor_text)
);

-- 索引优化
CREATE INDEX idx_notes_status ON notes(status);
CREATE INDEX idx_notes_modified ON notes(modified_at DESC);
CREATE INDEX idx_links_source ON links(source_id);
CREATE INDEX idx_links_target ON links(target_id);
```

**索引服务接口**:
```rust
// services/indexer.rs
pub struct IndexerService {
    db_pool: SqlitePool,
    parser: MarkdownParser,
}

impl IndexerService {
    pub async fn index_note(&self, path: &Path) -> Result<()>;
    pub async fn remove_note(&self, path: &Path) -> Result<()>;
    pub async fn update_note(&self, path: &Path) -> Result<()>;
    pub async fn rebuild_index(&self) -> Result<()>;
    
    // 搜索功能
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>>;
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<Note>>;
    pub async fn get_related_notes(&self, note_id: &str) -> Result<Vec<Note>>;
}
```

**搜索功能**:
- 全文搜索 (标题、内容、标签)
- 模糊匹配和拼写纠错
- 搜索结果高亮
- 搜索历史和建议
- 高级搜索语法 (布尔操作、字段限定)

**验收标准**:
- [ ] 10k笔记的搜索响应时间 < 100ms
- [ ] 增量索引更新正常工作
- [ ] 中文分词和搜索正确
- [ ] 搜索结果相关性排序合理
- [ ] 索引文件大小合理 (< 原文件20%)

### 2.4 存储抽象层

**任务描述**: 建立统一的数据访问接口

**存储接口定义**:
```rust
// storage/mod.rs
#[async_trait]
pub trait Storage {
    // 笔记操作
    async fn get_note(&self, id: &str) -> Result<Option<Note>>;
    async fn save_note(&self, note: &Note) -> Result<()>;
    async fn delete_note(&self, id: &str) -> Result<()>;
    async fn list_notes(&self, filter: Option<NoteFilter>) -> Result<Vec<Note>>;
    
    // 标签操作
    async fn get_tags(&self) -> Result<Vec<Tag>>;
    async fn get_notes_by_tag(&self, tag: &str) -> Result<Vec<Note>>;
    
    // 链接操作
    async fn get_links_from(&self, note_id: &str) -> Result<Vec<Link>>;
    async fn get_links_to(&self, note_id: &str) -> Result<Vec<Link>>;
    
    // 搜索操作
    async fn search(&self, query: &SearchQuery) -> Result<SearchResults>;
}
```

**文件系统存储实现**:
```rust
// storage/filesystem.rs
pub struct FileSystemStorage {
    base_path: PathBuf,
    db_pool: SqlitePool,
    indexer: IndexerService,
    file_manager: FileManager,
}

impl Storage for FileSystemStorage {
    // 实现所有storage trait方法
}
```

**缓存机制**:
```rust
// storage/cache.rs
pub struct CacheLayer<T: Storage> {
    inner: T,
    cache: Arc<RwLock<LruCache<String, Note>>>,
}

impl<T: Storage> Storage for CacheLayer<T> {
    // 添加缓存逻辑的storage实现
}
```

**验收标准**:
- [ ] 所有存储操作通过统一接口访问
- [ ] 缓存命中率 > 80%
- [ ] 数据一致性保证
- [ ] 并发访问安全
- [ ] 备份和恢复功能正常

## 集成测试

### 2.5 端到端测试

**测试场景**:
1. **文件监控测试**
   - 创建新文件自动索引
   - 修改文件内容更新索引
   - 删除文件清理索引
   - 批量文件操作

2. **解析功能测试**
   - 各种Markdown语法解析
   - frontmatter提取
   - 链接识别和提取
   - 边界情况处理

3. **搜索性能测试**
   - 大量数据搜索性能
   - 并发搜索测试
   - 内存使用监控
   - 索引更新性能

4. **数据一致性测试**
   - 并发读写测试
   - 数据库事务测试
   - 崩溃恢复测试
   - 备份恢复测试

**性能基准**:
```rust
// 性能测试目标
const PERFORMANCE_TARGETS: &[(&str, Duration)] = &[
    ("file_watch_response", Duration::from_millis(1000)),
    ("markdown_parse_10mb", Duration::from_millis(100)),
    ("search_10k_notes", Duration::from_millis(100)),
    ("index_update", Duration::from_millis(500)),
];
```

## 里程碑和验收

### 第1周里程碑
- 文件监控服务基础实现
- Markdown解析器核心功能
- 数据库Schema设计完成

### 第2周里程碑
- 索引系统基础功能实现
- 存储抽象层接口定义
- 基础的CRUD操作完成

### 第3周里程碑
- 全文搜索功能完成
- 缓存机制实现
- 性能优化和测试

### 最终验收标准
- [ ] 所有单元测试通过
- [ ] 集成测试覆盖核心场景
- [ ] 性能基准达到目标
- [ ] 内存使用稳定
- [ ] 跨平台兼容性验证
- [ ] 错误处理和日志完善

## 技术选型和依赖

### 主要依赖
```toml
# Cargo.toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
notify = "6.0"
pulldown-cmark = "0.9"
yaml-rust = "0.4"
serde = { version = "1.0", features = ["derive"] }
uuid = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

### 技术决策
1. **文件监控**: notify crate (跨平台支持好)
2. **Markdown解析**: pulldown-cmark (性能优秀，标准兼容)
3. **数据库**: SQLite + sqlx (轻量级，性能好)
4. **异步运行时**: tokio (生态成熟)

## 风险管理

### 主要风险
1. **性能风险**: 大量文件监控的性能影响
   - 缓解: 使用防抖机制，批量处理

2. **并发风险**: 数据库并发访问
   - 缓解: 连接池管理，事务控制

3. **兼容性风险**: 跨平台文件系统差异
   - 缓解: 充分测试，平台特定处理

4. **内存风险**: 大文件解析内存占用
   - 缓解: 流式处理，内存限制

## 下一阶段准备

### Phase 3 准备工作
- 确定前端编辑器技术选型
- 设计用户界面交互规范
- 准备UI组件库集成

### 性能基准建立
- 建立持续性能监控
- 设置性能回归警报
- 优化热点路径

---

**创建时间**: 2024-06-30  
**负责人**: 后端开发团队  
**状态**: 规划中  
**依赖**: Phase 1 基础架构完成