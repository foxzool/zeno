#!/bin/bash

# Zeno 项目 .gitignore 验证脚本

echo "🔍 检查 .gitignore 配置..."
echo ""

GITIGNORE_FILE=".gitignore"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_ROOT"

if [ ! -f "$GITIGNORE_FILE" ]; then
    echo "❌ .gitignore 文件不存在！"
    exit 1
fi

echo "📁 项目根目录: $PROJECT_ROOT"
echo "📄 检查 .gitignore 文件: $GITIGNORE_FILE"
echo ""

# 检查关键忽略规则是否存在
check_rule() {
    local rule="$1"
    local description="$2"
    
    if grep -q "^$rule" "$GITIGNORE_FILE" || grep -q "^# $rule" "$GITIGNORE_FILE" || grep -q "$rule" "$GITIGNORE_FILE"; then
        echo "✅ $description"
    else
        echo "❌ 缺少规则: $rule ($description)"
    fi
}

echo "🔍 检查 Rust 相关规则:"
check_rule "target/" "Cargo 构建输出"
check_rule "\*\*/\*.rs.bk" "Rust 备份文件"
check_rule "Cargo.lock" "Cargo 锁文件"

echo ""
echo "🔍 检查 Node.js/前端规则:"
check_rule "node_modules/" "Node.js 依赖"
check_rule "dist/" "构建输出"
check_rule "\*.tsbuildinfo" "TypeScript 构建缓存"
check_rule "\*.log" "日志文件"

echo ""
echo "🔍 检查 Tauri 特定规则:"
check_rule "src-tauri/target/" "Tauri 构建目录"
check_rule "\*.app/" "macOS 应用包"
check_rule "\*.msi" "Windows 安装包"
check_rule "\*.dmg" "macOS 磁盘镜像"

echo ""
echo "🔍 检查数据库和用户数据规则:"
check_rule "\*.db" "数据库文件"
check_rule "\*.sqlite" "SQLite 数据库"
check_rule ".zeno/" "用户数据目录"

echo ""
echo "🔍 检查开发工具规则:"
check_rule ".DS_Store" "macOS 系统文件"
check_rule ".env" "环境变量文件"
check_rule "\*.tmp" "临时文件"
check_rule "coverage/" "测试覆盖率"

echo ""
echo "🔍 检查当前工作区状态:"

# 检查是否有应该被忽略但当前被跟踪的文件
echo "📋 检查被跟踪的文件中是否有应该忽略的:"

# 一些常见的应该被忽略的文件
should_be_ignored=(
    "target/"
    "node_modules/"
    "dist/"
    ".DS_Store"
    "*.log"
    "Cargo.lock"
    ".env"
    "coverage/"
)

for pattern in "${should_be_ignored[@]}"; do
    # 使用 git ls-files 检查是否有匹配的已跟踪文件
    if git ls-files | grep -q "$pattern" 2>/dev/null; then
        echo "⚠️  发现被跟踪的文件匹配模式: $pattern"
    fi
done

echo ""
echo "🔍 检查 git 状态:"

# 显示当前 git 状态（简短版本）
if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    untracked_count=$(git ls-files --others --exclude-standard | wc -l | tr -d ' ')
    tracked_count=$(git ls-files | wc -l | tr -d ' ')
    
    echo "📊 统计信息:"
    echo "   - 已跟踪文件数: $tracked_count"
    echo "   - 未跟踪文件数: $untracked_count"
    
    if [ "$untracked_count" -gt 0 ]; then
        echo ""
        echo "📁 当前未跟踪的文件:"
        git ls-files --others --exclude-standard | head -10
        if [ "$untracked_count" -gt 10 ]; then
            echo "   ... 还有 $((untracked_count - 10)) 个文件"
        fi
    fi
else
    echo "ℹ️  不是 Git 仓库或 Git 不可用"
fi

echo ""
echo "✅ .gitignore 检查完成！"
echo ""
echo "💡 提示:"
echo "   - 使用 'git status' 查看当前工作区状态"
echo "   - 使用 'git check-ignore <file>' 测试特定文件是否被忽略"
echo "   - 使用 'git ls-files --others --exclude-standard' 查看未跟踪文件"