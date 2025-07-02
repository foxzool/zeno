# Note Statistics Plugin

一个为 Zeno 知识管理系统提供详细统计分析功能的插件，帮助用户了解自己的写作习惯和笔记模式。

## 功能特性

### 📊 概览统计
- 笔记总数、总字数、总字符数
- 标签数量和链接数量
- 平均每篇笔记字数
- 实时更新的统计数据

### 🏷️ 标签分析
- 最常用标签排行榜
- 标签使用频率分布
- 标签层次结构分析
- 标签使用趋势

### ✍️ 写作模式分析
- 每日、每月、每年写作统计
- 最高产的日子识别
- 写作习惯模式分析
- 平均写作会话长度

### 📝 内容分析
- 最长和最短笔记识别
- 链接最多的笔记
- 最近修改的笔记列表
- 词频统计分析

### 🔗 关系分析
- 孤立笔记检测（无链接的笔记）
- 中心笔记识别（链接最多的笔记）
- 笔记间链接密度计算
- 知识网络聚类分析

## 安装和配置

### 安装步骤
1. 将插件文件复制到 Zeno 插件目录
2. 在插件管理器中启用"Note Statistics"插件
3. 插件将自动开始收集统计数据

### 配置选项

```json
{
  "update_interval": 300,        // 统计更新间隔（秒）
  "include_archived": false,     // 是否包含存档的笔记
  "show_detailed_tags": true,    // 显示详细标签统计
  "export_format": "json"        // 导出格式：json/csv/markdown
}
```

## 使用方法

### 快捷键
- `Ctrl+Shift+S`: 显示统计概览

### 命令面板
- `Note Stats: Show Overview` - 显示总体统计
- `Note Stats: Tag Analysis` - 标签使用分析
- `Note Stats: Writing Patterns` - 写作模式分析
- `Note Stats: Export Statistics` - 导出统计数据
- `Note Stats: Refresh` - 手动刷新统计

### 菜单访问
插件会在主菜单中添加"Statistics"菜单，包含所有统计功能的快速访问。

## 统计数据结构

插件生成的统计数据包含以下结构：

```javascript
{
  overview: {
    totalNotes: 1250,
    totalWords: 125000,
    totalCharacters: 750000,
    totalTags: 180,
    totalLinks: 2500,
    averageWordsPerNote: 100,
    lastUpdated: "2024-01-01T12:00:00Z"
  },
  tags: {
    mostUsed: [
      { tag: "research", count: 45, percentage: "3.6%" },
      // ...
    ],
    distribution: { "research": 45, "project": 32, ... },
    hierarchy: { /* 标签层次结构 */ }
  },
  writing: {
    dailyStats: { "2024-01-01": { notes: 3, words: 450 } },
    monthlyStats: { "2024-01": { notes: 85, words: 12500 } },
    yearlyStats: { "2024": { notes: 1000, words: 100000 } },
    mostProductiveDays: [
      { date: "2024-01-15", notes: 8, words: 1200 }
    ]
  },
  content: {
    longestNote: { title: "...", path: "...", wordCount: 2500 },
    shortestNote: { title: "...", path: "...", wordCount: 5 },
    mostLinkedNote: { title: "...", path: "...", linkCount: 25 },
    recentlyModified: [ /* 最近修改的笔记 */ ],
    wordFrequency: { "knowledge": 145, "system": 132, ... }
  },
  relationships: {
    orphanNotes: [ /* 孤立笔记 */ ],
    hubNotes: [ /* 中心笔记 */ ],
    linkDensity: 0.045,
    clusterAnalysis: { /* 聚类分析结果 */ }
  }
}
```

## 数据导出

插件支持多种格式的数据导出：

### JSON 格式
完整的结构化数据，适合进一步处理和分析。

### CSV 格式
表格化数据，适合在 Excel 或其他表格软件中分析。

### Markdown 格式
人类可读的报告格式，可以直接添加到笔记中。

## 性能优化

### 缓存机制
- 插件使用智能缓存来避免重复计算
- 只有在笔记发生变化时才重新计算相关统计

### 增量更新
- 支持增量统计更新，避免全量重新计算
- 可配置的更新间隔，平衡实时性和性能

### 后台处理
- 统计计算在后台进行，不阻塞用户界面
- 大型笔记库的处理进度提示

## 扩展开发

### 自定义分析器
可以扩展插件添加自定义分析功能：

```javascript
// 添加自定义词频分析器
plugin.addAnalyzer('custom_words', {
  analyze: (notes) => {
    // 自定义分析逻辑
    return customAnalysisResult;
  },
  format: (result) => {
    // 格式化结果
    return formattedResult;
  }
});
```

### API 扩展
插件提供 API 供其他插件使用：

```javascript
// 获取当前统计数据
const stats = await plugin.getStatistics();

// 获取特定时间段的统计
const periodStats = await plugin.getStatistics({
  startDate: '2024-01-01',
  endDate: '2024-01-31'
});

// 订阅统计更新事件
plugin.onStatsUpdate((newStats) => {
  // 处理统计更新
});
```

## 故障排除

### 常见问题

**统计数据不更新**
- 检查插件是否已启用
- 确认更新间隔设置
- 查看插件日志了解错误信息

**内存使用过高**
- 减少更新频率
- 排除大型笔记文件
- 清理插件缓存

**导出失败**
- 检查文件系统权限
- 确认导出目录存在
- 尝试不同的导出格式

### 调试模式
启用调试模式查看详细日志：

```javascript
// 在浏览器控制台中
window.ZenoNoteStatsPlugin.enableDebug(true);
```

## 许可证

MIT License - 参见 LICENSE 文件了解详情。

## 贡献

欢迎提交 Issues 和 Pull Requests 到 [Zeno GitHub 仓库](https://github.com/foxzool/zeno)。

## 更新日志

### v1.0.0
- 初始版本发布
- 基础统计功能
- 标签和写作模式分析
- 多格式数据导出