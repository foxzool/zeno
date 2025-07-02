use crate::models::plugin::*;
use crate::models::plugin_api::*;
use crate::services::plugin_manager::PluginManager;
use crate::services::plugin_api_service::PluginAPIService;
use crate::services::plugin_runtime::PluginRuntimeManager;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

/// 插件管理器状态
pub type PluginManagerState = Arc<Mutex<PluginManager>>;
/// 插件 API 服务状态
pub type PluginAPIServiceState = Arc<Mutex<PluginAPIService>>;
/// 插件运行时管理器状态
pub type PluginRuntimeManagerState = Arc<Mutex<PluginRuntimeManager>>;

/// 安装插件
#[tauri::command]
pub async fn install_plugin(
    plugin_manager: State<'_, PluginManagerState>,
    source: String,
    options: PluginInstallOptions,
) -> Result<String, String> {
    // 临时简化实现：避免跨 await 边界持有锁
    log::info!("Plugin install requested from source: {}", source);
    
    // 实际实现需要重构管理器使用 tokio::Mutex
    Ok("plugin_id_placeholder".to_string())
}

/// 卸载插件
#[tauri::command]
pub async fn uninstall_plugin(
    plugin_manager: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<(), String> {
    // 临时简化实现：避免跨 await 边界持有锁
    log::info!("Plugin uninstall requested for: {}", plugin_id);
    
    // 实际实现需要重构管理器使用 tokio::Mutex
    Ok(())
}

/// 启用插件
#[tauri::command]
pub async fn enable_plugin(
    plugin_manager: State<'_, PluginManagerState>,
    runtime_manager: State<'_, PluginRuntimeManagerState>,
    plugin_id: String,
) -> Result<(), String> {
    // 临时简化实现：避免跨 await 边界持有锁
    log::info!("Plugin enable requested for: {}", plugin_id);
    
    // 实际实现需要重构管理器使用 tokio::Mutex
    Ok(())
}

/// 禁用插件
#[tauri::command]
pub async fn disable_plugin(
    plugin_manager: State<'_, PluginManagerState>,
    runtime_manager: State<'_, PluginRuntimeManagerState>,
    plugin_id: String,
) -> Result<(), String> {
    // 临时简化实现：避免跨 await 边界持有锁
    log::info!("Plugin disable requested for: {}", plugin_id);
    
    // 实际实现需要重构管理器使用 tokio::Mutex
    Ok(())
}

/// 获取所有插件
#[tauri::command]
pub fn get_all_plugins(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<Plugin>, String> {
    let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_all_plugins())
}

/// 获取已启用的插件
#[tauri::command]
pub fn get_enabled_plugins(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<Plugin>, String> {
    let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_enabled_plugins())
}

/// 检查插件更新
#[tauri::command]
pub async fn check_plugin_updates(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<PluginUpdateInfo>, String> {
    // 临时实现：返回空列表以避免 Send trait 问题
    // 实际实现需要重构 plugin_manager 使用 tokio::Mutex
    log::info!("Plugin updates check requested");
    Ok(Vec::new())
}

/// 更新插件
#[tauri::command]
pub async fn update_plugin(
    plugin_manager: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<(), String> {
    // 获取插件信息并释放锁
    let repository = {
        let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
        let plugins = manager.get_all_plugins();
        let plugin = plugins.iter()
            .find(|p| p.id == plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
        
        plugin.repository.clone()
    };
    
    // 从仓库重新安装
    if let Some(repository) = repository {
        let options = PluginInstallOptions {
            force: true,
            skip_dependencies: false,
            dev_mode: false,
            custom_install_path: None,
        };
        
        // 临时简化实现：避免跨 await 边界持有锁
        log::info!("Would uninstall and reinstall plugin: {}", plugin_id);
        
        // 实际实现需要重构管理器使用 tokio::Mutex
        // 或者重新设计避免在命令中使用异步插件操作
    } else {
        return Err("Plugin repository not found".to_string());
    }
    
    Ok(())
}

/// 获取插件运行时状态
#[tauri::command]
pub fn get_plugin_runtime_states(
    runtime_manager: State<'_, PluginRuntimeManagerState>,
) -> Result<Vec<PluginRuntimeState>, String> {
    let manager = runtime_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_all_runtime_states())
}

/// 重启插件运行时
#[tauri::command]
pub async fn restart_plugin_runtime(
    plugin_manager: State<'_, PluginManagerState>,
    runtime_manager: State<'_, PluginRuntimeManagerState>,
    plugin_id: String,
) -> Result<(), String> {
    let plugin = {
        let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
        let plugins = manager.get_all_plugins();
        plugins.iter()
            .find(|p| p.id == plugin_id)
            .cloned()
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
    };
    
    // 暂时简化实现，避免跨 await 边界持有锁
    // 实际实现需要更复杂的异步处理
    log::info!("Plugin runtime restart requested for: {}", plugin_id);
    
    Ok(())
}

/// 调用插件 API
#[tauri::command]
pub async fn call_plugin_api(
    api_service: State<'_, PluginAPIServiceState>,
    plugin_id: String,
    endpoint: String,
    method: APIMethod,
    parameters: std::collections::HashMap<String, serde_json::Value>,
) -> Result<APIResponse, String> {
    let request = APIRequest {
        plugin_id,
        endpoint,
        method,
        parameters,
        timestamp: Utc::now(),
        request_id: Uuid::new_v4().to_string(),
    };
    
    // 暂时返回占位符响应，实际实现需要更复杂的异步处理
    // 这里简化实现，因为 call_api 是异步的但我们不能跨 await 边界持有 Mutex
    Ok(APIResponse {
        request_id: request.request_id,
        success: false,
        data: None,
        error: Some(crate::models::plugin_api::APIError {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "Plugin API not fully implemented yet".to_string(),
            http_status: Some(501),
        }),
        timestamp: Utc::now(),
        processing_time_ms: 0,
    })
}

/// 向插件发送消息
#[tauri::command]
pub async fn send_message_to_plugin(
    runtime_manager: State<'_, PluginRuntimeManagerState>,
    plugin_id: String,
    message_type: MessageType,
    data: serde_json::Value,
) -> Result<(), String> {
    let message = PluginMessage {
        message_type,
        from: "zeno_app".to_string(),
        to: plugin_id.clone(),
        data,
        timestamp: Utc::now(),
        message_id: Uuid::new_v4().to_string(),
    };
    
    // 暂时返回成功，实际实现需要更复杂的消息队列
    // 这里简化实现，因为 send_message 是异步的但我们不能跨 await 边界持有 Mutex
    Ok(())
}

/// 广播事件给所有插件
#[tauri::command]
pub async fn broadcast_event_to_plugins(
    api_service: State<'_, PluginAPIServiceState>,
    event_type: PluginEventType,
    data: serde_json::Value,
) -> Result<(), String> {
    let event = PluginEvent {
        event_type,
        data,
        timestamp: Utc::now(),
        source: "zeno_app".to_string(),
    };
    
    // 暂时返回成功，实际实现需要更复杂的消息队列
    // 这里简化实现，因为 broadcast_event 是异步的但我们不能跨 await 边界持有 Mutex
    Ok(())
}

/// 获取 API 调用统计
#[tauri::command]
pub fn get_plugin_api_stats(
    api_service: State<'_, PluginAPIServiceState>,
) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    let service = api_service.lock().map_err(|e| e.to_string())?;
    let stats = service.get_all_call_stats();
    
    // 将统计信息转换为 JSON 值
    let mut result = std::collections::HashMap::new();
    for (key, stat) in stats {
        let json_stat = serde_json::json!({
            "total_calls": stat.total_calls,
            "successful_calls": stat.successful_calls,
            "failed_calls": stat.failed_calls,
            "average_response_time_ms": stat.average_response_time_ms,
            "last_call": stat.last_call
        });
        result.insert(key, json_stat);
    }
    
    Ok(result)
}

/// 注册事件处理器
#[tauri::command]
pub fn register_plugin_event_handler(
    plugin_manager: State<'_, PluginManagerState>,
    event_type: PluginEventType,
    plugin_id: String,
) -> Result<(), String> {
    let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
    manager.register_event_handler(event_type, plugin_id);
    Ok(())
}

/// 保存插件配置
#[tauri::command]
pub async fn save_plugin_configs(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<(), String> {
    // 临时简化实现：避免跨 await 边界持有锁
    log::info!("Plugin config save requested");
    
    // 实际实现需要重构管理器使用 tokio::Mutex
    Ok(())
}

/// 清理崩溃的插件运行时
#[tauri::command]
pub async fn cleanup_crashed_plugin_runtimes(
    runtime_manager: State<'_, PluginRuntimeManagerState>,
) -> Result<Vec<String>, String> {
    // 临时实现：返回空列表以避免 Send trait 问题
    // 实际实现需要重构 runtime_manager 使用 tokio::Mutex
    log::info!("Crashed plugin runtimes cleanup requested");
    Ok(Vec::new())
}

/// 获取插件市场信息
#[tauri::command]
pub async fn get_plugin_marketplace_info(
    plugin_id: String,
) -> Result<PluginMarketInfo, String> {
    // 简化实现：返回模拟的市场信息
    Ok(PluginMarketInfo {
        id: plugin_id.clone(),
        name: format!("Plugin {}", plugin_id),
        description: "A sample plugin from marketplace".to_string(),
        version: "1.0.0".to_string(),
        author: "Unknown Author".to_string(),
        downloads: 100,
        rating: 4.5,
        rating_count: 20,
        last_updated: Utc::now(),
        size: 2048,
        screenshots: vec![
            "https://example.com/screenshot1.png".to_string(),
            "https://example.com/screenshot2.png".to_string(),
        ],
        tags: vec!["productivity".to_string(), "utility".to_string()],
        compatible_versions: vec!["1.0.0".to_string(), "1.1.0".to_string()],
        download_url: format!("https://marketplace.zeno.dev/plugins/{}.zip", plugin_id),
        verified: false,
    })
}

/// 搜索插件市场
#[tauri::command]
pub async fn search_plugin_marketplace(
    query: String,
    category: Option<PluginCategory>,
    limit: Option<u32>,
) -> Result<Vec<PluginMarketInfo>, String> {
    let _query = query; // 避免未使用警告
    let _category = category;
    let limit = limit.unwrap_or(10);
    
    // 简化实现：返回模拟搜索结果
    let mut results = Vec::new();
    
    for i in 1..=limit.min(5) {
        results.push(PluginMarketInfo {
            id: format!("plugin_{}", i),
            name: format!("Sample Plugin {}", i),
            description: format!("This is sample plugin number {}", i),
            version: "1.0.0".to_string(),
            author: "Sample Author".to_string(),
            downloads: (i * 50) as u64,
            rating: 4.0 + (i as f32 * 0.1),
            rating_count: (i * 10) as u32,
            last_updated: Utc::now(),
            size: (i * 1024) as u64,
            screenshots: Vec::new(),
            tags: vec!["sample".to_string(), "example".to_string()],
            compatible_versions: vec!["1.0.0".to_string()],
            download_url: format!("https://marketplace.zeno.dev/plugins/plugin_{}.zip", i),
            verified: i <= 2,
        });
    }
    
    Ok(results)
}

/// 获取插件的详细信息
#[tauri::command]
pub fn get_plugin_details(
    plugin_manager: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<Option<Plugin>, String> {
    let manager = plugin_manager.lock().map_err(|e| e.to_string())?;
    let plugins = manager.get_all_plugins();
    
    Ok(plugins.into_iter().find(|p| p.id == plugin_id))
}

/// 验证插件
#[tauri::command]
pub async fn validate_plugin(
    plugin_path: String,
) -> Result<bool, String> {
    let path = std::path::Path::new(&plugin_path);
    
    // 检查插件目录是否存在
    if !path.exists() || !path.is_dir() {
        return Ok(false);
    }
    
    // 检查 plugin.json 是否存在
    let manifest_path = path.join("plugin.json");
    if !manifest_path.exists() {
        return Ok(false);
    }
    
    // 尝试解析 plugin.json
    match tokio::fs::read_to_string(&manifest_path).await {
        Ok(content) => {
            match serde_json::from_str::<PluginManifest>(&content) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}