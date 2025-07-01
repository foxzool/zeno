use thiserror::Error;

/// Zeno 核心错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML 解析错误: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("文件监控错误: {0}")]
    FileWatcher(#[from] notify::Error),

    #[error("UUID 解析错误: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("解析错误: {0}")]
    Parse(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("权限错误: {0}")]
    Permission(String),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("存储错误: {0}")]
    Storage(String),

    #[error("索引错误: {0}")]
    Index(String),

    #[error("发布错误: {0}")]
    Publisher(String),

    #[error("插件错误: {0}")]
    Plugin(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("功能未实现: {0}")]
    NotImplemented(String),

    #[error("操作被取消")]
    Cancelled,

    #[error("超时")]
    Timeout,

    #[error("多个错误: {0:?}")]
    Multiple(Vec<Error>),
}

impl Error {
    /// 创建解析错误
    pub fn parse<T: std::fmt::Display>(msg: T) -> Self {
        Self::Parse(msg.to_string())
    }

    /// 创建验证错误
    pub fn validation<T: std::fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    /// 创建配置错误
    pub fn config<T: std::fmt::Display>(msg: T) -> Self {
        Self::Config(msg.to_string())
    }

    /// 创建未找到错误
    pub fn not_found<T: std::fmt::Display>(msg: T) -> Self {
        Self::NotFound(msg.to_string())
    }

    /// 创建权限错误
    pub fn permission<T: std::fmt::Display>(msg: T) -> Self {
        Self::Permission(msg.to_string())
    }

    /// 创建网络错误
    pub fn network<T: std::fmt::Display>(msg: T) -> Self {
        Self::Network(msg.to_string())
    }

    /// 创建存储错误
    pub fn storage<T: std::fmt::Display>(msg: T) -> Self {
        Self::Storage(msg.to_string())
    }

    /// 创建索引错误
    pub fn index<T: std::fmt::Display>(msg: T) -> Self {
        Self::Index(msg.to_string())
    }

    /// 创建发布错误
    pub fn publisher<T: std::fmt::Display>(msg: T) -> Self {
        Self::Publisher(msg.to_string())
    }

    /// 创建插件错误
    pub fn plugin<T: std::fmt::Display>(msg: T) -> Self {
        Self::Plugin(msg.to_string())
    }

    /// 创建内部错误
    pub fn internal<T: std::fmt::Display>(msg: T) -> Self {
        Self::Internal(msg.to_string())
    }

    /// 创建未实现错误
    pub fn not_implemented<T: std::fmt::Display>(msg: T) -> Self {
        Self::NotImplemented(msg.to_string())
    }

    /// 检查是否为致命错误
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Error::Internal(_) | Error::Database(_) | Error::Io(_)
        )
    }

    /// 检查是否为用户错误
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Error::Validation(_) | Error::Config(_) | Error::Parse(_) | Error::NotFound(_)
        )
    }

    /// 检查是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::Timeout | Error::Database(_)
        )
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Io(_) => "IO_ERROR",
            Error::Database(_) => "DATABASE_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::Yaml(_) => "YAML_ERROR",
            Error::Toml(_) => "TOML_ERROR",
            Error::FileWatcher(_) => "FILE_WATCHER_ERROR",
            Error::Uuid(_) => "UUID_ERROR",
            Error::Parse(_) => "PARSE_ERROR",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Permission(_) => "PERMISSION_ERROR",
            Error::Network(_) => "NETWORK_ERROR",
            Error::Storage(_) => "STORAGE_ERROR",
            Error::Index(_) => "INDEX_ERROR",
            Error::Publisher(_) => "PUBLISHER_ERROR",
            Error::Plugin(_) => "PLUGIN_ERROR",
            Error::Internal(_) => "INTERNAL_ERROR",
            Error::NotImplemented(_) => "NOT_IMPLEMENTED",
            Error::Cancelled => "CANCELLED",
            Error::Timeout => "TIMEOUT",
            Error::Multiple(_) => "MULTIPLE_ERRORS",
        }
    }

    /// 获取HTTP状态码
    pub fn http_status(&self) -> u16 {
        match self {
            Error::NotFound(_) => 404,
            Error::Permission(_) => 403,
            Error::Validation(_) | Error::Parse(_) => 400,
            Error::Internal(_) | Error::Database(_) => 500,
            Error::Network(_) => 502,
            Error::Timeout => 504,
            Error::NotImplemented(_) => 501,
            _ => 500,
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 错误上下文扩展
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    fn context(self, msg: &str) -> Result<T>;
}

impl<T> ErrorContext<T> for Result<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| match e {
            Error::Multiple(mut errors) => {
                errors.push(Error::internal(f()));
                Error::Multiple(errors)
            }
            _ => Error::Multiple(vec![e, Error::internal(f())]),
        })
    }

    fn context(self, msg: &str) -> Result<T> {
        self.with_context(|| msg.to_string())
    }
}


/// 错误收集器
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<Error>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn add(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn add_result<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.add(error);
                None
            }
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else if self.errors.len() == 1 {
            Err(self.errors.into_iter().next().unwrap())
        } else {
            Err(Error::Multiple(self.errors))
        }
    }

    pub fn finish(self) -> Result<()> {
        self.into_result(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::parse("测试解析错误");
        assert_eq!(error.error_code(), "PARSE_ERROR");
        assert_eq!(error.http_status(), 400);
        assert!(error.is_user_error());
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(Error::parse("原始错误"));
        let result_with_context = result.context("添加上下文");
        
        assert!(result_with_context.is_err());
        if let Err(Error::Multiple(errors)) = result_with_context {
            assert_eq!(errors.len(), 2);
        } else {
            panic!("期望 Multiple 错误");
        }
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();
        
        collector.add(Error::parse("错误1"));
        collector.add(Error::validation("错误2"));
        
        assert!(collector.has_errors());
        assert_eq!(collector.len(), 2);
        
        let result = collector.finish();
        assert!(result.is_err());
        
        if let Err(Error::Multiple(errors)) = result {
            assert_eq!(errors.len(), 2);
        }
    }

    #[test]
    fn test_error_properties() {
        let network_error = Error::network("连接失败");
        assert!(network_error.is_retryable());
        assert_eq!(network_error.http_status(), 502);
        
        let validation_error = Error::validation("无效输入");
        assert!(validation_error.is_user_error());
        assert!(!validation_error.is_retryable());
        
        let internal_error = Error::internal("系统错误");
        assert!(internal_error.is_fatal());
        assert_eq!(internal_error.http_status(), 500);
    }
}