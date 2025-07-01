# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Noto是一个基于Rust和Tauri构建的个人知识管理与发布平台。项目旨在为知识工作者提供从信息收集、整理、创作到发布的完整工作流支持，确保用户对数据的完全控制权。

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

由于这是一个新项目，目前只有架构设计文档，还没有实际代码。根据项目方案：

### Technology Stack
- 后端核心: Rust
- 桌面框架: Tauri
- 前端框架: React + TypeScript
- 状态管理: Zustand
- UI组件: Radix UI + Tailwind CSS
- 编辑器: Monaco Editor / Milkdown
- 数据库: SQLite (索引) + 文件系统 (存储)
- 解析器: markdown-rs
- ORM: Diesel

### Expected Commands (when implemented)
```bash
# Development
cargo run --bin zeno-app    # 运行桌面应用
cargo run --bin zeno-cli    # 运行CLI工具
cargo test                  # 运行测试
cargo fmt                   # 格式化代码
cargo clippy                # 代码检查

# For Tauri app
cd zeno-app
npm install                 # 安装前端依赖
npm run dev                 # 开发模式
npm run build               # 构建
npm run tauri dev           # Tauri开发模式
npm run tauri build         # 构建桌面应用
```

## Data Architecture

- 混合存储架构：文件系统作为主存储，SQLite作为索引
- 本地优先：所有数据本地存储，支持Git版本控制
- 支持多平台发布：Zola静态网站、微信公众号、知乎等

## Key Features (Planned)
- 双向链接和知识图谱
- Markdown编辑器
- 全文搜索
- 模板系统
- 多平台发布
- 插件系统

## Development Guidelines

- 确保数据主权：所有数据本地存储
- 遵循Rust最佳实践
- 和我交互使用中文
- 提交消息使用中文和conventional commit格式
- 代码注释和文档使用中文
- 报告保存到docs目录下, 注意报告的生成时间是当前时间