/**
 * Hello World Plugin for Zeno
 * 
 * 这是一个简单的示例插件，演示了 Zeno 插件系统的基本功能：
 * 1. 插件生命周期管理
 * 2. 与主应用的消息通信
 * 3. 配置系统的使用
 * 4. API 调用和事件处理
 */

class HelloWorldPlugin {
  constructor() {
    this.isEnabled = false;
    this.config = null;
    this.messageHandlers = new Map();
    
    // 注册消息处理器
    this.registerMessageHandlers();
  }

  /**
   * 插件启动时调用
   */
  async onEnable(config) {
    console.log('[HelloWorld] Plugin enabled with config:', config);
    this.isEnabled = true;
    this.config = config;
    
    // 向 Zeno 发送欢迎消息
    await this.sendNotification('success', this.getGreetingMessage());
    
    // 注册命令
    await this.registerCommands();
    
    // 监听事件
    await this.registerEventListeners();
  }

  /**
   * 插件停用时调用
   */
  async onDisable() {
    console.log('[HelloWorld] Plugin disabled');
    this.isEnabled = false;
    
    // 清理资源
    this.cleanup();
    
    await this.sendNotification('info', 'Hello World plugin disabled');
  }

  /**
   * 配置更新时调用
   */
  async onConfigUpdate(newConfig) {
    console.log('[HelloWorld] Config updated:', newConfig);
    this.config = newConfig;
  }

  /**
   * 注册消息处理器
   */
  registerMessageHandlers() {
    this.messageHandlers.set('ping', this.handlePing.bind(this));
    this.messageHandlers.set('hello', this.handleHello.bind(this));
    this.messageHandlers.set('get_info', this.handleGetInfo.bind(this));
    this.messageHandlers.set('count_notes', this.handleCountNotes.bind(this));
  }

  /**
   * 处理传入消息
   */
  async handleMessage(message) {
    const { type, data } = message;
    
    if (this.messageHandlers.has(type)) {
      try {
        const result = await this.messageHandlers.get(type)(data);
        return { success: true, data: result };
      } catch (error) {
        console.error(`[HelloWorld] Error handling message ${type}:`, error);
        return { success: false, error: error.message };
      }
    } else {
      return { success: false, error: `Unknown message type: ${type}` };
    }
  }

  /**
   * 处理 ping 消息
   */
  async handlePing(data) {
    return {
      message: 'pong',
      timestamp: new Date().toISOString(),
      data: data
    };
  }

  /**
   * 处理 hello 消息
   */
  async handleHello(data) {
    const name = data?.name || 'World';
    return {
      message: `Hello, ${name}!`,
      greeting: this.getGreetingMessage(),
      timestamp: this.config?.settings?.show_timestamp ? new Date().toISOString() : null
    };
  }

  /**
   * 处理获取插件信息请求
   */
  async handleGetInfo(data) {
    return {
      plugin_id: 'hello-world',
      version: '1.0.0',
      status: this.isEnabled ? 'enabled' : 'disabled',
      config: this.config,
      uptime: Date.now() - this.startTime,
      features: [
        'Basic messaging',
        'Configuration management',
        'Event handling',
        'Command registration'
      ]
    };
  }

  /**
   * 处理笔记计数请求
   */
  async handleCountNotes(data) {
    try {
      // 调用 Zeno API 获取笔记列表
      const response = await this.callZenoAPI('get_all_notes');
      const notes = response.data || [];
      
      return {
        total_notes: notes.length,
        note_titles: notes.slice(0, 5).map(note => note.title), // 只返回前5个标题作为示例
        query_time: new Date().toISOString()
      };
    } catch (error) {
      throw new Error(`Failed to count notes: ${error.message}`);
    }
  }

  /**
   * 获取问候消息
   */
  getGreetingMessage() {
    const baseMessage = this.config?.settings?.greeting_message || 'Hello from Zeno!';
    const timestamp = this.config?.settings?.show_timestamp ? 
      ` (${new Date().toLocaleString()})` : '';
    return baseMessage + timestamp;
  }

  /**
   * 注册命令
   */
  async registerCommands() {
    const commands = [
      {
        id: 'hello_world.greet',
        name: 'Hello World: Greet',
        description: 'Show a greeting message',
        shortcut: 'Ctrl+Shift+H'
      },
      {
        id: 'hello_world.count_notes',
        name: 'Hello World: Count Notes',
        description: 'Count total notes in workspace'
      }
    ];

    for (const command of commands) {
      await this.callZenoAPI('register_command', command);
    }
  }

