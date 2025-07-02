# Zeno Plugin Marketplace

Zeno 插件市场是一个集中化的平台，用于发现、安装和管理 Zeno 插件。

## 特性概览

### 🏪 插件发现
- 分类浏览（工具、分析、集成、主题等）
- 关键词搜索和过滤
- 评分和评论系统
- 下载量和受欢迎程度排序

### 📦 包管理
- 一键安装和卸载
- 自动依赖解析
- 版本管理和更新通知
- 批量操作支持

### 🔐 安全验证
- 插件代码审核
- 数字签名验证
- 权限透明展示
- 恶意软件扫描

### 👥 社区功能
- 开发者档案
- 用户评价和反馈
- 插件使用统计
- 社区支持论坛

## 当前可用插件

### 🛠️ 实用工具类

#### Hello World Plugin
- **版本**: 1.0.0
- **作者**: Zeno Team
- **描述**: 插件开发示例，演示基本功能
- **权限**: 只读工作空间、显示通知
- **大小**: 15 KB
- **下载量**: 1,234
- **评分**: ⭐⭐⭐⭐⭐ (4.8/5)

**特性**:
- 基础消息通信演示
- 配置系统使用示例
- 命令注册和快捷键
- 事件监听处理

**安装命令**:
```bash
zeno plugin install hello-world
```

#### Note Statistics
- **版本**: 1.0.0
- **作者**: Zeno Team
- **描述**: 详细的笔记统计和分析工具
- **权限**: 读取笔记、访问标签、创建菜单
- **大小**: 45 KB
- **下载量**: 892
- **评分**: ⭐⭐⭐⭐⭐ (4.9/5)

**特性**:
- 全面的统计概览
- 标签使用分析
- 写作模式识别
- 多格式数据导出

**安装命令**:
```bash
zeno plugin install note-stats
```

### 📝 编辑增强类

#### Advanced Editor
- **版本**: 2.1.0
- **作者**: EditorPro Team
- **描述**: 高级编辑器功能扩展
- **权限**: 修改内容、文件系统访问
- **大小**: 128 KB
- **下载量**: 2,156
- **评分**: ⭐⭐⭐⭐☆ (4.6/5)

**特性**:
- 语法高亮增强
- 代码片段管理
- 自动补全功能
- 实时预览同步

#### Math Renderer
- **版本**: 1.3.2
- **作者**: MathNinja
- **描述**: 数学公式渲染支持
- **权限**: 读取内容、注册渲染器
- **大小**: 85 KB
- **下载量**: 1,567
- **评分**: ⭐⭐⭐⭐⭐ (4.7/5)

**特性**:
- LaTeX 公式支持
- 实时公式预览
- 公式模板库
- 导出为图片

### 🔗 集成工具类

#### GitHub Sync
- **版本**: 1.5.0
- **作者**: GitMaster
- **描述**: 与 GitHub 仓库同步笔记
- **权限**: 网络访问、读写工作空间
- **大小**: 156 KB
- **下载量**: 3,421
- **评分**: ⭐⭐⭐⭐☆ (4.5/5)

**特性**:
- 自动提交和推送
- 冲突解决助手
- 分支管理
- Issue 集成

#### Obsidian Importer
- **版本**: 2.0.1
- **作者**: Migration Tools Inc
- **描述**: 从 Obsidian 导入笔记和设置
- **权限**: 读取外部文件、写入工作空间
- **大小**: 67 KB
- **下载量**: 1,890
- **评分**: ⭐⭐⭐⭐⭐ (4.8/5)

**特性**:
- 批量导入笔记
- 链接关系保持
- 标签映射
- 配置迁移

### 🎨 主题美化类

#### Dark Mode Pro
- **版本**: 1.2.0
- **作者**: ThemeCreator
- **描述**: 专业深色主题包
- **权限**: 修改界面样式
- **大小**: 234 KB
- **下载量**: 4,567
- **评分**: ⭐⭐⭐⭐⭐ (4.9/5)

**特性**:
- 多种深色主题
- 自定义颜色方案
- 护眼模式
- 定时切换

#### Minimal Theme
- **版本**: 3.0.0
- **作者**: MinimalDesign
- **描述**: 简约清爽的界面主题
- **权限**: 修改界面样式
- **大小**: 89 KB
- **下载量**: 2,345
- **评分**: ⭐⭐⭐⭐☆ (4.4/5)

**特性**:
- 极简设计理念
- 高对比度选项
- 自适应布局
- 专注模式

## 插件开发指南

### 开发环境设置

