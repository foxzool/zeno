# Zeno 开发指南

本文档介绍如何使用 Zeno 项目的开发工具和脚本。

## 快速开始

### 安装 Just

首先安装 [Just](https://github.com/casey/just) 命令运行器：

```bash
# macOS
brew install just

# 或使用 cargo
cargo install just
```

### 查看所有可用命令

```bash
just
```

这将显示所有可用的开发命令。

## 核心开发工作流程

### 1. 开发环境启动

```bash
# 启动完整开发环境 (推荐)
just dev

# 或只启动前端开发服务器
just dev-web
```

### 2. 代码检查和修复

```bash
# 检查所有代码
just check

# 自动修复代码格式和简单问题
just fix

# 手动格式化代码
just fmt

# 运行代码检查工具
just lint
```

### 3. 构建和测试

```bash
# 构建所有项目
just build

# 只构建桌面应用
just build-app

# 运行所有测试
just test

# 只运行 Rust 测试
just test-rust
```

## 项目管理

### 依赖管理

```bash
# 安装/更新依赖
just deps

# 更新依赖版本
just update

# 清理构建产物
just clean
```

### 文档生成

```bash
# 生成文档
just doc
```

## CLI 工具使用

### 基本命令

```bash
# 显示 CLI 版本
just cli version

# 初始化新的知识库
just init ~/Documents/my-notes

# 解析 Markdown 文件
just parse ~/Documents/my-notes/note.md

# 列出目录中的 Markdown 文件
just list ~/Documents/my-notes
```

### 直接使用 zeno-cli

```bash
# 运行任意 CLI 命令
just cli <command> [args...]

# 示例
just cli init --path ~/test-notes
just cli parse --file ~/test.md --format json
```

## 实用工具

### 开发信息

```bash
# 检查开发环境
just env-check

# 查看项目统计
just stats

# 显示版本信息
just version

# 查看项目结构
just tree
```

### 监控和调试

```bash
# 监控文件变化并自动测试
just watch-test

# 监控文件变化并自动检查
just watch-check

# 生成代码覆盖率报告
just coverage
```

### 安全和审计

```bash
# 安全审计
just audit

# 重置开发数据库
just reset-db

# 备份用户数据
just backup
```

## 发布流程

### 准备发布

```bash
# 完整的发布前检查
just release-check
```

这个命令会执行：
- 代码检查 (`just check`)
- 运行测试 (`just test`)
- 代码质量检查 (`just lint`)
- 安全审计 (`just audit`)
- 构建项目 (`just build`)

### 创建发布版本

```bash
# 创建发布版本
just release 1.0.0
```

## 项目结构

```
zeno/
├── zeno-core/          # 核心库
├── zeno-app/           # Tauri 桌面应用
├── zeno-cli/           # 命令行工具
├── docs/               # 项目文档
├── justfile            # 开发脚本
└── DEVELOPMENT.md      # 本文档
```

## 技术栈

- **后端**: Rust + Tauri
- **前端**: React + TypeScript + Vite
- **数据库**: SQLite
- **构建工具**: Cargo + pnpm + Vite
- **开发工具**: Just + Cargo Watch

## 故障排除

### 常见问题

1. **编译错误**
   ```bash
   just clean
   just deps
   just check
   ```

2. **前端依赖问题**
   ```bash
   cd zeno-app
   rm -rf node_modules pnpm-lock.yaml
   pnpm install
   ```

3. **数据库问题**
   ```bash
   just reset-db
   ```

### 环境检查

如果遇到问题，首先运行环境检查：

```bash
just env-check
```

确保所有必需的工具都已正确安装。

## 贡献指南

1. 创建功能分支
2. 使用 `just check` 检查代码
3. 使用 `just test` 运行测试
4. 使用 `just fix` 修复格式问题
5. 提交变更
6. 运行 `just release-check` 确保一切正常

## 更多帮助

- 查看所有可用命令：`just`
- 查看特定命令帮助：`just --help <command>`
- 项目文档：`docs/` 目录