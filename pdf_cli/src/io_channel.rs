use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use clap::Args;
use crate::error::PdfCliError;

const SUPPORTED_IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tiff", "tif"];

#[derive(Args, Debug, Clone)]
pub struct CommonInputArgs {
    /// 目标单一 PDF 文件
    #[arg(short, long)]
    pub file: Option<String>,

    /// 目标离散文件列表（逗号分隔）
    #[arg(long, value_delimiter = ',')]
    pub files: Option<Vec<String>>,

    /// 目标目录（自动自然排序扫描所有 .pdf 文件）
    #[arg(short, long)]
    pub dir: Option<String>,

    /// 文件清单文本（每行一个文件绝对路径）
    #[arg(short, long)]
    pub list: Option<String>,
}

impl CommonInputArgs {
    /// 解析出所有干净、合法的输入 PDF 文件绝对路径列表
    pub fn resolve_inputs(&self, exclude_output: Option<&str>) -> Result<Vec<PathBuf>, PdfCliError> {
        let mut candidates = Vec::new();

        // 1. 单文件模式
        if let Some(ref f) = self.file {
            candidates.push(clean_path_buf(f));
        }

        // 2. 离散多文件模式
        if let Some(ref list) = self.files {
            for item in list {
                candidates.push(clean_path_buf(item));
            }
        }

        // 3. 目录扫描模式 (Zero-Escaping)
        if let Some(ref d) = self.dir {
            let dir_buf = clean_path_buf(d);
            if !dir_buf.exists() || !dir_buf.is_dir() {
                return Err(PdfCliError::not_found(format!("指定的输入目录不存在: {}", dir_buf.display())));
            }
            let entries = fs::read_dir(&dir_buf).map_err(|e| {
                PdfCliError::io(format!("读取目录 '{}' 失败: {}", dir_buf.display(), e))
            })?;

            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |ext| ext.to_string_lossy().eq_ignore_ascii_case("pdf")) {
                    candidates.push(p);
                }
            }
            // 自然正序排序
            candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        }

        // 4. 清单文件模式 (Filelist)
        if let Some(ref l) = self.list {
            let list_path = clean_path_buf(l);
            if !list_path.exists() {
                return Err(PdfCliError::not_found(format!("指定的文件清单不存在: {}", list_path.display())));
            }
            let file = fs::File::open(&list_path).map_err(|e| PdfCliError::io(e.to_string()))?;
            for line in io::BufReader::new(file).lines().flatten() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    candidates.push(clean_path_buf(trimmed));
                }
            }
        }

        // 5. 过滤排除输出文件本身（防死循环/自我包含）
        let out_canonical = exclude_output.and_then(|o| {
            let p = clean_path_buf(o);
            p.canonicalize().ok()
        });

        let mut valid_files = Vec::new();
        for p in candidates {
            if let Ok(c_path) = p.canonicalize() {
                if let Some(ref out_can) = out_canonical {
                    if c_path == *out_can {
                        continue;
                    }
                }
                valid_files.push(c_path);
            } else if p.exists() {
                valid_files.push(p);
            }
        }

        if valid_files.is_empty() {
            return Err(PdfCliError::invalid_parameter("未定位到任何有效的输入 PDF 文件"));
        }

        Ok(valid_files)
    }

    /// 解析出所有图片文件（单文件、逗号隔开的多文件、目录扫描、txt 清单）
    pub fn resolve_image_inputs(&self) -> Result<Vec<PathBuf>, PdfCliError> {
        let mut candidates = Vec::new();

        // 1. 单文件模式
        if let Some(ref f) = self.file {
            candidates.push(clean_path_buf(f));
        }

        // 2. 离散列表模式 (--files a.jpg,b.png)
        if let Some(ref list) = self.files {
            for item in list {
                candidates.push(clean_path_buf(item));
            }
        }

        // 3. 目录扫描模式 (--dir ./photos)
        if let Some(ref d) = self.dir {
            let dir_buf = clean_path_buf(d);
            if !dir_buf.exists() || !dir_buf.is_dir() {
                return Err(PdfCliError::not_found(format!("指定的图片目录不存在: {}", dir_buf.display())));
            }
            let entries = fs::read_dir(&dir_buf).map_err(|e| {
                PdfCliError::io(format!("读取目录 '{}' 失败: {}", dir_buf.display(), e))
            })?;

            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if SUPPORTED_IMAGE_EXTS.iter().any(|&e| ext.eq_ignore_ascii_case(e)) {
                            candidates.push(p);
                        }
                    }
                }
            }
            // 自然顺序排序，确保打包成 PDF 时的页码顺序与文件名正序一致
            candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        }

        // 4. 清单文本文件模式 (--list img_list.txt)
        if let Some(ref l) = self.list {
            let list_path = clean_path_buf(l);
            if !list_path.exists() {
                return Err(PdfCliError::not_found(format!("指定的清单文件不存在: {}", list_path.display())));
            }
            let file = fs::File::open(&list_path).map_err(|e| PdfCliError::io(e.to_string()))?;
            for line in io::BufReader::new(file).lines().flatten() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    candidates.push(clean_path_buf(trimmed));
                }
            }
        }

        let mut valid_files = Vec::new();
        for p in candidates {
            if p.exists() && p.is_file() {
                valid_files.push(p);
            }
        }

        if valid_files.is_empty() {
            return Err(PdfCliError::invalid_parameter("未定位到任何有效的图片输入文件"));
        }

        Ok(valid_files)
    }
}

/// 路径清洁器：脱除一切多重包裹引号，规范跨平台反斜杠
pub fn clean_path_buf(raw: &str) -> PathBuf {
    let unquoted = raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace("\\\"", "\"");
    PathBuf::from(unquoted)
}

/// 保证输出目录必然存在
pub fn ensure_output_dir(target_file: &Path) -> Result<(), PdfCliError> {
    if let Some(parent) = target_file.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                PdfCliError::io(format!("无法创建输出目录 '{}': {}", parent.display(), e))
            })?;
        }
    }
    Ok(())
}