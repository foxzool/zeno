use crate::models::plugin::*;
use crate::models::plugin_api::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use chrono::Utc;

/// 插件运行时管理器
pub struct PluginRuntimeManager {
    /// JavaScript 运行时实例
    js_runtimes: RwLock<HashMap<String, JSRuntimeInstance>>,
    /// WASM 运行时实例
    wasm_runtimes: RwLock<HashMap<String, WASMRuntimeInstance>>,
    /// 本地进程运行时实例
    process_runtimes: RwLock<HashMap<String, ProcessRuntimeInstance>>,
    /// 沙箱配置
    sandbox_configs: RwLock<HashMap<String, PluginSandbox>>,
    /// 运行时监控
    runtime_monitor: Arc<RuntimeMonitor>,
}

/// JavaScript 运行时实例
struct JSRuntimeInstance {
    plugin_id: String,
    runtime_config: JSPluginRuntime,
    isolate_handle: Option<()>, // V8 Isolate handle (占位符)
    message_channel: mpsc::UnboundedSender<PluginMessage>,
    status: PluginRuntimeStatus,
    start_time: chrono::DateTime<Utc>,
}

/// WASM 运行时实例
struct WASMRuntimeInstance {
    plugin_id: String,
    runtime_config: WASMPluginRuntime,
    module_instance: Option<()>, // WASM Module instance (占位符)
    message_channel: mpsc::UnboundedSender<PluginMessage>,
    status: PluginRuntimeStatus,
    start_time: chrono::DateTime<Utc>,
}

/// 进程运行时实例
struct ProcessRuntimeInstance {
    plugin_id: String,
    process: Child,
    message_channel: mpsc::UnboundedSender<PluginMessage>,
    status: PluginRuntimeStatus,
    start_time: chrono::DateTime<Utc>,
}

/// 运行时监控器
pub struct RuntimeMonitor {
    monitoring_enabled: RwLock<bool>,
    resource_usage: RwLock<HashMap<String, ResourceUsage>>,
}

/// 资源使用情况
#[derive(Debug, Clone)]
struct ResourceUsage {
    memory_mb: f64,
    cpu_percent: f64,
    network_usage: NetworkUsage,
    last_updated: chrono::DateTime<Utc>,
}

impl PluginRuntimeManager {
    /// 创建新的插件运行时管理器
    pub fn new() -> Self {
        Self {
            js_runtimes: RwLock::new(HashMap::new()),
            wasm_runtimes: RwLock::new(HashMap::new()),
            process_runtimes: RwLock::new(HashMap::new()),
            sandbox_configs: RwLock::new(HashMap::new()),
            runtime_monitor: Arc::new(RuntimeMonitor::new()),
        }
    }

    /// 启动插件运行时
    pub async fn start_plugin_runtime(&self, plugin: &Plugin) -> Result<PluginRuntimeState> {
        log::info!("Starting runtime for plugin: {}", plugin.id);
        
        // 创建沙箱配置
        let sandbox_config = self.create_sandbox_config(plugin)?;
        {
            let mut configs = self.sandbox_configs.write().unwrap();
            configs.insert(plugin.id.clone(), sandbox_config.clone());
        }
        
        // 根据插件类型启动相应的运行时
        let runtime_state = match self.detect_plugin_type(plugin)? {
            PluginType::JavaScript => self.start_js_runtime(plugin, &sandbox_config).await?,
            PluginType::WASM => self.start_wasm_runtime(plugin, &sandbox_config).await?,
            PluginType::Native => self.start_process_runtime(plugin, &sandbox_config).await?,
        };
        
        // 启动监控
        self.runtime_monitor.start_monitoring(&plugin.id).await?;
        
        log::info!("Plugin runtime started successfully: {}", plugin.id);
        Ok(runtime_state)
    }

    /// 停止插件运行时
    pub async fn stop_plugin_runtime(&self, plugin_id: &str) -> Result<()> {
        log::info!("Stopping runtime for plugin: {}", plugin_id);
        
        // 停止各类型的运行时
        self.stop_js_runtime(plugin_id).await?;
        self.stop_wasm_runtime(plugin_id).await?;
        self.stop_process_runtime(plugin_id).await?;
        
        // 停止监控
        self.runtime_monitor.stop_monitoring(plugin_id).await?;
        
        // 清理沙箱配置
        {
            let mut configs = self.sandbox_configs.write().unwrap();
            configs.remove(plugin_id);
        }
        
        log::info!("Plugin runtime stopped successfully: {}", plugin_id);
        Ok(())
    }

