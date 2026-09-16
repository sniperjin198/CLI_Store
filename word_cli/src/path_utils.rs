use std::fs;
use std::path::PathBuf;
use crate::error::WordCliError;

/// 清洗输入的路径字符串：脱除可能存在的多层外围引号、空白及转义字符
pub fn clean_path(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace("\\\"", "\"")
        .to_string()
}

/// 将任意输入转换为合规安全的 PathBuf，校验存在性
pub fn sanitize_existing_path(raw: &str) -> Result<PathBuf, WordCliError> {
    let cleaned = clean_path(raw);
    let p = PathBuf::from(&cleaned);
    if !p.exists() {
        return Err(WordCliError::file_not_found(format!(
            "指定的目标文件或路径不存在: {}",
            cleaned
        )));
    }
    Ok(p)
}

/// 确保目标输出文件的父目录一定存在，不存在则递归创建
pub fn ensure_output_parent_dir(raw: &str) -> Result<PathBuf, WordCliError> {
    let cleaned = clean_path(raw);
    let p = PathBuf::from(&cleaned);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                WordCliError::file_io(format!(
                    "创建输出目标父级目录 '{}' 失败: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }
    Ok(p)
}