1. **安装开发工具**
```bash
npm install -g @zeno/plugin-cli
```

2. **创建插件项目**
```bash
zeno-plugin create my-awesome-plugin
cd my-awesome-plugin
```

3. **开发和测试**
```bash
npm run dev    # 开发模式
npm run test   # 运行测试
npm run build  # 构建插件
```

### 插件架构

```
my-plugin/
├── plugin.json       # 插件清单
├── index.js          # 主入口文件
├── package.json      # Node.js 包配置
├── README.md         # 文档
├── assets/           # 资源文件
│   ├── icons/
│   └── styles/
├── src/              # 源代码
│   ├── main.js
│   ├── api.js
│   └── utils.js
└── tests/            # 测试文件
    └── main.test.js
```

### 核心 API

#### 插件生命周期
```javascript
class MyPlugin {
  async onEnable(config) {
    // 插件启用时调用
  }
  
  async onDisable() {
    // 插件停用时调用
  }
  
  async onConfigUpdate(newConfig) {
    // 配置更新时调用
  }
}
```

#### Zeno API 调用
```javascript
// 获取笔记列表
const notes = await this.callZenoAPI('get_all_notes');

// 创建新笔记
await this.callZenoAPI('create_note', {
  title: 'New Note',
  content: 'Note content...'
});

// 注册命令
await this.callZenoAPI('register_command', {
  id: 'my_plugin.command',
  name: 'My Command',
  description: 'Does something useful'
});
```

#### 事件处理
```javascript
// 监听笔记事件
await this.callZenoAPI('subscribe_event', {
  event_type: 'note_created'
});

// 处理事件
async handleEvent(event) {
  if (event.type === 'note_created') {
    // 处理笔记创建事件
  }
}
```

### 权限系统

插件需要声明所需权限：

```json
{
  "permissions": {
    "file_system": {
      "read_workspace": true,
      "write_workspace": false,
      "allowed_paths": ["assets/", "exports/"],
      "denied_paths": ["config/", "system/"]
    },
    "network": {
      "http_request": true,
      "allowed_domains": ["api.example.com"],
      "denied_domains": ["malicious.com"]
    },
    "ui": {
      "show_notifications": true,
      "create_menus": true,
      "register_commands": true
    },
    "api": {
      "access_notes": true,
      "access_tags": true,
      "access_links": false,
      "modify_content": false
    }
  }
}
```

### 发布到市场

1. **准备发布**
```bash
zeno-plugin validate   # 验证插件
zeno-plugin package    # 打包插件
```

2. **提交审核**
```bash
zeno-plugin submit --file my-plugin.zpkg
```

3. **审核流程**
- 代码安全审查
- 功能测试验证
- 文档完整性检查
- 社区反馈收集

## 市场统计

### 总体数据
- **总插件数**: 127
- **总下载量**: 45,678
- **活跃开发者**: 89
- **平均评分**: 4.6/5

### 分类分布
- 实用工具: 42 个插件
- 编辑增强: 28 个插件
- 集成工具: 31 个插件
- 主题美化: 26 个插件

### 热门标签
1. #productivity (45)
2. #editor (32)
3. #integration (28)
4. #analytics (24)
5. #theme (26)
6. #automation (19)
7. #export (17)
8. #sync (15)

## 安全政策

### 代码审核标准
- **静态代码分析**: 检测潜在安全漏洞
- **依赖项扫描**: 检查第三方库安全性
- **权限审计**: 确保权限使用合理
- **沙箱测试**: 在隔离环境中测试

### 举报机制
如发现恶意插件或安全问题：
- 邮箱: security@zeno.dev
- 在线举报: https://marketplace.zeno.dev/report
- GitHub Issues: https://github.com/foxzool/zeno/issues

### 更新策略
- **安全更新**: 立即推送
- **功能更新**: 用户确认后安装
- **重大版本**: 手动升级确认

## 支持和帮助

### 文档资源
- [插件开发指南](https://docs.zeno.dev/plugins/)
- [API 参考文档](https://docs.zeno.dev/api/)
- [最佳实践](https://docs.zeno.dev/best-practices/)
- [常见问题](https://docs.zeno.dev/faq/)

### 社区支持
- [开发者论坛](https://forum.zeno.dev/)
- [Discord 社区](https://discord.gg/zeno)
- [GitHub 讨论](https://github.com/foxzool/zeno/discussions)

### 技术支持
- 邮箱: support@zeno.dev
- 在线客服: https://zeno.dev/support
- 工单系统: https://support.zeno.dev

---

*最后更新: 2024年7月2日*
*市场版本: v1.0.0*