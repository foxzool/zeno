# Zeno 项目开发助手 - Just 脚本
# 使用 `just` 命令查看所有可用任务
# 使用 `just <任务名>` 执行特定任务

# 默认任务 - 显示帮助信息
default:
    @echo "🦀 Zeno 知识管理平台开发助手"
    @echo ""
    @echo "核心开发命令:"
    @echo "  just dev        - 启动开发环境 (Tauri + 前端热重载)"
    @echo "  just dev-web    - 只启动前端开发服务器"
    @echo "  just check      - 检查所有代码 (Rust + TypeScript)"
    @echo "  just fix        - 自动修复代码格式"
    @echo ""
    @echo "构建和测试:"
    @echo "  just build      - 构建所有项目"
    @echo "  just build-app  - 构建桌面应用"
    @echo "  just test       - 运行所有测试"
    @echo "  just test-rust  - 只运行 Rust 测试"
    @echo ""
    @echo "项目管理:"
    @echo "  just clean      - 清理构建产物"
    @echo "  just deps       - 安装/更新依赖"
    @echo "  just update     - 更新依赖版本"
    @echo "  just doc        - 生成文档"
    @echo ""
    @echo "工具命令:"
    @echo "  just cli <cmd>  - 运行 zeno-cli 命令"
    @echo "  just fmt        - 格式化代码"
    @echo "  just lint       - 代码检查"
    @echo "  just version    - 显示版本信息"

# ============================================================================
# 核心开发命令
# ============================================================================

# 启动完整开发环境 (Tauri + 前端热重载)
dev:
    @echo "🚀 启动 Zeno 开发环境..."
    cd zeno-app && pnpm tauri dev

# 只启动前端开发服务器
dev-web:
    @echo "🌐 启动前端开发服务器..."
    cd zeno-app && pnpm dev

# 检查所有代码
check:
    @echo "🔍 检查 Rust 代码..."
    cargo check --workspace --all-targets --all-features
    @echo "🔍 检查 TypeScript 代码..."
    cd zeno-app && pnpm type-check

# 自动修复代码格式和问题
fix:
    @echo "🔧 修复 Rust 代码..."
    cargo fmt --all
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty
    @echo "🔧 修复 TypeScript 代码..."
    cd zeno-app && pnpm lint --fix

# ============================================================================
# 构建和测试
# ============================================================================

# 构建所有项目
build:
    @echo "🏗️  构建所有项目..."
    cargo build --workspace --release
    cd zeno-app && pnpm build

# 构建桌面应用
build-app:
    @echo "🏗️  构建桌面应用..."
    cd zeno-app && pnpm tauri build

# 运行所有测试
test:
    @echo "🧪 运行 Rust 测试..."
    cargo test --workspace --all-features
    @echo "🧪 运行前端测试..."
    cd zeno-app && pnpm test

# 只运行 Rust 测试
test-rust:
    @echo "🧪 运行 Rust 测试..."
    cargo test --workspace --all-features

# 运行基准测试
bench:
    @echo "⚡ 运行基准测试..."
    cargo bench --workspace

# ============================================================================
# 代码质量
# ============================================================================

# 格式化代码
fmt:
    @echo "✨ 格式化 Rust 代码..."
    cargo fmt --all --check
    @echo "✨ 格式化 TypeScript 代码..."
    cd zeno-app && pnpm format

# 代码检查 (lint)
lint:
    @echo "🔎 Rust 代码检查..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    @echo "🔎 TypeScript 代码检查..."
    cd zeno-app && pnpm lint

# 安全审计
audit:
    @echo "🔒 安全审计..."
    cargo audit
    cd zeno-app && pnpm audit

# ============================================================================
# 项目管理
# ============================================================================

# 清理构建产物
clean:
    @echo "🧹 清理构建产物..."
    cargo clean
    cd zeno-app && rm -rf dist target
    rm -rf target

# 安装/更新依赖
deps:
    @echo "📦 安装 Rust 依赖..."
    cargo fetch
    @echo "📦 安装前端依赖..."
    cd zeno-app && pnpm install

