use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("文件操作错误: {0}")]
    FileError(String),
    
    #[error("解析错误: {0}")]
    ParseError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ParseError(err.to_string())
    }
}