  /**
   * 注册事件监听器
   */
  async registerEventListeners() {
    const events = [
      'note_created',
      'note_updated',
      'note_deleted'
    ];

    for (const event of events) {
      await this.callZenoAPI('subscribe_event', { event_type: event });
    }
  }

  /**
   * 处理 Zeno 事件
   */
  async handleEvent(event) {
    console.log('[HelloWorld] Received event:', event);
    
    switch (event.type) {
      case 'note_created':
        await this.sendNotification('info', `New note created: ${event.data.title}`);
        break;
      case 'note_updated':
        await this.sendNotification('info', `Note updated: ${event.data.title}`);
        break;
      case 'note_deleted':
        await this.sendNotification('warning', `Note deleted: ${event.data.title}`);
        break;
      case 'command_invoked':
        await this.handleCommand(event.data);
        break;
    }
  }

  /**
   * 处理命令调用
   */
  async handleCommand(commandData) {
    const { command_id } = commandData;
    
    switch (command_id) {
      case 'hello_world.greet':
        await this.sendNotification('success', this.getGreetingMessage());
        break;
      case 'hello_world.count_notes':
        try {
          const result = await this.handleCountNotes();
          await this.sendNotification('info', 
            `Total notes: ${result.total_notes}`);
        } catch (error) {
          await this.sendNotification('error', 
            `Failed to count notes: ${error.message}`);
        }
        break;
    }
  }

  /**
   * 调用 Zeno API
   */
  async callZenoAPI(endpoint, data = {}) {
    // 这里应该通过插件 API 调用 Zeno 的功能
    // 在实际实现中，这会通过消息传递或 IPC 实现
    const message = {
      type: 'api_call',
      endpoint: endpoint,
      data: data,
      timestamp: new Date().toISOString(),
      request_id: this.generateRequestId()
    };
    
    // 发送给 Zeno 主应用
    await this.sendToZeno(message);
    
    // 在实际实现中，这里会等待响应
    return { success: true, data: null };
  }

  /**
   * 发送通知给用户
   */
  async sendNotification(type, message) {
    const notification = {
      type: 'notification',
      level: type,
      message: message,
      timestamp: new Date().toISOString(),
      plugin_id: 'hello-world'
    };
    
    await this.sendToZeno(notification);
  }

  /**
   * 向 Zeno 主应用发送消息
   */
  async sendToZeno(message) {
    // 在实际实现中，这里会使用具体的通信机制
    // 比如 postMessage、IPC 或者其他插件运行时提供的 API
    console.log('[HelloWorld] Sending to Zeno:', message);
    
    // 模拟发送
    if (typeof window !== 'undefined' && window.parent !== window) {
      window.parent.postMessage({
        source: 'zeno_plugin',
        plugin_id: 'hello-world',
        ...message
      }, '*');
    }
  }

  /**
   * 生成请求 ID
   */
  generateRequestId() {
    return 'hw_' + Math.random().toString(36).substr(2, 9);
  }

  /**
   * 清理资源
   */
  cleanup() {
    this.messageHandlers.clear();
    this.config = null;
  }
}

// 创建插件实例
const plugin = new HelloWorldPlugin();

// 如果在 Node.js 环境中，导出插件
if (typeof module !== 'undefined' && module.exports) {
  module.exports = plugin;
}

// 如果在浏览器环境中，注册到全局
if (typeof window !== 'undefined') {
  window.ZenoPlugin = plugin;
}

// 插件入口点
async function main() {
  try {
    // 等待 Zeno 插件运行时就绪
    if (typeof window !== 'undefined') {
      window.addEventListener('message', async (event) => {
        if (event.data.source === 'zeno_runtime') {
          switch (event.data.type) {
            case 'enable':
              await plugin.onEnable(event.data.config);
              break;
            case 'disable':
              await plugin.onDisable();
              break;
            case 'config_update':
              await plugin.onConfigUpdate(event.data.config);
              break;
            case 'message':
              const result = await plugin.handleMessage(event.data.message);
              // 发送响应
              window.parent.postMessage({
                source: 'zeno_plugin',
                plugin_id: 'hello-world',
                type: 'response',
                request_id: event.data.request_id,
                result: result
              }, '*');
              break;
            case 'event':
              await plugin.handleEvent(event.data.event);
              break;
          }
        }
      });
      
      // 通知 Zeno 插件已加载
      window.parent.postMessage({
        source: 'zeno_plugin',
        plugin_id: 'hello-world',
        type: 'ready'
      }, '*');
    }
    
    console.log('[HelloWorld] Plugin loaded and ready');
  } catch (error) {
    console.error('[HelloWorld] Failed to initialize plugin:', error);
  }
}

// 启动插件
main();