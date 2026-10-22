use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error_type: String,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum WebScrapeError {
    Io(String),
    Network(String),
    Timeout(String),
    HttpStatus(u16, String),
    ParseError(String),
    InvalidParameter(String),
    NotFound(String),
    Unsupported(String),
}

#[allow(dead_code)]
impl WebScrapeError {
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    pub fn http_status(code: u16, msg: impl Into<String>) -> Self {
        Self::HttpStatus(code, msg.into())
    }

    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::InvalidParameter(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let (error_type, message) = match self {
            Self::Io(m) => ("IoError", m.clone()),
            Self::Network(m) => ("NetworkError", m.clone()),
            Self::Timeout(m) => ("TimeoutError", m.clone()),
            Self::HttpStatus(code, m) => ("HttpStatusError", format!("[Code {}] {}", code, m)),
            Self::ParseError(m) => ("ParseError", m.clone()),
            Self::InvalidParameter(m) => ("InvalidParameterError", m.clone()),
            Self::NotFound(m) => ("NotFoundError", m.clone()),
            Self::Unsupported(m) => ("UnsupportedOperationError", m.clone()),
        };

        ErrorResponse {
            status: "error".to_string(),
            error_type: error_type.to_string(),
            message,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.to_error_response())
            .unwrap_or_else(|_| r#"{"status":"error","error_type":"SerializationError","message":"序列化错误响应失败"}"#.to_string())
    }
}

impl fmt::Display for WebScrapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl std::error::Error for WebScrapeError {}

impl From<std::io::Error> for WebScrapeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<reqwest::Error> for WebScrapeError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(format!("请求超时: {}", err))
        } else {
            Self::Network(format!("网络请求异常: {}", err))
        }
    }
}

impl From<url::ParseError> for WebScrapeError {
    fn from(err: url::ParseError) -> Self {
        Self::ParseError(format!("URL 解析错误: {}", err))
    }
}