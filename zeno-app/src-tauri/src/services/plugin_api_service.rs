use crate::models::plugin::*;
use crate::models::plugin_api::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::Utc;
use uuid::Uuid;
use tokio::sync::mpsc;

/// 插件 API 服务
pub struct PluginAPIService {
    /// 已注册的 API 端点
    endpoints: RwLock<HashMap<String, APIEndpoint>>,
    /// 活跃的插件运行时
    runtimes: RwLock<HashMap<String, PluginRuntimeState>>,
    /// 消息通道
    message_sender: Arc<mpsc::UnboundedSender<PluginMessage>>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PluginMessage>>>>,
    /// 速率限制器
    rate_limiters: RwLock<HashMap<String, RateLimiter>>,
    /// API 调用统计
    call_stats: RwLock<HashMap<String, APICallStats>>,
}

/// 速率限制器
#[derive(Debug, Clone)]
struct RateLimiter {
    requests_per_minute: u32,
    burst_size: u32,
    current_requests: u32,
    last_reset: chrono::DateTime<Utc>,
    burst_tokens: u32,
}

/// API 调用统计
#[derive(Debug, Clone)]
pub struct APICallStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_response_time_ms: f64,
    pub last_call: chrono::DateTime<Utc>,
}

impl PluginAPIService {
    /// 创建新的插件 API 服务
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            endpoints: RwLock::new(HashMap::new()),
            runtimes: RwLock::new(HashMap::new()),
            message_sender: Arc::new(sender),
            message_receiver: Arc::new(RwLock::new(Some(receiver))),
            rate_limiters: RwLock::new(HashMap::new()),
            call_stats: RwLock::new(HashMap::new()),
        }
    }

    /// 注册 API 端点
    pub fn register_endpoint(&self, plugin_id: &str, endpoint: APIEndpoint) -> Result<()> {
        let endpoint_key = format!("{}:{}", plugin_id, endpoint.name);
        
        // 验证端点配置
        self.validate_endpoint(&endpoint)?;
        
        // 在移动 endpoint 之前获取需要的字段
        let endpoint_name = endpoint.name.clone();
        let rate_limit = endpoint.rate_limit.clone();
        
        // 注册端点
        {
            let mut endpoints = self.endpoints.write().unwrap();
            endpoints.insert(endpoint_key.clone(), endpoint);
        }
        
        // 初始化速率限制器
        if let Some(rate_limit) = rate_limit {
            let limiter = RateLimiter {
                requests_per_minute: rate_limit.requests_per_minute,
                burst_size: rate_limit.burst_size,
                current_requests: 0,
                last_reset: Utc::now(),
                burst_tokens: rate_limit.burst_size,
            };
            
            let mut limiters = self.rate_limiters.write().unwrap();
            limiters.insert(endpoint_key.clone(), limiter);
        }
        
        // 初始化统计
        {
            let mut stats = self.call_stats.write().unwrap();
            stats.insert(endpoint_key, APICallStats {
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                average_response_time_ms: 0.0,
                last_call: Utc::now(),
            });
        }
        
        log::info!("Registered API endpoint: {} for plugin {}", endpoint_name, plugin_id);
        Ok(())
    }

    /// 注销 API 端点
    pub fn unregister_endpoint(&self, plugin_id: &str, endpoint_name: &str) -> Result<()> {
        let endpoint_key = format!("{}:{}", plugin_id, endpoint_name);
        
        {
            let mut endpoints = self.endpoints.write().unwrap();
            endpoints.remove(&endpoint_key);
        }
        
        {
            let mut limiters = self.rate_limiters.write().unwrap();
            limiters.remove(&endpoint_key);
        }
        
        {
            let mut stats = self.call_stats.write().unwrap();
            stats.remove(&endpoint_key);
        }
        
        log::info!("Unregistered API endpoint: {} for plugin {}", endpoint_name, plugin_id);
        Ok(())
    }

    /// 调用 API 端点
    pub async fn call_api(&self, request: APIRequest) -> Result<APIResponse> {
        let start_time = std::time::Instant::now();
        let endpoint_key = format!("{}:{}", request.plugin_id, request.endpoint);
        
        // 检查端点是否存在
        let endpoint = {
            let endpoints = self.endpoints.read().unwrap();
            endpoints.get(&endpoint_key).cloned()
                .ok_or_else(|| anyhow!("API endpoint not found: {}", endpoint_key))?
        };
        
        // 验证权限
        self.validate_permissions(&request.plugin_id, &endpoint.permissions_required)?;
        
        // 检查速率限制
        self.check_rate_limit(&endpoint_key)?;
        
        // 验证参数
        self.validate_parameters(&request.parameters, &endpoint.parameters)?;
        
        // 执行 API 调用
        let result = self.execute_api_call(&request, &endpoint).await;
        
        // 更新统计
        let processing_time = start_time.elapsed().as_millis() as u64;
        self.update_call_stats(&endpoint_key, &result, processing_time);
        
        // 构建响应
        let response = match result {
            Ok(data) => APIResponse {
                request_id: request.request_id,
                success: true,
                data: Some(data),
                error: None,
                timestamp: Utc::now(),
                processing_time_ms: processing_time,
            },
            Err(e) => APIResponse {
                request_id: request.request_id,
                success: false,
                data: None,
                error: Some(APIError {
                    code: "EXECUTION_ERROR".to_string(),
                    message: e.to_string(),
                    http_status: Some(500),
                }),
                timestamp: Utc::now(),
                processing_time_ms: processing_time,
            },
        };
        
        Ok(response)
    }

    /// 发送消息给插件
    pub async fn send_message(&self, message: PluginMessage) -> Result<()> {
        self.message_sender.send(message)
            .map_err(|e| anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    /// 广播事件给所有插件
    pub async fn broadcast_event(&self, event: PluginEvent) -> Result<()> {
        let message = PluginMessage {
            message_type: MessageType::Event,
            from: "system".to_string(),
            to: "broadcast".to_string(),
            data: serde_json::to_value(&event)?,
            timestamp: Utc::now(),
            message_id: Uuid::new_v4().to_string(),
        };
        
        self.send_message(message).await
    }

    /// 注册插件运行时
    pub fn register_runtime(&self, plugin_id: String, runtime_state: PluginRuntimeState) {
        let mut runtimes = self.runtimes.write().unwrap();
        runtimes.insert(plugin_id, runtime_state);
    }

    /// 注销插件运行时
    pub fn unregister_runtime(&self, plugin_id: &str) {
        let mut runtimes = self.runtimes.write().unwrap();
        runtimes.remove(plugin_id);
    }

    /// 更新插件运行时状态
    pub fn update_runtime_state(&self, plugin_id: &str, status: PluginRuntimeStatus) -> Result<()> {
        let mut runtimes = self.runtimes.write().unwrap();
        
        if let Some(runtime) = runtimes.get_mut(plugin_id) {
            runtime.status = status;
            runtime.last_heartbeat = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Plugin runtime not found: {}", plugin_id))
        }
    }

    /// 获取插件运行时状态
    pub fn get_runtime_state(&self, plugin_id: &str) -> Option<PluginRuntimeState> {
        let runtimes = self.runtimes.read().unwrap();
        runtimes.get(plugin_id).cloned()
    }

    /// 获取所有运行时状态
    pub fn get_all_runtime_states(&self) -> Vec<PluginRuntimeState> {
        let runtimes = self.runtimes.read().unwrap();
        runtimes.values().cloned().collect()
    }

    /// 获取 API 调用统计
    pub fn get_call_stats(&self, endpoint_key: &str) -> Option<APICallStats> {
        let stats = self.call_stats.read().unwrap();
        stats.get(endpoint_key).cloned()
    }

    /// 获取所有 API 调用统计
    pub fn get_all_call_stats(&self) -> HashMap<String, APICallStats> {
        let stats = self.call_stats.read().unwrap();
        stats.clone()
    }

    /// 清理过期的运行时状态
    pub fn cleanup_expired_runtimes(&self, timeout_seconds: u64) {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(timeout_seconds as i64);
        
        let mut runtimes = self.runtimes.write().unwrap();
        runtimes.retain(|plugin_id, runtime| {
            let is_active = runtime.last_heartbeat > cutoff_time;
            if !is_active {
                log::warn!("Removing expired runtime for plugin: {}", plugin_id);
            }
            is_active
        });
    }

    // 私有方法实现

    /// 验证 API 端点配置
    fn validate_endpoint(&self, endpoint: &APIEndpoint) -> Result<()> {
        if endpoint.name.is_empty() {
            return Err(anyhow!("Endpoint name cannot be empty"));
        }
        
        if endpoint.path.is_empty() {
            return Err(anyhow!("Endpoint path cannot be empty"));
        }
        
        // 验证参数定义
        for param in &endpoint.parameters {
            if param.name.is_empty() {
                return Err(anyhow!("Parameter name cannot be empty"));
            }
        }
        
        Ok(())
    }

    /// 验证插件权限
    fn validate_permissions(&self, plugin_id: &str, required_permissions: &[String]) -> Result<()> {
        // 这里应该检查插件是否具有所需权限
        // 简化实现，实际应该与插件管理器集成
        
        if required_permissions.is_empty() {
            return Ok(());
        }
        
        log::debug!("Validating permissions for plugin {}: {:?}", plugin_id, required_permissions);
        
        // 占位符实现，实际应该检查插件的权限配置
        Ok(())
    }

    /// 检查速率限制
    fn check_rate_limit(&self, endpoint_key: &str) -> Result<()> {
        let mut limiters = self.rate_limiters.write().unwrap();
        
        if let Some(limiter) = limiters.get_mut(endpoint_key) {
            let now = Utc::now();
            
            // 检查是否需要重置计数器
            if (now - limiter.last_reset).num_seconds() >= 60 {
                limiter.current_requests = 0;
                limiter.burst_tokens = limiter.burst_size;
                limiter.last_reset = now;
            }
            
            // 检查是否超过速率限制
            if limiter.current_requests >= limiter.requests_per_minute {
                return Err(anyhow!("Rate limit exceeded for endpoint: {}", endpoint_key));
            }
            
            // 检查突发限制
            if limiter.burst_tokens == 0 {
                return Err(anyhow!("Burst limit exceeded for endpoint: {}", endpoint_key));
            }
            
            // 更新计数器
            limiter.current_requests += 1;
            limiter.burst_tokens = limiter.burst_tokens.saturating_sub(1);
        }
        
        Ok(())
    }

    /// 验证 API 参数
    fn validate_parameters(&self, params: &HashMap<String, serde_json::Value>, param_defs: &[APIParameter]) -> Result<()> {
        // 检查必需参数
        for param_def in param_defs {
            if param_def.required && !params.contains_key(&param_def.name) {
                return Err(anyhow!("Required parameter missing: {}", param_def.name));
            }
            
            // 验证参数类型和值
            if let Some(value) = params.get(&param_def.name) {
                self.validate_parameter_value(value, &param_def.param_type, &param_def.validation)?;
            }
        }
        
        Ok(())
    }

    /// 验证单个参数值
    fn validate_parameter_value(&self, value: &serde_json::Value, param_type: &APIParameterType, validation: &Option<ParameterValidation>) -> Result<()> {
        // 类型检查
        match param_type {
            APIParameterType::String => {
                if !value.is_string() {
                    return Err(anyhow!("Parameter should be a string"));
                }
            }
            APIParameterType::Integer => {
                if !value.is_i64() {
                    return Err(anyhow!("Parameter should be an integer"));
                }
            }
            APIParameterType::Float => {
                if !value.is_f64() {
                    return Err(anyhow!("Parameter should be a float"));
                }
            }
            APIParameterType::Boolean => {
                if !value.is_boolean() {
                    return Err(anyhow!("Parameter should be a boolean"));
                }
            }
            APIParameterType::Array(_) => {
                if !value.is_array() {
                    return Err(anyhow!("Parameter should be an array"));
                }
            }
            APIParameterType::Object(_) => {
                if !value.is_object() {
                    return Err(anyhow!("Parameter should be an object"));
                }
            }
            APIParameterType::Custom(_) => {
                // 自定义类型验证
            }
        }
        
        // 值验证
        if let Some(validation_rules) = validation {
            if let Some(ref allowed_values) = validation_rules.allowed_values {
                if !allowed_values.contains(value) {
                    return Err(anyhow!("Parameter value not in allowed list"));
                }
            }
            
            if let Some(pattern) = &validation_rules.pattern {
                if let Some(str_value) = value.as_str() {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                    if !regex.is_match(str_value) {
                        return Err(anyhow!("Parameter value does not match pattern"));
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 执行 API 调用
    async fn execute_api_call(&self, request: &APIRequest, endpoint: &APIEndpoint) -> Result<serde_json::Value> {
        // 这里应该根据端点的具体实现来执行调用
        // 可能需要调用插件的特定函数或向插件发送消息
        
        log::debug!("Executing API call: {} for plugin {}", endpoint.name, request.plugin_id);
        
        // 发送请求消息给插件
        let message = PluginMessage {
            message_type: MessageType::Request,
            from: "api_service".to_string(),
            to: request.plugin_id.clone(),
            data: serde_json::to_value(request)?,
            timestamp: Utc::now(),
            message_id: request.request_id.clone(),
        };
        
        self.send_message(message).await?;
        
        // 等待响应（简化实现）
        // 实际实现应该使用异步等待响应
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 返回占位符结果
        Ok(serde_json::json!({
            "result": "success",
            "endpoint": endpoint.name,
            "plugin_id": request.plugin_id
        }))
    }

    /// 更新 API 调用统计
    fn update_call_stats(&self, endpoint_key: &str, result: &Result<serde_json::Value>, processing_time_ms: u64) {
        let mut stats = self.call_stats.write().unwrap();
        
        if let Some(stat) = stats.get_mut(endpoint_key) {
            stat.total_calls += 1;
            stat.last_call = Utc::now();
            
            match result {
                Ok(_) => stat.successful_calls += 1,
                Err(_) => stat.failed_calls += 1,
            }
            
            // 更新平均响应时间（简单移动平均）
            let total_time = stat.average_response_time_ms * (stat.total_calls - 1) as f64 + processing_time_ms as f64;
            stat.average_response_time_ms = total_time / stat.total_calls as f64;
        }
    }
}

impl Default for PluginAPIService {
    fn default() -> Self {
        Self::new()
    }
}