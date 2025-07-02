# Zeno 插件开发指南

这份指南将带你从零开始开发一个 Zeno 插件，涵盖基础概念、开发流程和最佳实践。

## 📚 基础概念

### 什么是 Zeno 插件？

Zeno 插件是扩展 Zeno 知识管理系统功能的独立模块。插件可以：
- 添加新的编辑器功能
- 集成外部服务
- 提供数据分析和可视化
- 自动化工作流程
- 自定义界面主题

### 插件架构

```
Zeno 主应用
├── 插件运行时环境
│   ├── JavaScript 引擎 (V8)
│   ├── WASM 运行时
│   └── 本地进程管理
├── 插件 API 层
│   ├── 笔记操作 API
│   ├── 标签管理 API
│   ├── 链接分析 API
│   └── 界面扩展 API
└── 安全沙箱
    ├── 权限控制
    ├── 资源限制
    └── 网络隔离
```

## 🚀 快速开始

### 环境准备

1. **安装 Zeno 开发版**
```bash
# 从源码构建
git clone https://github.com/foxzool/zeno.git
cd zeno
cargo build --release

# 或下载预编译版本
curl -L https://github.com/foxzool/zeno/releases/latest/download/zeno-dev.tar.gz | tar xz
```

2. **安装插件开发工具**
```bash
npm install -g @zeno/plugin-cli
```

3. **验证安装**
```bash
zeno --version
zeno-plugin --version
```

### 创建第一个插件

1. **创建插件项目**
```bash
zeno-plugin create my-first-plugin
cd my-first-plugin
```

这会创建以下结构：
```
my-first-plugin/
├── plugin.json      # 插件配置文件
├── index.js         # 主入口文件
├── package.json     # Node.js 包配置
├── README.md        # 说明文档
└── .gitignore       # Git 忽略文件
```

2. **编辑插件配置**

编辑 `plugin.json`：
```json
{
  "id": "my-first-plugin",
  "name": "My First Plugin",
  "version": "1.0.0",
  "description": "My first Zeno plugin",
  "author": "Your Name",
  "license": "MIT",
  "main": "index.js",
  "category": "utility",
  "keywords": ["example", "first", "demo"],
  "permissions": {
    "file_system": {
      "read_workspace": true,
      "write_workspace": false
    },
    "ui": {
      "show_notifications": true,
      "register_commands": true
    },
    "api": {
      "access_notes": true
    }
  },
  "config": {
    "auto_enable": false,
    "settings": {
      "message": {
        "type": "string",
        "default": "Hello from my plugin!",
        "description": "Greeting message"
      }
    }
  },
  "zeno_version": ">=0.1.0"
}
```

3. **编写插件代码**

编辑 `index.js`：
```javascript
class MyFirstPlugin {
  constructor() {
    this.isEnabled = false;
    this.config = null;
  }

  // 插件启用时调用
  async onEnable(config) {
    console.log('[MyFirstPlugin] Enabled');
    this.isEnabled = true;
    this.config = config;
    
    // 注册命令
    await this.registerCommands();
    
    // 显示欢迎消息
    const message = config.settings?.message || 'Hello from my plugin!';
    await this.showNotification('success', message);
  }

  // 插件停用时调用
  async onDisable() {
    console.log('[MyFirstPlugin] Disabled');
    this.isEnabled = false;
    await this.showNotification('info', 'Plugin disabled');
  }

  // 注册命令
  async registerCommands() {
    await this.callZenoAPI('register_command', {
      id: 'my_first_plugin.greet',
      name: 'My First Plugin: Greet',
      description: 'Show a greeting message',
      shortcut: 'Ctrl+Shift+G'
    });
  }

  // 处理命令
  async handleCommand(commandData) {
    if (commandData.command_id === 'my_first_plugin.greet') {
      const message = this.config.settings?.message || 'Hello!';
      await this.showNotification('info', `Greeting: ${message}`);
    }
  }

  // 显示通知
  async showNotification(type, message) {
    const notification = {
      type: 'notification',
      level: type,
      message: message,
      plugin_id: 'my-first-plugin'
    };
    
    await this.sendToZeno(notification);
  }

  // 调用 Zeno API
  async callZenoAPI(endpoint, data = {}) {
    const message = {
      type: 'api_call',
      endpoint: endpoint,
      data: data,
      request_id: this.generateRequestId()
    };
    
    await this.sendToZeno(message);
    return { success: true };
  }

  // 发送消息到 Zeno
  async sendToZeno(message) {
    console.log('[MyFirstPlugin] Sending to Zeno:', message);
    
    if (typeof window !== 'undefined' && window.parent !== window) {
      window.parent.postMessage({
        source: 'zeno_plugin',
        plugin_id: 'my-first-plugin',
        ...message
      }, '*');
    }
  }

  // 生成请求 ID
  generateRequestId() {
    return 'mfp_' + Math.random().toString(36).substr(2, 9);
  }
}

// 创建插件实例
const plugin = new MyFirstPlugin();

// 插件入口点
async function main() {
  if (typeof window !== 'undefined') {
    // 监听来自 Zeno 的消息
    window.addEventListener('message', async (event) => {
      if (event.data.source === 'zeno_runtime') {
        switch (event.data.type) {
          case 'enable':
            await plugin.onEnable(event.data.config);
            break;
          case 'disable':
            await plugin.onDisable();
            break;
          case 'command':
            await plugin.handleCommand(event.data);
            break;
        }
      }
    });
    
    // 通知 Zeno 插件已准备就绪
    window.parent.postMessage({
      source: 'zeno_plugin',
      plugin_id: 'my-first-plugin',
      type: 'ready'
    }, '*');
  }
}

// 启动插件
main();
```

