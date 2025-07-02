# Hello World Plugin

这是一个 Zeno 插件系统的示例插件，演示了如何开发和集成插件到 Zeno 知识管理平台。

## 功能特性

- **基础消息通信**: 演示插件与主应用之间的双向通信
- **配置管理**: 支持可配置的设置和实时配置更新
- **事件处理**: 监听和响应 Zeno 应用事件
- **命令注册**: 注册自定义命令和快捷键
- **API 调用**: 调用 Zeno 提供的 API 获取数据
- **通知系统**: 向用户显示不同类型的通知

## 安装

1. 将此插件目录复制到 Zeno 的插件目录中
2. 在 Zeno 的插件管理页面中启用此插件

## 配置选项

插件支持以下配置选项：

- `greeting_message`: 自定义问候消息（默认: "Hello from Zeno!"）
- `show_timestamp`: 是否在消息中显示时间戳（默认: true）

## 支持的命令

插件注册了以下命令：

- `hello_world.greet` (Ctrl+Shift+H): 显示问候消息
- `hello_world.count_notes`: 统计工作空间中的笔记数量

## API 端点

插件支持以下消息类型：

- `ping`: 简单的连通性测试，返回 pong
- `hello`: 个性化问候，支持传入姓名
- `get_info`: 获取插件信息和状态
- `count_notes`: 统计笔记数量

## 事件监听

插件监听以下 Zeno 事件：

- `note_created`: 笔记创建时
- `note_updated`: 笔记更新时  
- `note_deleted`: 笔记删除时

## 开发说明

### 插件结构

```
hello-world/
├── plugin.json    # 插件清单文件
├── index.js       # 主要插件代码
└── README.md      # 文档
```

### 插件清单

`plugin.json` 包含插件的元数据、权限要求、配置选项等信息。

### 主要类和方法

- `HelloWorldPlugin`: 主插件类
- `onEnable()`: 插件启用时的初始化逻辑
- `onDisable()`: 插件停用时的清理逻辑
- `handleMessage()`: 处理来自主应用的消息
- `callZenoAPI()`: 调用 Zeno 提供的 API

### 权限系统

插件需要声明所需的权限：

- `file_system`: 文件系统访问权限
- `network`: 网络访问权限
- `ui`: 用户界面操作权限
- `api`: Zeno API 访问权限

## 扩展开发

基于此示例，你可以开发更复杂的插件：

1. **文件处理插件**: 处理特定格式的文件
2. **集成插件**: 与外部服务集成
3. **UI 扩展插件**: 添加自定义界面组件
4. **工作流插件**: 自动化常见任务

## 调试

启用插件后，可以在浏览器开发者工具的控制台中查看插件日志：

```javascript
// 发送测试消息
window.postMessage({
  source: 'zeno_runtime',
  type: 'message',
  message: { type: 'ping', data: {} },
  request_id: 'test_123'
}, '*');
```

## 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件了解详情。

## 贡献

欢迎提交问题和改进建议到 [Zeno GitHub 仓库](https://github.com/foxzool/zeno)。