    /// 重启插件运行时
    pub async fn restart_plugin_runtime(&self, plugin: &Plugin) -> Result<PluginRuntimeState> {
        log::info!("Restarting runtime for plugin: {}", plugin.id);
        
        // 先停止现有运行时
        if let Err(e) = self.stop_plugin_runtime(&plugin.id).await {
            log::warn!("Failed to stop existing runtime: {}", e);
        }
        
        // 等待一段时间确保清理完成
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // 重新启动
        self.start_plugin_runtime(plugin).await
    }

    /// 获取插件运行时状态
    pub fn get_runtime_state(&self, plugin_id: &str) -> Option<PluginRuntimeState> {
        // 检查各种运行时类型
        if let Some(js_runtime) = self.js_runtimes.read().unwrap().get(plugin_id) {
            return Some(self.create_runtime_state_from_js(js_runtime));
        }
        
        if let Some(wasm_runtime) = self.wasm_runtimes.read().unwrap().get(plugin_id) {
            return Some(self.create_runtime_state_from_wasm(wasm_runtime));
        }
        
        if let Some(process_runtime) = self.process_runtimes.read().unwrap().get(plugin_id) {
            return Some(self.create_runtime_state_from_process(process_runtime));
        }
        
        None
    }

    /// 向插件发送消息
    pub async fn send_message(&self, plugin_id: &str, message: PluginMessage) -> Result<()> {
        // 尝试发送到各种运行时
        if let Some(js_runtime) = self.js_runtimes.read().unwrap().get(plugin_id) {
            return js_runtime.message_channel.send(message)
                .map_err(|e| anyhow!("Failed to send message to JS runtime: {}", e));
        }
        
        if let Some(wasm_runtime) = self.wasm_runtimes.read().unwrap().get(plugin_id) {
            return wasm_runtime.message_channel.send(message)
                .map_err(|e| anyhow!("Failed to send message to WASM runtime: {}", e));
        }
        
        if let Some(process_runtime) = self.process_runtimes.read().unwrap().get(plugin_id) {
            return process_runtime.message_channel.send(message)
                .map_err(|e| anyhow!("Failed to send message to process runtime: {}", e));
        }
        
        Err(anyhow!("Plugin runtime not found: {}", plugin_id))
    }

    /// 获取所有运行时状态
    pub fn get_all_runtime_states(&self) -> Vec<PluginRuntimeState> {
        let mut states = Vec::new();
        
        // 收集 JS 运行时状态
        {
            let js_runtimes = self.js_runtimes.read().unwrap();
            for runtime in js_runtimes.values() {
                states.push(self.create_runtime_state_from_js(runtime));
            }
        }
        
        // 收集 WASM 运行时状态
        {
            let wasm_runtimes = self.wasm_runtimes.read().unwrap();
            for runtime in wasm_runtimes.values() {
                states.push(self.create_runtime_state_from_wasm(runtime));
            }
        }
        
        // 收集进程运行时状态
        {
            let process_runtimes = self.process_runtimes.read().unwrap();
            for runtime in process_runtimes.values() {
                states.push(self.create_runtime_state_from_process(runtime));
            }
        }
        
        states
    }

    /// 清理崩溃的运行时
    pub async fn cleanup_crashed_runtimes(&self) -> Result<Vec<String>> {
        let mut cleaned_plugins = Vec::new();
        
        // 清理 JS 运行时
        {
            let mut js_runtimes = self.js_runtimes.write().unwrap();
            js_runtimes.retain(|plugin_id, runtime| {
                if matches!(runtime.status, PluginRuntimeStatus::Crashed | PluginRuntimeStatus::Error) {
                    cleaned_plugins.push(plugin_id.clone());
                    log::info!("Cleaning up crashed JS runtime: {}", plugin_id);
                    false
                } else {
                    true
                }
            });
        }
        
        // 清理 WASM 运行时
        {
            let mut wasm_runtimes = self.wasm_runtimes.write().unwrap();
            wasm_runtimes.retain(|plugin_id, runtime| {
                if matches!(runtime.status, PluginRuntimeStatus::Crashed | PluginRuntimeStatus::Error) {
                    cleaned_plugins.push(plugin_id.clone());
                    log::info!("Cleaning up crashed WASM runtime: {}", plugin_id);
                    false
                } else {
                    true
                }
            });
        }
        
        // 清理进程运行时
        {
            let mut process_runtimes = self.process_runtimes.write().unwrap();
            process_runtimes.retain(|plugin_id, runtime| {
                if matches!(runtime.status, PluginRuntimeStatus::Crashed | PluginRuntimeStatus::Error) {
                    cleaned_plugins.push(plugin_id.clone());
                    log::info!("Cleaning up crashed process runtime: {}", plugin_id);
                    false
                } else {
                    true
                }
            });
        }
        
        Ok(cleaned_plugins)
    }

