use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub categories: Vec<PluginCategory>,
    pub main: String, // 主入口文件路径
    pub manifest_path: String,
    pub install_path: String,
    pub enabled: bool,
    pub config: PluginConfig,
    pub permissions: PluginPermissions,
    pub dependencies: Vec<PluginDependency>,
    pub installed_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub checksum: String,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum PluginCategory {
    Editor,       // 编辑器增强
    Export,       // 导出功能
    Import,       // 导入功能
    Theme,        // 主题
    Workflow,     // 工作流
    Integration,  // 第三方集成
    Utility,      // 实用工具
    Developer,    // 开发工具
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub settings: HashMap<String, serde_json::Value>,
    pub shortcuts: HashMap<String, String>,
    pub ui_config: Option<UIConfig>,
    pub auto_enable: bool,
    pub update_check: bool,
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub show_in_menu: bool,
    pub menu_position: MenuPosition,
    pub toolbar_button: Option<ToolbarButton>,
    pub sidebar_panel: Option<SidebarPanel>,
    pub status_bar: Option<StatusBarItem>,
}

/// 菜单位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuPosition {
    File,
    Edit,
    View,
    Tools,
    Help,
    Custom(String),
}

/// 工具栏按钮
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarButton {
    pub icon: String,
    pub tooltip: String,
    pub position: i32,
}

/// 侧边栏面板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarPanel {
    pub title: String,
    pub icon: String,
    pub position: i32,
    pub collapsible: bool,
}

/// 状态栏项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarItem {
    pub text: String,
    pub tooltip: Option<String>,
    pub position: StatusBarPosition,
}

/// 状态栏位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusBarPosition {
    Left,
    Center,
    Right,
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub file_system: FileSystemPermissions,
    pub network: NetworkPermissions,
    pub ui: UIPermissions,
    pub system: SystemPermissions,
}

/// 文件系统权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPermissions {
    pub read_workspace: bool,
    pub write_workspace: bool,
    pub read_system: bool,
    pub write_system: bool,
    pub execute: bool,
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
}

/// 网络权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    pub http_request: bool,
    pub websocket: bool,
    pub allowed_domains: Vec<String>,
    pub denied_domains: Vec<String>,
}

/// UI 权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPermissions {
    pub modify_ui: bool,
    pub show_notifications: bool,
    pub create_dialogs: bool,
    pub access_clipboard: bool,
}

/// 系统权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermissions {
    pub execute_commands: bool,
    pub access_env_vars: bool,
    pub create_processes: bool,
    pub access_hardware: bool,
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version_requirement: String,
    pub optional: bool,
}

/// 插件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Installed,
    Enabled,
    Disabled,
    Error(String),
    Updating,
    Installing,
    Uninstalling,
}

/// 插件API事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub event_type: PluginEventType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// 插件事件类型
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum PluginEventType {
    NoteCreated,
    NoteUpdated,
    NoteDeleted,
    WorkspaceOpened,
    WorkspaceClosed,
    SettingsChanged,
    PluginEnabled,
    PluginDisabled,
    BeforeExport,
    AfterExport,
    BeforeImport,
    AfterImport,
    Custom(String),
}

/// 插件清单文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub manifest_version: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub categories: Vec<PluginCategory>,
    pub main: String,
    pub engines: EngineRequirements,
    pub permissions: PluginPermissions,
    pub dependencies: Vec<PluginDependency>,
    pub scripts: Option<PluginScripts>,
    pub files: Vec<String>,
}

/// 引擎要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequirements {
    pub zeno: String,
    pub tauri: Option<String>,
    pub node: Option<String>,
}

/// 插件脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScripts {
    pub preinstall: Option<String>,
    pub postinstall: Option<String>,
    pub preuninstall: Option<String>,
    pub postuninstall: Option<String>,
    pub test: Option<String>,
    pub build: Option<String>,
}

/// 插件市场信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub downloads: u64,
    pub rating: f32,
    pub rating_count: u32,
    pub last_updated: DateTime<Utc>,
    pub size: u64,
    pub screenshots: Vec<String>,
    pub tags: Vec<String>,
    pub compatible_versions: Vec<String>,
    pub download_url: String,
    pub verified: bool,
}

/// 插件安装选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallOptions {
    pub force: bool,
    pub skip_dependencies: bool,
    pub dev_mode: bool,
    pub custom_install_path: Option<String>,
}

/// 插件更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub changelog: String,
    pub breaking_changes: bool,
    pub download_url: String,
    pub size: u64,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
            shortcuts: HashMap::new(),
            ui_config: None,
            auto_enable: true,
            update_check: true,
        }
    }
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            file_system: FileSystemPermissions {
                read_workspace: false,
                write_workspace: false,
                read_system: false,
                write_system: false,
                execute: false,
                allowed_paths: Vec::new(),
                denied_paths: Vec::new(),
            },
            network: NetworkPermissions {
                http_request: false,
                websocket: false,
                allowed_domains: Vec::new(),
                denied_domains: Vec::new(),
            },
            ui: UIPermissions {
                modify_ui: false,
                show_notifications: false,
                create_dialogs: false,
                access_clipboard: false,
            },
            system: SystemPermissions {
                execute_commands: false,
                access_env_vars: false,
                create_processes: false,
                access_hardware: false,
            },
        }
    }
}

impl Plugin {
    pub fn new(manifest: PluginManifest, install_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            author: manifest.author,
            homepage: manifest.homepage,
            repository: manifest.repository,
            license: manifest.license,
            keywords: manifest.keywords,
            categories: manifest.categories,
            main: manifest.main,
            manifest_path: format!("{}/plugin.json", install_path),
            install_path: install_path.clone(),
            enabled: false,
            config: PluginConfig::default(),
            permissions: manifest.permissions,
            dependencies: manifest.dependencies,
            installed_at: now,
            last_updated: now,
            checksum: String::new(), // 将在安装时计算
        }
    }

    pub fn is_compatible(&self, zeno_version: &str) -> bool {
        // 简化的版本兼容性检查
        // 实际实现应该使用 semver 库进行更精确的版本比较
        true
    }

    pub fn get_setting<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.config.settings.get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    pub fn set_setting<T>(&mut self, key: &str, value: T)
    where
        T: serde::Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.config.settings.insert(key.to_string(), json_value);
        }
    }
}