4. **测试插件**
```bash
# 开发模式运行
zeno-plugin dev

# 在另一个终端启动 Zeno
zeno --dev-plugins ./my-first-plugin
```

## 🔧 开发进阶

### 插件配置系统

插件可以定义用户可配置的设置：

```json
{
  "config": {
    "settings": {
      "theme_color": {
        "type": "string",
        "default": "#007acc",
        "description": "Theme color",
        "pattern": "^#[0-9a-fA-F]{6}$"
      },
      "auto_save": {
        "type": "boolean",
        "default": true,
        "description": "Enable auto save"
      },
      "save_interval": {
        "type": "number",
        "default": 30,
        "min": 5,
        "max": 300,
        "description": "Auto save interval in seconds"
      },
      "export_format": {
        "type": "string",
        "default": "markdown",
        "options": ["markdown", "html", "pdf"],
        "description": "Default export format"
      }
    }
  }
}
```

### 权限系统

详细的权限控制确保插件安全：

```json
{
  "permissions": {
    "file_system": {
      "read_workspace": true,           // 读取工作空间
      "write_workspace": false,         // 写入工作空间
      "allowed_paths": [                // 允许访问的路径
        "exports/",
        "temp/"
      ],
      "denied_paths": [                 // 禁止访问的路径
        "config/",
        "system/"
      ]
    },
    "network": {
      "http_request": true,             // HTTP 请求权限
      "allowed_domains": [              // 允许的域名
        "api.example.com",
        "cdn.example.org"
      ],
      "denied_domains": [               // 禁止的域名
        "malicious.com"
      ]
    },
    "ui": {
      "show_notifications": true,       // 显示通知
      "create_menus": true,             // 创建菜单
      "register_commands": true,        // 注册命令
      "modify_interface": false         // 修改界面
    },
    "api": {
      "access_notes": true,             // 访问笔记
      "access_tags": true,              // 访问标签
      "access_links": true,             // 访问链接
      "modify_content": false,          // 修改内容
      "access_settings": false          // 访问设置
    }
  }
}
```

### API 使用示例

#### 笔记操作
```javascript
// 获取所有笔记
const notes = await this.callZenoAPI('get_all_notes');

// 获取特定笔记
const note = await this.callZenoAPI('get_note', { id: 'note-id' });

// 创建新笔记
await this.callZenoAPI('create_note', {
  title: 'New Note',
  content: '# Hello World\n\nThis is a new note.',
  tags: ['example', 'demo']
});

// 更新笔记
await this.callZenoAPI('update_note', {
  id: 'note-id',
  title: 'Updated Title',
  content: 'Updated content...'
});

// 删除笔记
await this.callZenoAPI('delete_note', { id: 'note-id' });
```

