use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::error::WordCliError;
use crate::package::DocxPackage;

const MINIMAL_DOCX_BYTES: &[u8] = include_bytes!("../assets/blank.docx");

#[derive(Debug, Serialize)]
pub struct LifecycleResponse {
    pub status: String,
    pub command: String,
    pub target_file: String,
    pub message: String,
}

/// 新建合规文档（支持纯净空白或基于模板派生）
pub fn create_document(
    target_path: &str,
    template_path: Option<&str>,
) -> Result<LifecycleResponse, WordCliError> {
    if let Some(parent) = Path::new(target_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                WordCliError::file_io(format!("创建目标文件所在目录失败: {}", e))
            })?;
        }
    }

    let pkg = if let Some(tpl) = template_path {
        DocxPackage::from_file(tpl)?
    } else {
        DocxPackage::from_bytes(MINIMAL_DOCX_BYTES)?
    };

    pkg.save_to_file(target_path)?;

    Ok(LifecycleResponse {
        status: "success".to_string(),
        command: "create".to_string(),
        target_file: target_path.to_string(),
        message: if template_path.is_some() {
            "已成功从模板派生新文档".to_string()
        } else {
            "已成功创建极简合规标准空白文档".to_string()
        },
    })
}

/// 另存为副本
pub fn save_as(source_path: &str, target_path: &str) -> Result<LifecycleResponse, WordCliError> {
    let pkg = DocxPackage::from_file(source_path)?;
    pkg.save_to_file(target_path)?;

    Ok(LifecycleResponse {
        status: "success".to_string(),
        command: "save-as".to_string(),
        target_file: target_path.to_string(),
        message: format!("已成功将文档另存为 {}", target_path),
    })
}