use crate::models::plugin::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs;
use chrono::Utc;
use serde_json;

/// 插件管理器
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Arc<Plugin>>>,
    plugin_directory: PathBuf,
    config_directory: PathBuf,
    event_handlers: RwLock<HashMap<PluginEventType, Vec<String>>>,
    enabled_plugins: RwLock<Vec<String>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(plugin_directory: PathBuf, config_directory: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&plugin_directory)?;
        std::fs::create_dir_all(&config_directory)?;

        Ok(Self {
            plugins: RwLock::new(HashMap::new()),
            plugin_directory,
            config_directory,
            event_handlers: RwLock::new(HashMap::new()),
            enabled_plugins: RwLock::new(Vec::new()),
        })
    }

    /// 初始化插件管理器，扫描并加载已安装的插件
    pub async fn initialize(&self) -> Result<()> {
        log::info!("Initializing plugin manager...");
        
        // 扫描插件目录
        self.scan_plugins().await?;
        
        // 加载插件配置
        self.load_plugin_configs().await?;
        
        // 启用自动启用的插件
        self.enable_auto_plugins().await?;
        
        log::info!("Plugin manager initialized successfully");
        Ok(())
    }

    /// 扫描插件目录并加载插件清单
    async fn scan_plugins(&self) -> Result<()> {
        let mut entries = fs::read_dir(&self.plugin_directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                if let Err(e) = self.load_plugin_from_directory(&path).await {
                    log::warn!("Failed to load plugin from {}: {}", path.display(), e);
                }
            }
        }
        
        Ok(())
    }

    /// 从目录加载插件
    async fn load_plugin_from_directory(&self, plugin_dir: &Path) -> Result<()> {
        let manifest_path = plugin_dir.join("plugin.json");
        if !manifest_path.exists() {
            return Err(anyhow!("Plugin manifest not found: {}", manifest_path.display()));
        }

        let manifest_content = fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;

        // 验证插件完整性
        self.validate_plugin_structure(&manifest, plugin_dir)?;

        let plugin_name = manifest.name.clone();
        let plugin_version = manifest.version.clone();
        let plugin = Plugin::new(manifest, plugin_dir.to_string_lossy().to_string());
        
        // 添加到插件列表
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin.id.clone(), Arc::new(plugin));
        }

        log::info!("Loaded plugin: {} v{}", plugin_name, plugin_version);
        Ok(())
    }

    /// 验证插件目录结构
    fn validate_plugin_structure(&self, manifest: &PluginManifest, plugin_dir: &Path) -> Result<()> {
        // 检查主入口文件
        let main_file = plugin_dir.join(&manifest.main);
        if !main_file.exists() {
            return Err(anyhow!("Main file not found: {}", main_file.display()));
        }

        // 检查必需文件
        for file in &manifest.files {
            let file_path = plugin_dir.join(file);
            if !file_path.exists() {
                return Err(anyhow!("Required file not found: {}", file_path.display()));
            }
        }

        Ok(())
    }

    /// 加载插件配置
    async fn load_plugin_configs(&self) -> Result<()> {
        let config_file = self.config_directory.join("plugins.json");
        if !config_file.exists() {
            return Ok(());
        }

        let config_content = fs::read_to_string(&config_file).await?;
        let configs: HashMap<String, PluginConfig> = serde_json::from_str(&config_content)?;

        // 应用配置到插件
        let plugins = self.plugins.read().unwrap();
        for (plugin_id, config) in configs {
            if let Some(plugin) = plugins.get(&plugin_id) {
                // 这里需要实现配置更新逻辑
                // 由于 Plugin 在 Arc 中，需要使用内部可变性或重新设计
                log::info!("Loaded config for plugin: {}", plugin_id);
            }
        }

        Ok(())
    }

    /// 启用自动启用的插件
    async fn enable_auto_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        let mut to_enable = Vec::new();

        for plugin in plugins.values() {
            if plugin.config.auto_enable {
                to_enable.push(plugin.id.clone());
            }
        }

        drop(plugins); // 释放读锁

        for plugin_id in to_enable {
            if let Err(e) = self.enable_plugin(&plugin_id).await {
                log::warn!("Failed to auto-enable plugin {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// 安装插件
    pub async fn install_plugin(&self, source: &str, options: &PluginInstallOptions) -> Result<String> {
        log::info!("Installing plugin from: {}", source);

        // 确定安装来源类型
        let install_path = if source.starts_with("http://") || source.starts_with("https://") {
            self.install_from_url(source, options).await?
        } else if Path::new(source).exists() {
            self.install_from_local(source, options).await?
        } else {
            self.install_from_marketplace(source, options).await?
        };

        // 重新扫描插件目录
        self.load_plugin_from_directory(&install_path).await?;

        let plugin_name = install_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        log::info!("Plugin installed successfully: {}", plugin_name);
        Ok(plugin_name)
    }

    /// 从 URL 安装插件
    async fn install_from_url(&self, url: &str, _options: &PluginInstallOptions) -> Result<PathBuf> {
        // 下载插件包
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;

        // 解压并安装
        // 这里需要实现 zip 解压逻辑
        // 暂时返回一个占位符路径
        Ok(self.plugin_directory.join("downloaded_plugin"))
    }

    /// 从本地文件安装插件
    async fn install_from_local(&self, path: &str, _options: &PluginInstallOptions) -> Result<PathBuf> {
        let source_path = Path::new(path);
        
        if source_path.is_file() && path.ends_with(".zip") {
            // 解压 zip 文件
            self.extract_plugin_zip(source_path).await
        } else if source_path.is_dir() {
            // 复制目录
            self.copy_plugin_directory(source_path).await
        } else {
            Err(anyhow!("Invalid plugin source: {}", path))
        }
    }

    /// 从市场安装插件
    async fn install_from_marketplace(&self, plugin_id: &str, options: &PluginInstallOptions) -> Result<PathBuf> {
        // 查询插件市场
        let market_info = self.query_marketplace(plugin_id).await?;
        
        // 从市场下载
        self.install_from_url(&market_info.download_url, options).await
    }

    /// 解压插件 ZIP 文件
    async fn extract_plugin_zip(&self, zip_path: &Path) -> Result<PathBuf> {
        // 实现 ZIP 解压逻辑
        // 这里需要使用 zip 库来解压文件
        let target_dir = self.plugin_directory.join("extracted_plugin");
        std::fs::create_dir_all(&target_dir)?;
        
        // 占位符实现
        Ok(target_dir)
    }

    /// 复制插件目录
    async fn copy_plugin_directory(&self, source: &Path) -> Result<PathBuf> {
        let plugin_name = source.file_name()
            .ok_or_else(|| anyhow!("Invalid directory name"))?
            .to_string_lossy();
        
        let target_dir = self.plugin_directory.join(&*plugin_name);
        
        // 复制目录内容
        self.copy_directory_recursive(source, &target_dir).await?;
        
        Ok(target_dir)
    }

    /// 递归复制目录
    fn copy_directory_recursive<'a>(
        &'a self,
        source: &'a Path,
        target: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            fs::create_dir_all(target).await?;
            
            let mut entries = fs::read_dir(source).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let target_path = target.join(entry.file_name());
                
                if entry_path.is_dir() {
                    self.copy_directory_recursive(&entry_path, &target_path).await?;
                } else {
                    fs::copy(&entry_path, &target_path).await?;
                }
            }
            
            Ok(())
        })
    }

    /// 查询插件市场
    async fn query_marketplace(&self, plugin_id: &str) -> Result<PluginMarketInfo> {
        // 这里应该连接到实际的插件市场 API
        // 暂时返回模拟数据
        Ok(PluginMarketInfo {
            id: plugin_id.to_string(),
            name: format!("Plugin {}", plugin_id),
            description: "A sample plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            downloads: 0,
            rating: 0.0,
            rating_count: 0,
            last_updated: Utc::now(),
            size: 1024,
            screenshots: Vec::new(),
            tags: Vec::new(),
            compatible_versions: vec!["*".to_string()],
            download_url: format!("https://marketplace.zeno.dev/plugins/{}.zip", plugin_id),
            verified: false,
        })
    }

    /// 卸载插件
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<()> {
        log::info!("Uninstalling plugin: {}", plugin_id);

        // 先禁用插件
        self.disable_plugin(plugin_id).await?;

        // 获取插件信息
        let install_path = {
            let plugins = self.plugins.read().unwrap();
            let plugin = plugins.get(plugin_id)
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
            PathBuf::from(&plugin.install_path)
        };

        // 删除插件文件
        if install_path.exists() {
            fs::remove_dir_all(&install_path).await?;
        }

        // 从插件列表中移除
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_id);
        }

        log::info!("Plugin uninstalled successfully: {}", plugin_id);
        Ok(())
    }

    /// 启用插件
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        log::info!("Enabling plugin: {}", plugin_id);

        // 检查插件是否存在
        let plugin = {
            let plugins = self.plugins.read().unwrap();
            plugins.get(plugin_id)
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
                .clone()
        };

        // 检查依赖
        self.check_dependencies(&plugin).await?;

        // 验证权限
        self.validate_permissions(&plugin)?;

        // 加载插件
        self.load_plugin(&plugin).await?;

        // 添加到启用列表
        {
            let mut enabled = self.enabled_plugins.write().unwrap();
            if !enabled.contains(&plugin_id.to_string()) {
                enabled.push(plugin_id.to_string());
            }
        }

        // 触发插件启用事件
        self.emit_event(PluginEvent {
            event_type: PluginEventType::PluginEnabled,
            data: serde_json::json!({ "plugin_id": plugin_id }),
            timestamp: Utc::now(),
            source: "plugin_manager".to_string(),
        }).await;

        log::info!("Plugin enabled successfully: {}", plugin_id);
        Ok(())
    }

    /// 禁用插件
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        log::info!("Disabling plugin: {}", plugin_id);

        // 卸载插件
        self.unload_plugin(plugin_id).await?;

        // 从启用列表中移除
        {
            let mut enabled = self.enabled_plugins.write().unwrap();
            enabled.retain(|id| id != plugin_id);
        }

        // 触发插件禁用事件
        self.emit_event(PluginEvent {
            event_type: PluginEventType::PluginDisabled,
            data: serde_json::json!({ "plugin_id": plugin_id }),
            timestamp: Utc::now(),
            source: "plugin_manager".to_string(),
        }).await;

        log::info!("Plugin disabled successfully: {}", plugin_id);
        Ok(())
    }

    /// 检查插件依赖
    async fn check_dependencies(&self, plugin: &Plugin) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        
        for dependency in &plugin.dependencies {
            if !dependency.optional {
                if !plugins.contains_key(&dependency.id) {
                    return Err(anyhow!("Missing required dependency: {}", dependency.id));
                }
                
                // 这里应该检查版本兼容性
                // 使用 semver 库进行版本检查
            }
        }
        
        Ok(())
    }

    /// 验证插件权限
    fn validate_permissions(&self, plugin: &Plugin) -> Result<()> {
        // 这里可以实现权限验证逻辑
        // 检查插件请求的权限是否安全
        // 可以有白名单、黑名单等机制
        
        log::debug!("Validating permissions for plugin: {}", plugin.id);
        Ok(())
    }

    /// 加载插件
    async fn load_plugin(&self, plugin: &Plugin) -> Result<()> {
        // 这里需要实现实际的插件加载逻辑
        // 可能需要启动独立的进程、加载动态库等
        // 对于 JavaScript 插件，可以使用 V8 或类似的引擎
        
        log::info!("Loading plugin: {} from {}", plugin.name, plugin.main);
        
        // 占位符实现
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    /// 卸载插件
    async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        // 实现插件卸载逻辑
        // 清理资源、停止进程等
        
        log::info!("Unloading plugin: {}", plugin_id);
        
        // 占位符实现
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(())
    }

    /// 获取所有插件列表
    pub fn get_all_plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| (**p).clone()).collect()
    }

    /// 获取已启用的插件列表
    pub fn get_enabled_plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        let enabled = self.enabled_plugins.read().unwrap();
        
        enabled.iter()
            .filter_map(|id| plugins.get(id))
            .map(|p| (**p).clone())
            .collect()
    }

    /// 检查插件更新
    pub async fn check_updates(&self) -> Result<Vec<PluginUpdateInfo>> {
        // 首先获取插件列表并释放锁
        let plugins_to_check = {
            let plugins = self.plugins.read().unwrap();
            plugins.values().map(|p| (**p).clone()).collect::<Vec<_>>()
        };
        
        let mut updates = Vec::new();
        
        for plugin in plugins_to_check {
            if let Ok(update_info) = self.check_plugin_update(&plugin).await {
                if update_info.current_version != update_info.latest_version {
                    updates.push(update_info);
                }
            }
        }
        
        Ok(updates)
    }

    /// 检查单个插件更新
    async fn check_plugin_update(&self, plugin: &Plugin) -> Result<PluginUpdateInfo> {
        // 查询插件市场获取最新版本信息
        let market_info = self.query_marketplace(&plugin.id).await?;
        
        Ok(PluginUpdateInfo {
            plugin_id: plugin.id.clone(),
            current_version: plugin.version.clone(),
            latest_version: market_info.version,
            changelog: "New features and bug fixes".to_string(), // 从市场获取
            breaking_changes: false, // 从市场获取
            download_url: market_info.download_url,
            size: market_info.size,
        })
    }

    /// 发送事件
    pub async fn emit_event(&self, event: PluginEvent) {
        log::debug!("Emitting event: {:?}", event.event_type);
        
        // 获取事件处理器
        let handlers = {
            let event_handlers = self.event_handlers.read().unwrap();
            event_handlers.get(&event.event_type).cloned().unwrap_or_default()
        };
        
        // 通知所有注册的插件
        for plugin_id in handlers {
            if let Err(e) = self.notify_plugin(&plugin_id, &event).await {
                log::warn!("Failed to notify plugin {}: {}", plugin_id, e);
            }
        }
    }

    /// 通知插件事件
    async fn notify_plugin(&self, plugin_id: &str, event: &PluginEvent) -> Result<()> {
        // 实现插件事件通知逻辑
        // 可以通过 IPC、消息队列等方式通知插件
        
        log::debug!("Notifying plugin {} of event {:?}", plugin_id, event.event_type);
        Ok(())
    }

    /// 注册事件处理器
    pub fn register_event_handler(&self, event_type: PluginEventType, plugin_id: String) {
        let mut handlers = self.event_handlers.write().unwrap();
        handlers.entry(event_type).or_insert_with(Vec::new).push(plugin_id);
    }

    /// 保存插件配置
    pub async fn save_configs(&self) -> Result<()> {
        // 避免跨越 await 边界持有锁
        let configs = {
            let plugins = self.plugins.read().unwrap();
            let mut configs = HashMap::new();
            
            for (id, plugin) in plugins.iter() {
                configs.insert(id.clone(), plugin.config.clone());
            }
            
            configs
        };
        
        let config_file = self.config_directory.join("plugins.json");
        let config_content = serde_json::to_string_pretty(&configs)?;
        fs::write(&config_file, config_content).await?;
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plugin_dir = home_dir.join(".zeno").join("plugins");
        let config_dir = home_dir.join(".zeno").join("config");
        
        Self::new(plugin_dir, config_dir).unwrap()
    }
}