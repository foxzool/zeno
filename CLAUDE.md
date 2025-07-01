# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Zeno** 是一个基于 Rust 和 Tauri 构建的个人知识管理与发布平台。项目旨在为知识工作者提供从信息收集、整理、创作到发布的完整工作流支持，确保用户对数据的完全控制权。

### 项目状态
- **Phase 1**: ✅ 基础架构搭建 - 已完成
- **Phase 2**: 🚧 数据管理层 - 开发中
- **Repository**: https://github.com/foxzool/zeno

## Architecture

这是一个Rust工作空间项目，包含以下核心组件：

- **zeno-app/**: 主桌面应用程序（Tauri + React）
  - **src-tauri/**: Rust后端核心
    - commands/: Tauri命令模块
    - services/: 核心业务服务（索引器、解析器、发布器）
    - models/: 数据模型
    - db/: 数据库相关
  - **src/**: React前端
    - components/: UI组件
    - stores/: Zustand状态管理
    - hooks/: 自定义Hooks

- **zeno-core/**: 核心库（可独立使用）
  - parser/: Markdown解析
  - indexer/: 索引引擎
  - storage/: 存储抽象
  - publisher/: 发布引擎

- **zeno-cli/**: 命令行工具

## Development Setup

项目 Phase 1 基础架构已完成，包含完整的前后端代码和开发工具链。

### Technology Stack
- **后端核心**: Rust + Tauri
- **前端框架**: React + TypeScript + Vite
- **路由**: React Router (HashRouter)
- **状态管理**: React Hooks (未来可能添加 Zustand)
- **UI 样式**: 原生 CSS (计划迁移到 Tailwind CSS)
- **数据库**: SQLite (索引) + 文件系统 (存储)
- **解析器**: pulldown-cmark
- **包管理**: pnpm

### 开发命令 (使用 Just)

**推荐使用 `just` 命令运行器:**
```bash
# 查看所有可用命令
just

# 核心开发命令
just dev                    # 启动完整开发环境 (Tauri + 前端)
just dev-web                # 只启动前端开发服务器
just check                  # 检查所有代码 (Rust + TypeScript)
just build                  # 构建所有项目
just test                   # 运行所有测试

# CLI 工具
just cli version            # 显示 CLI 版本
just init <path>            # 初始化知识库
just parse <file>           # 解析 Markdown 文件

# 项目管理
just clean                  # 清理构建产物
just deps                   # 安装/更新依赖
just env-check              # 检查开发环境
```

**传统命令 (仍然可用):**
```bash
# Rust 项目
cargo check --workspace     # 检查代码
cargo test --workspace      # 运行测试
cargo run -p zeno-cli        # 运行 CLI 工具

# 前端项目
cd zeno-app
pnpm install                # 安装依赖
pnpm dev                    # 开发模式
pnpm build                  # 构建
pnpm tauri dev              # Tauri 开发模式
pnpm tauri build            # 构建桌面应用
```

## Data Architecture

- 混合存储架构：文件系统作为主存储，SQLite作为索引
- 本地优先：所有数据本地存储，支持Git版本控制
- 支持多平台发布：Zola静态网站、微信公众号、知乎等

## 已实现功能

### ✅ Phase 1 - 基础架构 (已完成)
- **项目工作空间**: 完整的 Rust 工作空间结构
- **桌面应用框架**: Tauri + React 基础架构
- **CLI 工具**: 功能完整的命令行工具
- **核心库**: zeno-core 基础库
- **开发工具链**: Just 脚本 + 完整开发环境

### 🔧 已实现的核心功能
- **应用信息**: 获取版本和应用信息
- **文件操作**: 读取、写入文件内容
- **笔记管理**: 列出 Markdown 文件
- **Markdown 解析**: 解析 Markdown 内容
- **配置管理**: 应用配置读写
- **路由系统**: 完整的前端路由 (首页、笔记、设置)

## 计划功能 (待开发)

### 🚧 Phase 2 - 数据管理层 (开发中)
- 完整的数据库 Schema
- 文件监控和自动索引
- 全文搜索功能
- 标签和分类系统

### 📋 未来功能
- 双向链接和知识图谱
- 可视化 Markdown 编辑器
- 模板系统
- 多平台发布
- 插件系统

## Development Guidelines

### 代码规范
- **数据主权**: 确保所有数据本地存储
- **Rust 最佳实践**: 遵循 Rust 社区标准
- **类型安全**: 充分利用 TypeScript 和 Rust 的类型系统
- **错误处理**: 使用 Result 类型和适当的错误处理

### 交互规范
- **交流语言**: 和我交互使用中文
- **提交消息**: 使用英文和 conventional commit 格式 (feat, fix, docs, etc.)
- **代码注释**: 使用中文注释
- **文档**: 中文文档，保存到 docs/ 目录

### 开发流程
- **分支管理**: 使用 feature 分支进行开发
- **代码检查**: 提交前运行 `just check`
- **测试**: 确保 `just test` 通过
- **文档更新**: 及时更新相关文档

### 项目文件结构
```
zeno/
├── zeno-core/          # 核心库 (parser, storage, indexer)
├── zeno-app/           # Tauri 桌面应用
│   ├── src-tauri/      # Rust 后端 (commands, services, models)
│   └── src/            # React 前端 (components, pages, hooks)
├── zeno-cli/           # 命令行工具
├── docs/               # 项目文档和报告
├── scripts/            # 开发脚本
├── justfile            # 开发命令脚本
└── DEVELOPMENT.md      # 开发指南
```

## 重要提醒

### 🚀 快速开始
- **启动开发**: `just dev` (推荐) 或 `cd zeno-app && pnpm tauri dev`
- **检查代码**: `just check`
- **查看帮助**: `just` 或查看 `DEVELOPMENT.md`

### 📁 当前项目状态
- **Phase 1 已完成**: 基础架构、前后端框架、开发工具链
- **应用可运行**: 桌面应用可以正常启动，包含基本的路由和页面
- **CLI 工具可用**: 支持初始化、解析、列表等基本操作
- **开发环境就绪**: 完整的开发工具链和脚本支持

### ⚠️ 已知问题和限制
- 前端 UI 组件库待完善 (当前使用原生 CSS)
- 数据库持久化功能待实现 (Phase 2)
- 笔记编辑功能待开发
- 测试覆盖率需要提升

### 📊 代码统计 (截至最新提交)
- **总文件数**: 83 个文件
- **Rust 代码**: ~1,764 行
- **TypeScript 代码**: ~303 行
- **提交历史**: 2 个提交 (基础提交 + Phase 1 完成)