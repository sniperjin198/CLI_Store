use lopdf::{Dictionary, Document, Object};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::error::PdfCliError;
use crate::io_channel::ensure_output_dir;

#[derive(Debug, Serialize)]
pub struct TransformResponse {
    pub status: String,
    pub action: String,
    pub output_file: String,
    pub input_files_count: usize,
    pub affected_pages: usize,
    pub message: String,
}

/// 物理合并：接收通用 io_channel 解析好的有效 PathBuf 列表
pub fn merge_resolved_pdfs(
    target_paths: &[PathBuf],
    output_file: &str,
) -> Result<TransformResponse, PdfCliError> {
    if target_paths.len() < 2 {
        return Err(PdfCliError::invalid_parameter(format!(
            "合并操作至少需要提供 2 个有效 PDF 文档，当前仅定位到 {} 个输入文件",
            target_paths.len()
        )));
    }

    let out_path = Path::new(output_file);
    ensure_output_dir(out_path)?;

    let mut target_doc = Document::with_version("1.5");
    let mut total_merged_pages = 0;

    let catalog_id = target_doc.new_object_id();
    let pages_id = target_doc.new_object_id();
    let target_pages_id = pages_id;
    let mut page_tree_kids: Vec<Object> = Vec::new();

    for file_path in target_paths {
        let path_str = file_path.to_string_lossy().to_string();
        let mut source_doc = Document::load(file_path).map_err(|e| {
            PdfCliError::corrupted_document(format!("无法加载输入文件 '{}': {}", path_str, e))
        })?;

        source_doc.renumber_objects_with(target_doc.max_id + 1);

        let source_pages = source_doc.get_pages();
        total_merged_pages += source_pages.len();

        for (_num, &page_id) in &source_pages {
            if let Ok(page_obj) = source_doc.get_object_mut(page_id) {
                if let Ok(page_dict) = page_obj.as_dict_mut() {
                    page_dict.set("Parent", pages_id);
                }
            }
            page_tree_kids.push(Object::Reference(page_id));
        }

        for (id, obj) in source_doc.objects {
            target_doc.objects.insert(id, obj);
        }

        target_doc.max_id = target_doc.objects.keys().map(|id| id.0).max().unwrap_or(target_doc.max_id);
    }

    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", "Pages");
    pages_dict.set("Kids", Object::Array(page_tree_kids));
    pages_dict.set("Count", total_merged_pages as i64);
    target_doc.objects.insert(target_pages_id, Object::Dictionary(pages_dict));

    let mut catalog_dict = Dictionary::new();
    catalog_dict.set("Type", "Catalog");
    catalog_dict.set("Pages", pages_id);
    target_doc.objects.insert(catalog_id, Object::Dictionary(catalog_dict));
    target_doc.trailer.set("Root", catalog_id);

    target_doc.prune_objects();
    target_doc.save(out_path).map_err(|e| {
        PdfCliError::io(format!("保存合并 PDF 失败 '{}': {}", output_file, e))
    })?;

    Ok(TransformResponse {
        status: "success".to_string(),
        action: "merge".to_string(),
        output_file: output_file.replace('\\', "/"),
        input_files_count: target_paths.len(),
        affected_pages: total_merged_pages,
        message: format!("已成功将 {} 个文档合并为 {} 页交付件", target_paths.len(), total_merged_pages),
    })
}

/// 页面抽取：从源文档抽取指定页并存为新 PDF
pub fn extract_pages(
    file_path: &str,
    range_expr: &str,
    output_file: &str,
) -> Result<TransformResponse, PdfCliError> {
    let out_path = Path::new(output_file);
    ensure_output_dir(out_path)?;

    let mut doc = Document::load(file_path).map_err(|e| {
        PdfCliError::corrupted_document(format!("无法加载 PDF 文档 '{}': {}", file_path, e))
    })?;

    let all_pages = doc.get_pages();
    let total_pages = all_pages.len();

    let target_page_nums = parse_range(range_expr, total_pages)?;
    if target_page_nums.is_empty() {
        return Err(PdfCliError::invalid_parameter(format!(
            "页码范围 '{}' 未命中任何有效页面",
            range_expr
        )));
    }

    let to_remove: Vec<u32> = all_pages
        .keys()
        .filter(|&&num| !target_page_nums.contains(&(num as usize)))
        .copied()
        .collect();

    doc.delete_pages(&to_remove);
    doc.prune_objects();

    doc.save(out_path).map_err(|e| {
        PdfCliError::io(format!("保存抽取页面文件 '{}': {}", output_file, e))
    })?;

    Ok(TransformResponse {
        status: "success".to_string(),
        action: "extract-pages".to_string(),
        output_file: output_file.replace('\\', "/"),
        input_files_count: 1,
        affected_pages: target_page_nums.len(),
        message: format!("已成功提取第 {:?} 页到新文件", target_page_nums),
    })
}

