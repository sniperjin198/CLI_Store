use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error_type: String,
    pub message: String,
}

#[derive(Debug)]
pub enum WordCliError {
    FileNotFound(String),
    CorruptedDocument(String),
    XmlParseError(String),
    NotFound(String),
    InvalidParameter(String),
    ExecutionError(String),
    IoError(std::io::Error),
}

impl WordCliError {
    pub fn file_not_found<S: Into<String>>(msg: S) -> Self {
        WordCliError::FileNotFound(msg.into())
    }

    pub fn corrupted_document<S: Into<String>>(msg: S) -> Self {
        WordCliError::CorruptedDocument(msg.into())
    }

    pub fn xml_parse<S: Into<String>>(msg: S) -> Self {
        WordCliError::XmlParseError(msg.into())
    }

    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        WordCliError::NotFound(msg.into())
    }

    pub fn invalid_parameter<S: Into<String>>(msg: S) -> Self {
        WordCliError::InvalidParameter(msg.into())
    }

    pub fn execution_error<S: Into<String>>(msg: S) -> Self {
        WordCliError::ExecutionError(msg.into())
    }

    pub fn file_io<S: Into<String>>(msg: S) -> Self {
        WordCliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }

    pub fn to_json(&self) -> String {
        let (err_type, message) = match self {
            WordCliError::FileNotFound(m) => ("FileNotFound", m.clone()),
            WordCliError::CorruptedDocument(m) => ("CorruptedDocument", m.clone()),
            WordCliError::XmlParseError(m) => ("XmlParseError", m.clone()),
            WordCliError::NotFound(m) => ("NotFound", m.clone()),
            WordCliError::InvalidParameter(m) => ("InvalidParameter", m.clone()),
            WordCliError::ExecutionError(m) => ("ExecutionError", m.clone()),
            WordCliError::IoError(e) => ("IoError", e.to_string()),
        };

        serde_json::to_string_pretty(&ErrorResponse {
            status: "error".to_string(),
            error_type: err_type.to_string(),
            message,
        })
        .unwrap_or_else(|_| r#"{"status":"error","error_type":"SerializationError","message":"内部错误序列化失败"}"#.to_string())
    }
}

impl fmt::Display for WordCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl std::error::Error for WordCliError {}

impl From<std::io::Error> for WordCliError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            WordCliError::file_not_found(err.to_string())
        } else {
            WordCliError::execution_error(format!("系统底层 I/O 执行异常: {}", err))
        }
    }
}

impl From<zip::result::ZipError> for WordCliError {
    fn from(err: zip::result::ZipError) -> Self {
        WordCliError::CorruptedDocument(format!("ZIP包解压或处理异常: {}", err))
    }
}