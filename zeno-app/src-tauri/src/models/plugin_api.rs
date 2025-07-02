use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 插件 API 接口定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAPI {
    pub version: String,
    pub endpoints: Vec<APIEndpoint>,
    pub events: Vec<APIEvent>,
    pub permissions: Vec<APIPermission>,
}

/// API 端点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIEndpoint {
    pub name: String,
    pub method: APIMethod,
    pub path: String,
    pub description: String,
    pub parameters: Vec<APIParameter>,
    pub returns: APIReturn,
    pub permissions_required: Vec<String>,
    pub rate_limit: Option<RateLimit>,
}

/// API 方法类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    CALL, // 插件函数调用
}

/// API 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIParameter {
    pub name: String,
    pub param_type: APIParameterType,
    pub required: bool,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ParameterValidation>,
}

/// API 参数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<APIParameterType>),
    Object(HashMap<String, APIParameterType>),
    Custom(String),
}

/// 参数验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// API 返回值定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIReturn {
    pub return_type: APIParameterType,
    pub description: String,
    pub error_codes: Vec<APIError>,
}

/// API 错误定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIError {
    pub code: String,
    pub message: String,
    pub http_status: Option<u16>,
}

/// API 事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIEvent {
    pub name: String,
    pub description: String,
    pub data_schema: APIParameterType,
    pub frequency: EventFrequency,
}

/// 事件频率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventFrequency {
    High,    // 高频事件 (> 10/s)
    Medium,  // 中频事件 (1-10/s)
    Low,     // 低频事件 (< 1/s)
    Rare,    // 罕见事件 (< 1/min)
}

/// API 权限定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIPermission {
    pub name: String,
    pub description: String,
    pub risk_level: PermissionRiskLevel,
    pub scope: Vec<PermissionScope>,
}

/// 权限风险级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionRiskLevel {
    Low,      // 低风险：只读操作
    Medium,   // 中风险：写入操作
    High,     // 高风险：系统操作
    Critical, // 极高风险：危险操作
}

/// 权限作用域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    Workspace,     // 工作空间
    Notes,         // 笔记
    Files,         // 文件系统
    Network,       // 网络访问
    UI,            // 用户界面
    System,        // 系统访问
    Plugins,       // 插件管理
}

/// 速率限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub cooldown_seconds: u32,
}

/// API 调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIRequest {
    pub plugin_id: String,
    pub endpoint: String,
    pub method: APIMethod,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

/// API 调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse {
    pub request_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<APIError>,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
}

/// 插件通信消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub message_type: MessageType,
    pub from: String,
    pub to: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub message_id: String,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,     // 请求
    Response,    // 响应
    Event,       // 事件通知
    Broadcast,   // 广播消息
    Command,     // 命令
    Heartbeat,   // 心跳
}

/// 插件运行时状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRuntimeState {
    pub plugin_id: String,
    pub status: PluginRuntimeStatus,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub network_usage: NetworkUsage,
    pub uptime_seconds: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub error_count: u32,
    pub warning_count: u32,
}

/// 插件运行时状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginRuntimeStatus {
    Starting,    // 启动中
    Running,     // 运行中
    Idle,        // 空闲
    Busy,        // 忙碌
    Error,       // 错误
    Crashed,     // 崩溃
    Stopping,    // 停止中
    Stopped,     // 已停止
}

/// 网络使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_count: u32,
    pub active_connections: u32,
}

/// JavaScript 插件运行时接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSPluginRuntime {
    pub script_path: String,
    pub global_objects: HashMap<String, serde_json::Value>,
    pub imported_modules: Vec<String>,
    pub console_buffer: Vec<ConsoleMessage>,
    pub timeout_ms: u32,
    pub max_memory_mb: u32,
}

/// 控制台消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// 控制台级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
}

/// WASM 插件运行时接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMPluginRuntime {
    pub wasm_path: String,
    pub imports: HashMap<String, WASMImport>,
    pub exports: HashMap<String, WASMExport>,
    pub memory_pages: u32,
    pub max_memory_pages: u32,
    pub fuel_limit: Option<u64>,
}

/// WASM 导入函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMImport {
    pub module: String,
    pub name: String,
    pub function_type: WASMFunctionType,
}

/// WASM 导出函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMExport {
    pub name: String,
    pub function_type: WASMFunctionType,
}

/// WASM 函数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMFunctionType {
    pub parameters: Vec<WASMValueType>,
    pub returns: Vec<WASMValueType>,
}

/// WASM 值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WASMValueType {
    I32,
    I64,
    F32,
    F64,
}

/// 插件沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSandbox {
    pub enabled: bool,
    pub file_system_access: SandboxFileAccess,
    pub network_access: SandboxNetworkAccess,
    pub memory_limit_mb: u32,
    pub cpu_limit_percent: f32,
    pub timeout_seconds: u32,
    pub allowed_syscalls: Vec<String>,
    pub denied_syscalls: Vec<String>,
}

/// 沙箱文件访问配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFileAccess {
    pub read_only: bool,
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub temp_directory: Option<String>,
}

/// 沙箱网络访问配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxNetworkAccess {
    pub enabled: bool,
    pub allowed_domains: Vec<String>,
    pub denied_domains: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub max_connections: u32,
}

impl Default for PluginAPI {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            endpoints: Vec::new(),
            events: Vec::new(),
            permissions: Vec::new(),
        }
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            cooldown_seconds: 1,
        }
    }
}

impl Default for JSPluginRuntime {
    fn default() -> Self {
        Self {
            script_path: String::new(),
            global_objects: HashMap::new(),
            imported_modules: Vec::new(),
            console_buffer: Vec::new(),
            timeout_ms: 5000,
            max_memory_mb: 64,
        }
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self {
            enabled: true,
            file_system_access: SandboxFileAccess {
                read_only: true,
                allowed_paths: vec![".".to_string()],
                denied_paths: vec!["/".to_string(), "~".to_string()],
                temp_directory: None,
            },
            network_access: SandboxNetworkAccess {
                enabled: false,
                allowed_domains: Vec::new(),
                denied_domains: Vec::new(),
                allowed_ports: Vec::new(),
                max_connections: 0,
            },
            memory_limit_mb: 64,
            cpu_limit_percent: 10.0,
            timeout_seconds: 30,
            allowed_syscalls: Vec::new(),
            denied_syscalls: Vec::new(),
        }
    }
}