/// 批量页面旋转：支持单文件或多文件（--dir）批量校正
pub fn rotate_pages_batch(
    target_paths: &[PathBuf],
    range: Option<&str>,
    degrees: i32,
    output_file: Option<&str>,
) -> Result<TransformResponse, PdfCliError> {
    if degrees % 90 != 0 {
        return Err(PdfCliError::invalid_parameter("旋转角度必须为 90 的整数倍（例如 90, 180, 270）"));
    }

    if target_paths.is_empty() {
        return Err(PdfCliError::invalid_parameter("未提供任何需要旋转的 PDF 文件"));
    }

    let mut total_affected_pages = 0;
    let is_batch = target_paths.len() > 1;

    for file_path in target_paths {
        let path_str = file_path.to_string_lossy().to_string();
        let mut doc = Document::load(file_path).map_err(|e| {
            PdfCliError::corrupted_document(format!("无法加载 PDF 文档 '{}': {}", path_str, e))
        })?;

        let all_pages = doc.get_pages();
        let total_pages = all_pages.len();

        let target_page_nums = if let Some(r) = range {
            parse_range(r, total_pages)?
        } else {
            (1..=total_pages).collect()
        };

        let mut current_file_rotated = 0;
        for page_num in &target_page_nums {
            if let Some(&page_id) = all_pages.get(&(*page_num as u32)) {
                if let Ok(page_dict) = doc.get_dictionary_mut(page_id) {
                    let current_rot = page_dict
                        .get(b"Rotate")
                        .and_then(Object::as_i64)
                        .unwrap_or(0) as i32;

                    let mut next_rot = (current_rot + degrees) % 360;
                    if next_rot < 0 {
                        next_rot += 360;
                    }

                    page_dict.set("Rotate", next_rot);
                    current_file_rotated += 1;
                }
            }
        }

        // 多文件批处理时默认覆盖各自源文件；单文件时支持 output 指定另存路径
        let save_target = if !is_batch && output_file.is_some() {
            output_file.unwrap().to_string()
        } else {
            path_str
        };

        let save_path = Path::new(&save_target);
        ensure_output_dir(save_path)?;

        doc.save(save_path).map_err(|e| {
            PdfCliError::io(format!("保存旋转文件失败 '{}': {}", save_target, e))
        })?;

        total_affected_pages += current_file_rotated;
    }

    let display_output = if is_batch {
        format!("批量就地修改 {} 个文件", target_paths.len())
    } else if let Some(out) = output_file {
        out.replace('\\', "/")
    } else {
        target_paths[0].to_string_lossy().replace('\\', "/")
    };

    Ok(TransformResponse {
        status: "success".to_string(),
        action: "rotate".to_string(),
        output_file: display_output,
        input_files_count: target_paths.len(),
        affected_pages: total_affected_pages,
        message: format!("已成功完成 {} 个文档共 {} 页的 {} 度旋转校正", target_paths.len(), total_affected_pages, degrees),
    })
}

fn parse_range(expr: &str, total_pages: usize) -> Result<Vec<usize>, PdfCliError> {
    if expr.eq_ignore_ascii_case("all") {
        return Ok((1..=total_pages).collect());
    }

    if let Some((start_s, end_s)) = expr.split_once(':') {
        let start = start_s.parse::<usize>().unwrap_or(1).max(1);
        let end = end_s.parse::<usize>().unwrap_or(total_pages).min(total_pages);
        if start > end {
            return Err(PdfCliError::invalid_parameter(format!(
                "无效范围: 起始页 {} 大于 终止页 {}",
                start, end
            )));
        }
        return Ok((start..=end).collect());
    }

    if let Ok(single) = expr.parse::<usize>() {
        if single == 0 || single > total_pages {
            return Err(PdfCliError::invalid_parameter(format!("页码 {} 越界", single)));
        }
        return Ok(vec![single]);
    }

    Err(PdfCliError::invalid_parameter(format!("无法解析的页码范围表达式: '{}'", expr)))
}