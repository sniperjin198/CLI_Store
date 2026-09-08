use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: String,
    pub command: String,
    pub data: Option<T>,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(command: impl Into<String>, data: T, message: impl Into<String>) -> Self {
        Self {
            status: "success".to_string(),
            command: command.into(),
            data: Some(data),
            message: message.into(),
        }
    }

    pub fn print_and_exit(&self) -> ! {
        let json_str = serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"status":"error","command":"unknown","data":null,"message":"JSON 序列化严重故障"}"#.to_string()
        });
        println!("{}", json_str);

        if self.status == "success" {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }
}

impl ApiResponse<()> {
    pub fn fail(command: impl Into<String>, message: impl Into<String>) -> Self {
        ApiResponse {
            status: "error".to_string(),
            command: command.into(),
            data: None,
            message: message.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("工作表未找到: {0}")]
    SheetNotFound(String),

    #[error("无效的单元格或区域坐标: {0}")]
    InvalidCoordinate(String),

    #[error("Calamine 读取错误: {0}")]
    Calamine(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    General(String),
}