#### 标签操作
```javascript
// 获取所有标签
const tags = await this.callZenoAPI('get_all_tags');

// 获取标签统计
const tagStats = await this.callZenoAPI('get_tag_stats');

// 创建新标签
await this.callZenoAPI('create_tag', {
  name: 'new-tag',
  color: '#ff6b6b'
});
```

#### 链接分析
```javascript
// 获取所有链接
const links = await this.callZenoAPI('get_all_links');

// 获取笔记的反向链接
const backlinks = await this.callZenoAPI('get_backlinks', {
  note_id: 'note-id'
});

// 分析链接关系
const graph = await this.callZenoAPI('analyze_link_graph');
```

#### 界面扩展
```javascript
// 注册命令
await this.callZenoAPI('register_command', {
  id: 'plugin.command',
  name: 'Plugin Command',
  description: 'Execute plugin command',
  shortcut: 'Ctrl+Shift+P'
});

// 创建菜单
await this.callZenoAPI('register_menu', {
  id: 'plugin_menu',
  label: 'Plugin Menu',
  items: [
    { id: 'action1', label: 'Action 1', command: 'plugin.action1' },
    { id: 'separator', type: 'separator' },
    { id: 'action2', label: 'Action 2', command: 'plugin.action2' }
  ]
});

// 显示通知
await this.callZenoAPI('show_notification', {
  type: 'success',
  message: 'Operation completed',
  timeout: 3000
});
```

### 事件处理

监听和响应 Zeno 事件：

```javascript
class MyPlugin {
  async onEnable(config) {
    // 订阅事件
    await this.callZenoAPI('subscribe_event', {
      event_type: 'note_created'
    });
    
    await this.callZenoAPI('subscribe_event', {
      event_type: 'note_updated'
    });
  }

  async handleEvent(event) {
    switch (event.type) {
      case 'note_created':
        console.log('New note created:', event.data.title);
        await this.onNoteCreated(event.data);
        break;
        
      case 'note_updated':
        console.log('Note updated:', event.data.title);
        await this.onNoteUpdated(event.data);
        break;
        
      case 'workspace_changed':
        console.log('Workspace changed');
        await this.onWorkspaceChanged();
        break;
    }
  }

  async onNoteCreated(noteData) {
    // 处理新笔记创建
    if (noteData.tags.includes('important')) {
      await this.callZenoAPI('show_notification', {
        type: 'info',
        message: `Important note created: ${noteData.title}`
      });
    }
  }

  async onNoteUpdated(noteData) {
    // 处理笔记更新
    const wordCount = this.countWords(noteData.content);
    if (wordCount > 1000) {
      await this.callZenoAPI('add_tag', {
        note_id: noteData.id,
        tag: 'long-form'
      });
    }
  }
}
```

## 🎨 UI 扩展

### 创建自定义组件

```javascript
class UIPlugin {
  async onEnable(config) {
    // 注册自定义组件
    await this.callZenoAPI('register_component', {
      id: 'my_widget',
      name: 'My Widget',
      description: 'A custom widget',
      html: this.getWidgetHTML(),
      css: this.getWidgetCSS(),
      js: this.getWidgetJS()
    });
  }

  getWidgetHTML() {
    return `
      <div id="my-widget" class="zeno-widget">
        <h3>My Custom Widget</h3>
        <div class="widget-content">
          <p>Widget content goes here</p>
          <button onclick="myWidgetAction()">Click Me</button>
        </div>
      </div>
    `;
  }

  getWidgetCSS() {
    return `
      .zeno-widget {
        background: var(--bg-secondary);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 16px;
        margin: 8px 0;
      }
      
      .zeno-widget h3 {
        margin: 0 0 12px 0;
        color: var(--text-primary);
      }
      
      .widget-content button {
        background: var(--accent-color);
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 4px;
        cursor: pointer;
      }
    `;
  }

  getWidgetJS() {
    return `
      function myWidgetAction() {
        window.parent.postMessage({
          source: 'zeno_plugin',
          plugin_id: 'ui-plugin',
          type: 'widget_action',
          action: 'button_clicked'
        }, '*');
      }
    `;
  }
}
```

### 主题开发

