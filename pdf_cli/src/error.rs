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
pub enum PdfCliError {
    Io(String),
    CorruptedDocument(String),
    EncryptedPdf(String),
    InvalidParameter(String),
    NotFound(String),
    FontMappingFailed(String),
    LayoutAnalysisFailed(String),
    UnsupportedOperation(String),
}

#[allow(dead_code)]
impl PdfCliError {
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    pub fn corrupted_document(msg: impl Into<String>) -> Self {
        Self::CorruptedDocument(msg.into())
    }

    pub fn encrypted_pdf(msg: impl Into<String>) -> Self {
        Self::EncryptedPdf(msg.into())
    }

    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::InvalidParameter(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn font_mapping(msg: impl Into<String>) -> Self {
        Self::FontMappingFailed(msg.into())
    }

    pub fn layout_failed(msg: impl Into<String>) -> Self {
        Self::LayoutAnalysisFailed(msg.into())
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::UnsupportedOperation(msg.into())
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let (error_type, message) = match self {
            Self::Io(m) => ("IoError", m.clone()),
            Self::CorruptedDocument(m) => ("CorruptedDocumentError", m.clone()),
            Self::EncryptedPdf(m) => ("EncryptedPdfError", m.clone()),
            Self::InvalidParameter(m) => ("InvalidParameterError", m.clone()),
            Self::NotFound(m) => ("NotFoundError", m.clone()),
            Self::FontMappingFailed(m) => ("FontMappingError", m.clone()),
            Self::LayoutAnalysisFailed(m) => ("LayoutAnalysisError", m.clone()),
            Self::UnsupportedOperation(m) => ("UnsupportedOperationError", m.clone()),
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

impl fmt::Display for PdfCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl std::error::Error for PdfCliError {}

impl From<std::io::Error> for PdfCliError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<lopdf::Error> for PdfCliError {
    fn from(err: lopdf::Error) -> Self {
        let err_str = err.to_string();
        if err_str.to_lowercase().contains("password") || err_str.to_lowercase().contains("encrypted") {
            Self::EncryptedPdf(format!("文档已加密或受密码保护: {}", err_str))
        } else {
            Self::CorruptedDocument(format!("PDF 解析异常: {}", err_str))
        }
    }
}