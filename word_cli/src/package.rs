// OpenXML (docx) Package Management
// Provides zip-based extraction, file I/O, and repackaging for DOCX files

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::error::WordCliError;

pub struct DocxPackage {
    pub entries: HashMap<String, Vec<u8>>,
}

impl DocxPackage {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, WordCliError> {
        let p_ref = path.as_ref();
        if !p_ref.exists() {
            return Err(WordCliError::file_not_found(format!(
                "指定的目标文件不存在: {}",
                p_ref.display()
            )));
        }

        let file = File::open(p_ref).map_err(|e| {
            WordCliError::file_not_found(format!("无法打开文件 {}: {}", p_ref.display(), e))
        })?;

        let mut archive = ZipArchive::new(file)?;
        let mut entries = HashMap::new();

        for i in 0..archive.len() {
            let mut file_in_zip = archive.by_index(i)?;
            let mut contents = Vec::new();
            file_in_zip.read_to_end(&mut contents)?;
            entries.insert(file_in_zip.name().to_string(), contents);
        }

        let pkg = DocxPackage { entries };
        // 关键点 1：文档合法性基本防线校验
        pkg.validate_structure()?;
        Ok(pkg)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WordCliError> {
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader).map_err(|e| {
            WordCliError::corrupted_document(format!("无法解析内置模板压缩包：{}", e))
        })?;

        let mut entries = HashMap::new();
        for i in 0..archive.len() {
            let mut file_entry = archive.by_index(i).map_err(|e| {
                WordCliError::corrupted_document(format!("读取压缩包条目失败：{}", e))
            })?;

            if file_entry.is_file() {
                let mut content = Vec::new();
                file_entry.read_to_end(&mut content).map_err(|e| {
                    WordCliError::file_io(format!("读取条目内容失败：{}", e))
                })?;
                entries.insert(file_entry.name().to_string(), content);
            }
        }

        let pkg = Self { entries };
        pkg.validate_structure()?;
        Ok(pkg)
    }

    /// 校验是否属于合规的 DOCX 结构
    fn validate_structure(&self) -> Result<(), WordCliError> {
        if !self.entries.contains_key("word/document.xml") {
            return Err(WordCliError::corrupted_document(
                "非合规 DOCX 文档: 缺少核心组件 'word/document.xml'",
            ));
        }
        Ok(())
    }

    pub fn get_text(&self, path: &str) -> Result<String, WordCliError> {
        let bytes = self.entries.get(path).ok_or_else(|| {
            WordCliError::not_found(format!("文档中缺少必要组件：{}", path))
        })?;

        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| {
                WordCliError::corrupted_document(format!("XML 文本编码解析错误 ({}): {}", path, e))
            })
    }

    pub fn set_text(&mut self, path: &str, content: String) {
        self.entries.insert(path.to_string(), content.into_bytes());
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), WordCliError> {
        let target_path = path.as_ref();
        
        // 关键点 2：父目录深度递归创建与严谨的上下文报错
        if let Some(parent) = target_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    WordCliError::file_io(format!(
                        "创建目标文件父目录失败 ({:?}): {}",
                        parent, e
                    ))
                })?;
            }
        }

        let file = File::create(target_path).map_err(|e| {
            WordCliError::file_io(format!(
                "无法创建目标落盘文件 {:?}: {}",
                target_path, e
            ))
        })?;

        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for (name, data) in &self.entries {
            zip.start_file(name, options).map_err(|e| {
                WordCliError::file_io(format!("写入压缩包条目元数据失败 ({}): {}", name, e))
            })?;
            zip.write_all(data).map_err(|e| {
                WordCliError::file_io(format!("写入条目内容数据失败 ({}): {}", name, e))
            })?;
        }

        zip.finish().map_err(|e| {
            WordCliError::file_io(format!("完成 docx 最终封装打包失败：{}", e))
        })?;

        Ok(())
    }
}

/// 全局公用的 XML 实体安全转义工具
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}