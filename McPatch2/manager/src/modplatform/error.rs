use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum PlatformError {
    AuthFailed(String),
    NetworkError(String),
    RateLimited(Duration),
    NotFound(String),
    ParseError(String),
    Internal(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::AuthFailed(msg) => write!(f, "认证失败: {}", msg),
            PlatformError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            PlatformError::RateLimited(wait) => write!(f, "请求被限流，建议等待 {:?}", wait),
            PlatformError::NotFound(msg) => write!(f, "未找到: {}", msg),
            PlatformError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            PlatformError::Internal(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {}