    // 私有方法实现

    /// 检测插件类型
    fn detect_plugin_type(&self, plugin: &Plugin) -> Result<PluginType> {
        let main_file = Path::new(&plugin.main);
        
        if let Some(extension) = main_file.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "js" | "mjs" | "ts" => Ok(PluginType::JavaScript),
                "wasm" => Ok(PluginType::WASM),
                "exe" | "" => Ok(PluginType::Native),
                _ => Ok(PluginType::JavaScript), // 默认为 JavaScript
            }
        } else {
            Ok(PluginType::Native)
        }
    }

    /// 创建沙箱配置
    fn create_sandbox_config(&self, plugin: &Plugin) -> Result<PluginSandbox> {
        let mut sandbox = PluginSandbox::default();
        
        // 根据插件权限配置沙箱
        sandbox.file_system_access.read_only = !plugin.permissions.file_system.write_workspace;
        sandbox.file_system_access.allowed_paths = plugin.permissions.file_system.allowed_paths.clone();
        sandbox.file_system_access.denied_paths = plugin.permissions.file_system.denied_paths.clone();
        
        sandbox.network_access.enabled = plugin.permissions.network.http_request;
        sandbox.network_access.allowed_domains = plugin.permissions.network.allowed_domains.clone();
        sandbox.network_access.denied_domains = plugin.permissions.network.denied_domains.clone();
        
        // 设置资源限制
        sandbox.memory_limit_mb = 128; // 默认 128MB
        sandbox.cpu_limit_percent = 25.0; // 默认 25% CPU
        sandbox.timeout_seconds = 60; // 默认 60 秒超时
        
        Ok(sandbox)
    }

    /// 启动 JavaScript 运行时
    async fn start_js_runtime(&self, plugin: &Plugin, _sandbox: &PluginSandbox) -> Result<PluginRuntimeState> {
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        let runtime_config = JSPluginRuntime {
            script_path: plugin.main.clone(),
            global_objects: HashMap::new(),
            imported_modules: Vec::new(),
            console_buffer: Vec::new(),
            timeout_ms: 30000,
            max_memory_mb: 64,
        };
        
        let js_runtime = JSRuntimeInstance {
            plugin_id: plugin.id.clone(),
            runtime_config,
            isolate_handle: None, // V8 集成将在这里实现
            message_channel: sender,
            status: PluginRuntimeStatus::Starting,
            start_time: Utc::now(),
        };
        
        // 这里应该实际启动 V8 isolate 并加载插件脚本
        // 占位符实现
        
        {
            let mut js_runtimes = self.js_runtimes.write().unwrap();
            js_runtimes.insert(plugin.id.clone(), js_runtime);
        }
        
        Ok(PluginRuntimeState {
            plugin_id: plugin.id.clone(),
            status: PluginRuntimeStatus::Running,
            memory_usage: 0,
            cpu_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: 0,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        })
    }

    /// 启动 WASM 运行时
    async fn start_wasm_runtime(&self, plugin: &Plugin, _sandbox: &PluginSandbox) -> Result<PluginRuntimeState> {
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        let runtime_config = WASMPluginRuntime {
            wasm_path: plugin.main.clone(),
            imports: HashMap::new(),
            exports: HashMap::new(),
            memory_pages: 1,
            max_memory_pages: 10,
            fuel_limit: Some(1000000),
        };
        
        let wasm_runtime = WASMRuntimeInstance {
            plugin_id: plugin.id.clone(),
            runtime_config,
            module_instance: None, // WASM 集成将在这里实现
            message_channel: sender,
            status: PluginRuntimeStatus::Starting,
            start_time: Utc::now(),
        };
        
        // 这里应该实际加载和实例化 WASM 模块
        // 占位符实现
        
        {
            let mut wasm_runtimes = self.wasm_runtimes.write().unwrap();
            wasm_runtimes.insert(plugin.id.clone(), wasm_runtime);
        }
        
        Ok(PluginRuntimeState {
            plugin_id: plugin.id.clone(),
            status: PluginRuntimeStatus::Running,
            memory_usage: 0,
            cpu_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: 0,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        })
    }

    /// 启动进程运行时
    async fn start_process_runtime(&self, plugin: &Plugin, _sandbox: &PluginSandbox) -> Result<PluginRuntimeState> {
        let plugin_path = Path::new(&plugin.install_path).join(&plugin.main);
        
        let mut cmd = Command::new(&plugin_path);
        cmd.env("ZENO_PLUGIN_ID", &plugin.id);
        cmd.env("ZENO_PLUGIN_VERSION", &plugin.version);
        
        let child = cmd.spawn()
            .map_err(|e| anyhow!("Failed to start plugin process: {}", e))?;
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        let process_runtime = ProcessRuntimeInstance {
            plugin_id: plugin.id.clone(),
            process: child,
            message_channel: sender,
            status: PluginRuntimeStatus::Starting,
            start_time: Utc::now(),
        };
        
        {
            let mut process_runtimes = self.process_runtimes.write().unwrap();
            process_runtimes.insert(plugin.id.clone(), process_runtime);
        }
        
        Ok(PluginRuntimeState {
            plugin_id: plugin.id.clone(),
            status: PluginRuntimeStatus::Running,
            memory_usage: 0,
            cpu_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: 0,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        })
    }

    /// 停止 JavaScript 运行时
    async fn stop_js_runtime(&self, plugin_id: &str) -> Result<()> {
        let mut js_runtimes = self.js_runtimes.write().unwrap();
        if js_runtimes.remove(plugin_id).is_some() {
            log::info!("Stopped JS runtime for plugin: {}", plugin_id);
        }
        Ok(())
    }

    /// 停止 WASM 运行时
    async fn stop_wasm_runtime(&self, plugin_id: &str) -> Result<()> {
        let mut wasm_runtimes = self.wasm_runtimes.write().unwrap();
        if wasm_runtimes.remove(plugin_id).is_some() {
            log::info!("Stopped WASM runtime for plugin: {}", plugin_id);
        }
        Ok(())
    }

    /// 停止进程运行时
    async fn stop_process_runtime(&self, plugin_id: &str) -> Result<()> {
        // 获取进程句柄，然后释放锁
        let mut process = {
            let mut process_runtimes = self.process_runtimes.write().unwrap();
            if let Some(runtime) = process_runtimes.remove(plugin_id) {
                Some(runtime.process)
            } else {
                None
            }
        };
        
        // 在释放锁之后执行异步操作
        if let Some(ref mut proc) = process {
            let _ = proc.kill().await;
            log::info!("Stopped process runtime for plugin: {}", plugin_id);
        }
        
        Ok(())
    }

    /// 从 JS 运行时创建状态
    fn create_runtime_state_from_js(&self, runtime: &JSRuntimeInstance) -> PluginRuntimeState {
        let uptime = (Utc::now() - runtime.start_time).num_seconds() as u64;
        
        PluginRuntimeState {
            plugin_id: runtime.plugin_id.clone(),
            status: runtime.status.clone(),
            memory_usage: 0, // 从监控器获取
            cpu_usage: 0.0,  // 从监控器获取
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: uptime,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// 从 WASM 运行时创建状态
    fn create_runtime_state_from_wasm(&self, runtime: &WASMRuntimeInstance) -> PluginRuntimeState {
        let uptime = (Utc::now() - runtime.start_time).num_seconds() as u64;
        
        PluginRuntimeState {
            plugin_id: runtime.plugin_id.clone(),
            status: runtime.status.clone(),
            memory_usage: 0,
            cpu_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: uptime,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// 从进程运行时创建状态
    fn create_runtime_state_from_process(&self, runtime: &ProcessRuntimeInstance) -> PluginRuntimeState {
        let uptime = (Utc::now() - runtime.start_time).num_seconds() as u64;
        
        PluginRuntimeState {
            plugin_id: runtime.plugin_id.clone(),
            status: runtime.status.clone(),
            memory_usage: 0,
            cpu_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                active_connections: 0,
            },
            uptime_seconds: uptime,
            last_heartbeat: Utc::now(),
            error_count: 0,
            warning_count: 0,
        }
    }
}

/// 插件类型枚举
#[derive(Debug, Clone)]
enum PluginType {
    JavaScript,
    WASM,
    Native,
}

impl RuntimeMonitor {
    fn new() -> Self {
        Self {
            monitoring_enabled: RwLock::new(false),
            resource_usage: RwLock::new(HashMap::new()),
        }
    }

    async fn start_monitoring(&self, plugin_id: &str) -> Result<()> {
        log::debug!("Starting monitoring for plugin: {}", plugin_id);
        
        {
            let mut usage = self.resource_usage.write().unwrap();
            usage.insert(plugin_id.to_string(), ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                network_usage: NetworkUsage {
                    bytes_sent: 0,
                    bytes_received: 0,
                    requests_count: 0,
                    active_connections: 0,
                },
                last_updated: Utc::now(),
            });
        }
        
        Ok(())
    }

    async fn stop_monitoring(&self, plugin_id: &str) -> Result<()> {
        log::debug!("Stopping monitoring for plugin: {}", plugin_id);
        
        {
            let mut usage = self.resource_usage.write().unwrap();
            usage.remove(plugin_id);
        }
        
        Ok(())
    }
}

impl Default for PluginRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}