# Phase 1: 基础架构搭建计划

## 阶段概述

建立Noto项目的核心基础设施和开发框架，为后续功能开发奠定稳固基础。

**预计时间**: 4-6周  
**优先级**: 最高 (必须完成)

## 目标与交付物

### 主要目标
- 建立完整的项目工作空间结构
- 配置开发环境和工具链
- 实现基础的Tauri应用框架
- 完成核心数据模型设计

### 交付物
- 可运行的最小Tauri应用
- 完整的项目结构和配置
- 基础的前后端通信机制
- 核心数据结构定义

## 详细任务清单

### 1.1 项目初始化

**任务描述**: 创建Rust workspace结构和基础配置

**具体步骤**:
```bash
# 1. 创建项目根目录
mkdir zeno && cd zeno

# 2. 初始化工作空间
cargo init --name zeno-workspace --lib

# 3. 创建子项目
cargo new zeno-core --lib
cargo new zeno-app --lib  
cargo new zeno-cli --bin

# 4. 创建必要目录
mkdir docs assets scripts
```

**配置文件**:
- 根目录 Cargo.toml (workspace配置)
- .gitignore
- README.md
- LICENSE

**验收标准**:
- [ ] 能够运行 `cargo check` 无错误
- [ ] 工作空间结构符合设计规范
- [ ] Git仓库正确初始化

### 1.2 核心数据模型设计

**任务描述**: 定义系统核心数据结构和接口

**主要结构**:
```rust
// models/note.rs
pub struct Note {
    pub id: Uuid,
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub checksum: String,
}

// models/frontmatter.rs
pub struct Frontmatter {
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub status: NoteStatus,
    pub publish: Option<PublishConfig>,
}
```

**实现要求**:
- 完整的序列化/反序列化支持
- 类型安全的数据访问
- 扩展性预留

**验收标准**:
- [ ] 所有核心结构编译通过
- [ ] 序列化测试通过
- [ ] API接口定义完整

### 1.3 数据库Schema设计

**任务描述**: 设计SQLite数据库结构

**核心表结构**:
```sql
-- 笔记索引表
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    word_count INTEGER DEFAULT 0,
    reading_time INTEGER DEFAULT 0,
    frontmatter JSON
);

-- 标签表
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    color TEXT,
    description TEXT
);

-- 笔记-标签关联表
CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

**验收标准**:
- [ ] 数据库Schema文件完成
- [ ] 迁移脚本编写完成
- [ ] 基础CRUD操作测试通过

### 1.4 基础Tauri应用框架

**任务描述**: 建立Tauri桌面应用基础结构

**前端结构**:
```
zeno-app/src/
├── components/
│   ├── layout/
│   └── common/
├── pages/
├── stores/
├── hooks/
├── services/
├── types/
└── utils/
```

**后端结构**:
```
zeno-app/src-tauri/src/
├── commands/
│   ├── mod.rs
│   ├── notes.rs
│   └── config.rs
├── services/
├── models/
├── db/
└── utils/
```

**基础命令**:
```rust
#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| e.to_string())
}
```

**验收标准**:
- [ ] Tauri应用能够正常启动
- [ ] 前端能够调用后端命令
- [ ] 基础UI布局完成
- [ ] 热重载开发环境工作正常

### 1.5 开发环境配置

**任务描述**: 建立完整的开发工具链

**工具配置**:
- VS Code工作空间配置
- Rust工具链和扩展
- Node.js和前端工具链
- 调试配置

**CI/CD配置**:
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy
      - run: cargo fmt --check
```

**验收标准**:
- [ ] 开发环境能够正常运行
- [ ] CI/CD流水线正常工作
- [ ] 代码质量检查通过
- [ ] 文档生成正常

## 里程碑和验收

### 第1周里程碑
- 项目结构创建完成
- 基础开发环境配置完成
- 数据模型初步设计完成

### 第2周里程碑
- Tauri应用框架搭建完成
- 基础前后端通信实现
- 数据库Schema设计完成

### 第3-4周里程碑
- 基础文件操作功能实现
- 简单的笔记显示功能
- 开发工具链完全配置

### 最终验收标准
- [ ] 应用能够启动并显示基础界面
- [ ] 能够读取和显示一个markdown文件
- [ ] 前后端通信正常工作
- [ ] 所有测试通过
- [ ] 代码质量检查通过
- [ ] 文档完整且最新

## 依赖和风险

### 技术依赖
- Rust工具链 (stable版本)
- Node.js 18+ (前端开发)
- SQLite 3.x (数据存储)
- Tauri框架稳定性

### 主要风险
1. **技术选型风险**: Tauri生态成熟度
   - 缓解措施: 选择稳定版本，准备Electron备选方案

2. **开发复杂度风险**: Rust学习曲线
   - 缓解措施: 从简单功能开始，逐步深入

3. **性能风险**: SQLite在大数据量下的表现
   - 缓解措施: 早期建立性能基准测试

### 前置条件
- Rust开发环境正确安装
- 基础的Rust和前端开发知识
- 项目需求和架构设计确认

## 下一阶段准备

### Phase 2准备工作
- 文件监控需求分析
- Markdown解析器技术选型
- 索引系统设计方案

### 知识积累
- Tauri高级特性学习
- SQLite性能优化技巧
- Rust异步编程最佳实践

## 参考资源

### 技术文档
- [Tauri官方文档](https://tauri.app/)
- [SQLite文档](https://sqlite.org/docs.html)
- [Rust异步编程](https://rust-lang.github.io/async-book/)

### 相关项目
- Obsidian架构分析
- Zettlr技术实现
- Typora设计理念

---

**创建时间**: 2024-06-30  
**负责人**: 开发团队  
**状态**: 规划中