```javascript
class ThemePlugin {
  async onEnable(config) {
    // 注册主题
    await this.callZenoAPI('register_theme', {
      id: 'my_theme',
      name: 'My Custom Theme',
      description: 'A beautiful custom theme',
      css: this.getThemeCSS(),
      variables: this.getThemeVariables()
    });
  }

  getThemeVariables() {
    return {
      // 颜色变量
      '--bg-primary': '#1a1a1a',
      '--bg-secondary': '#2d2d2d',
      '--text-primary': '#ffffff',
      '--text-secondary': '#cccccc',
      '--accent-color': '#007acc',
      '--border-color': '#404040',
      
      // 字体变量
      '--font-family': '"Fira Code", monospace',
      '--font-size': '14px',
      '--line-height': '1.5',
      
      // 布局变量
      '--sidebar-width': '260px',
      '--content-max-width': '800px',
      '--border-radius': '6px'
    };
  }

  getThemeCSS() {
    return `
      /* 全局样式重置 */
      body {
        background: var(--bg-primary);
        color: var(--text-primary);
        font-family: var(--font-family);
        font-size: var(--font-size);
        line-height: var(--line-height);
      }
      
      /* 侧边栏样式 */
      .sidebar {
        background: var(--bg-secondary);
        border-right: 1px solid var(--border-color);
        width: var(--sidebar-width);
      }
      
      /* 编辑器样式 */
      .editor {
        background: var(--bg-primary);
        color: var(--text-primary);
        max-width: var(--content-max-width);
        margin: 0 auto;
        padding: 20px;
      }
      
      /* 按钮样式 */
      .btn {
        background: var(--accent-color);
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: var(--border-radius);
        cursor: pointer;
        transition: opacity 0.2s;
      }
      
      .btn:hover {
        opacity: 0.8;
      }
    `;
  }
}
```

## 📦 打包和发布

### 本地测试

```bash
# 验证插件
zeno-plugin validate

# 运行测试
npm test

# 构建插件
npm run build
```

### 打包发布

```bash
# 创建发布包
zeno-plugin package

# 这会创建一个 .zpkg 文件
# my-first-plugin-1.0.0.zpkg
```

### 发布到市场

```bash
# 登录开发者账户
zeno-plugin login

# 发布插件
zeno-plugin publish my-first-plugin-1.0.0.zpkg

# 或者直接发布当前目录
zeno-plugin publish .
```

## 🐛 调试技巧

### 开发模式调试

```bash
# 启用详细日志
zeno --dev-plugins ./my-plugin --log-level debug

# 在浏览器中调试
# 插件运行在 iframe 中，可以在开发者工具中查看
```

### 插件日志

```javascript
class MyPlugin {
  log(level, message, data = null) {
    const logData = {
      type: 'log',
      level: level,
      message: message,
      data: data,
      timestamp: new Date().toISOString(),
      plugin_id: 'my-plugin'
    };
    
    console[level](`[${this.constructor.name}] ${message}`, data);
    this.sendToZeno(logData);
  }

  async onEnable(config) {
    this.log('info', 'Plugin enabling', { config });
    // ... 其他代码
    this.log('info', 'Plugin enabled successfully');
  }

  async handleError(error) {
    this.log('error', 'Plugin error occurred', {
      message: error.message,
      stack: error.stack
    });
    
    await this.callZenoAPI('show_notification', {
      type: 'error',
      message: `Plugin error: ${error.message}`
    });
  }
}
```

### 性能监控

```javascript
class PerformancePlugin {
  constructor() {
    this.metrics = new Map();
  }

  startTimer(name) {
    this.metrics.set(name, performance.now());
  }

  endTimer(name) {
    const startTime = this.metrics.get(name);
    if (startTime) {
      const duration = performance.now() - startTime;
      this.log('debug', `Timer ${name}: ${duration.toFixed(2)}ms`);
      this.metrics.delete(name);
      return duration;
    }
    return 0;
  }

  async measureAsync(name, fn) {
    this.startTimer(name);
    try {
      const result = await fn();
      this.endTimer(name);
      return result;
    } catch (error) {
      this.endTimer(name);
      throw error;
    }
  }
}
```

## 📋 最佳实践

### 1. 错误处理