# 更新依赖版本
update:
    @echo "⬆️  更新 Rust 依赖..."
    cargo update
    @echo "⬆️  更新前端依赖..."
    cd zeno-app && pnpm update

# 生成文档
doc:
    @echo "📚 生成 Rust 文档..."
    cargo doc --workspace --all-features --no-deps --open
    @echo "📚 生成 TypeScript 文档..."
    cd zeno-app && pnpm build:docs

# ============================================================================
# 工具命令
# ============================================================================

# 运行 zeno-cli 命令
cli *ARGS:
    @echo "🖥️  运行 zeno-cli {{ARGS}}"
    cargo run -p zeno-cli -- {{ARGS}}

# 初始化新的知识库
init PATH:
    @echo "📁 初始化知识库: {{PATH}}"
    cargo run -p zeno-cli -- init --path {{PATH}}

# 解析 Markdown 文件
parse FILE:
    @echo "📄 解析文件: {{FILE}}"
    cargo run -p zeno-cli -- parse --file {{FILE}}

# 列出 Markdown 文件
list DIR:
    @echo "📋 列出目录: {{DIR}}"
    cargo run -p zeno-cli -- list --dir {{DIR}}

# 显示版本信息
version:
    @echo "📋 项目版本信息:"
    @echo "Workspace:"
    cargo version
    @echo "zeno-core:"
    cargo run -p zeno-cli -- version
    @echo "zeno-app:"
    cd zeno-app && pnpm --version

# ============================================================================
# 开发工具
# ============================================================================

# 监控文件变化并自动测试
watch-test:
    @echo "👀 监控文件变化并运行测试..."
    cargo watch -x 'test --workspace'

# 监控文件变化并自动检查
watch-check:
    @echo "👀 监控文件变化并检查代码..."
    cargo watch -x 'check --workspace'

# 生成代码覆盖率报告
coverage:
    @echo "📊 生成代码覆盖率报告..."
    cargo tarpaulin --workspace --out Html --output-dir coverage

# ============================================================================
# 发布和部署
# ============================================================================

# 准备发布 (检查、测试、构建)
release-check:
    @echo "🚀 准备发布检查..."
    just check
    just test
    just lint
    just audit
    just build

# 创建发布版本
release VERSION:
    @echo "🏷️  创建发布版本: {{VERSION}}"
    git tag -a v{{VERSION}} -m "Release version {{VERSION}}"
    just build-app

# ============================================================================
# 数据库和存储
# ============================================================================

# 重置开发数据库
reset-db:
    @echo "🗄️  重置开发数据库..."
    rm -f ~/.zeno/app.db
    @echo "数据库已重置"

# 备份用户数据
backup:
    @echo "💾 备份用户数据..."
    cp -r ~/.zeno ~/.zeno.backup.$(date +%Y%m%d_%H%M%S)
    @echo "备份完成"

# ============================================================================
# Docker 支持 (预留)
# ============================================================================

# 构建 Docker 镜像
docker-build:
    @echo "🐳 构建 Docker 镜像..."
    docker build -t zeno:latest .

# 运行 Docker 容器
docker-run:
    @echo "🐳 运行 Docker 容器..."
    docker run -it --rm -p 8080:8080 zeno:latest

# ============================================================================
# 帮助和调试
# ============================================================================

# 显示项目结构
tree:
    @echo "🌳 项目结构:"
    tree -I 'target|node_modules|dist|.git' -L 3

# 显示项目统计信息
stats:
    @echo "📊 项目统计:"
    @echo "Rust 代码行数:"
    find . -name "*.rs" -not -path "./target/*" -not -path "./node_modules/*" | xargs wc -l | tail -1
    @echo "TypeScript 代码行数:"
    find zeno-app/src -name "*.ts" -o -name "*.tsx" | xargs wc -l | tail -1

# 检查开发环境
env-check:
    @echo "🔧 检查开发环境..."
    @echo "Rust版本:"
    rustc --version
    @echo "Cargo版本:"
    cargo --version
    @echo "Node.js版本:"
    node --version
    @echo "pnpm版本:"
    pnpm --version
    @echo "Tauri CLI版本:"
    cd zeno-app && pnpm tauri --version