```javascript
class RobustPlugin {
  async safeAPICall(endpoint, data) {
    try {
      return await this.callZenoAPI(endpoint, data);
    } catch (error) {
      this.log('error', `API call failed: ${endpoint}`, error);
      await this.showNotification('error', 
        `Operation failed: ${error.message}`);
      throw error;
    }
  }

  async withRetry(fn, maxRetries = 3) {
    for (let i = 0; i < maxRetries; i++) {
      try {
        return await fn();
      } catch (error) {
        if (i === maxRetries - 1) throw error;
        await this.delay(1000 * Math.pow(2, i)); // 指数退避
      }
    }
  }

  delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

### 2. 内存管理

```javascript
class MemoryEfficientPlugin {
  constructor() {
    this.cache = new Map();
    this.maxCacheSize = 100;
    this.cleanupInterval = null;
  }

  async onEnable(config) {
    // 启动定期清理
    this.cleanupInterval = setInterval(() => {
      this.cleanupCache();
    }, 60000); // 每分钟清理一次
  }

  async onDisable() {
    // 清理资源
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
    this.cache.clear();
  }

  cleanupCache() {
    if (this.cache.size > this.maxCacheSize) {
      const entries = Array.from(this.cache.entries());
      const toDelete = entries.slice(0, 
        entries.length - this.maxCacheSize);
      
      toDelete.forEach(([key]) => {
        this.cache.delete(key);
      });
    }
  }

  getCached(key, factory) {
    if (this.cache.has(key)) {
      return this.cache.get(key);
    }
    
    const value = factory();
    this.cache.set(key, value);
    return value;
  }
}
```

### 3. 用户体验

```javascript
class UserFriendlyPlugin {
  async performLongOperation() {
    // 显示进度通知
    await this.showNotification('info', 'Starting operation...');
    
    try {
      // 分块处理大量数据
      const data = await this.getLargeDataset();
      const chunks = this.chunkArray(data, 100);
      
      for (let i = 0; i < chunks.length; i++) {
        await this.processChunk(chunks[i]);
        
        // 更新进度
        const progress = Math.round((i + 1) / chunks.length * 100);
        await this.updateProgress(`Processing... ${progress}%`);
        
        // 给 UI 线程一些时间
        await this.delay(10);
      }
      
      await this.showNotification('success', 'Operation completed!');
    } catch (error) {
      await this.showNotification('error', 
        `Operation failed: ${error.message}`);
    }
  }

  chunkArray(array, size) {
    const chunks = [];
    for (let i = 0; i < array.length; i += size) {
      chunks.push(array.slice(i, i + size));
    }
    return chunks;
  }

  async updateProgress(message) {
    await this.callZenoAPI('update_status', { message });
  }
}
```

## 📖 参考资源

### API 文档
- [Zeno Plugin API Reference](https://docs.zeno.dev/api/plugins/)
- [Core API Documentation](https://docs.zeno.dev/api/core/)
- [UI Extension Guide](https://docs.zeno.dev/ui/extensions/)

### 示例项目
- [Hello World Plugin](./plugins/hello-world/)
- [Note Statistics Plugin](./plugins/note-stats/)
- [Theme Development Examples](./plugins/themes/)

### 社区资源
- [插件开发论坛](https://forum.zeno.dev/plugins/)
- [GitHub 示例仓库](https://github.com/foxzool/zeno-plugins/)
- [Discord 开发者频道](https://discord.gg/zeno-dev)

### 工具和库
- [@zeno/plugin-cli](https://www.npmjs.com/package/@zeno/plugin-cli) - 插件开发 CLI
- [@zeno/plugin-api](https://www.npmjs.com/package/@zeno/plugin-api) - API 客户端库
- [@zeno/plugin-types](https://www.npmjs.com/package/@zeno/plugin-types) - TypeScript 类型定义

---

通过这份指南，你应该能够开始开发自己的 Zeno 插件了。记住，插件开发是一个迭代过程，从简单的功能开始，逐步增加复杂性。

如果遇到问题，请查看[常见问题解答](https://docs.zeno.dev/plugins/faq/)或在[社区论坛](https://forum.zeno.dev/)寻求帮助。

祝你插件开发愉快！🚀