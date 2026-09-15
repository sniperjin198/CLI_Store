use lopdf::content::Content;
use lopdf::{Document, Object, ObjectId};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::cmap::CMapTable;
use crate::error::PdfCliError;
use crate::inspect::obj_to_f64;

#[derive(Debug, Serialize, Clone)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TextBlock {
    pub text: String,
    pub bbox: BBox,
    pub font_size: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TableColumn {
    pub col_index: usize,
    pub x_start: f64,
    pub x_end: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct StructuredRow {
    pub y: f64,
    pub indent_level: usize,
    pub cells: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OverlapAnomaly {
    pub text_a: String,
    pub text_b: String,
    pub bbox: BBox,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct InspectLayoutResponse {
    pub status: String,
    pub file: String,
    pub page: usize,
    pub total_text_blocks: usize,
    pub detected_columns: Vec<TableColumn>,
    pub structured_rows: Vec<StructuredRow>,
    pub overlap_anomalies: Vec<OverlapAnomaly>,
}

pub fn inspect_page_layout(file_path: &str, page_num: usize) -> Result<InspectLayoutResponse, PdfCliError> {
    let doc = Document::load(file_path).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("password") || msg.to_lowercase().contains("encrypted") {
            PdfCliError::encrypted_pdf(format!("文档受密码保护: {}", msg))
        } else {
            PdfCliError::corrupted_document(format!("无法加载 PDF 文档: {}", msg))
        }
    })?;

    let pages = doc.get_pages();
    let total_pages = pages.len();
    if page_num == 0 || page_num > total_pages {
        return Err(PdfCliError::invalid_parameter(format!(
            "指定页码 {} 越界，文档总页数为 {}",
            page_num, total_pages
        )));
    }

    let page_id = *pages.get(&(page_num as u32)).ok_or_else(|| {
        PdfCliError::not_found(format!("未找到第 {} 页的页面对象", page_num))
    })?;

    let blocks = extract_text_blocks(&doc, page_id)?;
    let overlap_anomalies = detect_overlap_anomalies(&blocks);
    let detected_columns = cluster_columns(&blocks);
    let structured_rows = build_structured_rows(&blocks, &detected_columns);

    Ok(InspectLayoutResponse {
        status: "success".to_string(),
        file: file_path.to_string(),
        page: page_num,
        total_text_blocks: blocks.len(),
        detected_columns,
        structured_rows,
        overlap_anomalies,
    })
}

fn extract_text_blocks(doc: &Document, page_id: ObjectId) -> Result<Vec<TextBlock>, PdfCliError> {
    let page_dict = doc.get_dictionary(page_id).map_err(|e| {
        PdfCliError::corrupted_document(format!("获取页面字典失败: {}", e))
    })?;

    let fonts_cmap = extract_page_fonts_cmap(doc, page_dict);
    let content_data = doc.get_page_content(page_id).unwrap_or_default();
    let content = Content::decode(&content_data).map_err(|e| {
        PdfCliError::corrupted_document(format!("解码内容流失败: {}", e))
    })?;

    let mut blocks = Vec::new();
    let mut current_font_name = String::new();
    let mut current_font_size: f64 = 10.0;
    let mut current_x: f64 = 0.0;
    let mut current_y: f64 = 0.0;
    let mut in_text_object = false;

    for op in &content.operations {
        match op.operator.as_ref() {
            "BT" => {
                in_text_object = true;
                current_x = 0.0;
                current_y = 0.0;
            }
            "ET" => {
                in_text_object = false;
            }
            "Tf" => {
                if let Some(font_name_obj) = op.operands.first() {
                    if let Ok(name) = font_name_obj.as_name_str() {
                        current_font_name = name.to_string();
                    }
                }
                if let Some(size_obj) = op.operands.get(1) {
                    current_font_size = obj_to_f64(size_obj).unwrap_or(10.0);
                }
            }
            "Td" | "TD" => {
                if op.operands.len() >= 2 {
                    current_x += obj_to_f64(&op.operands[0]).unwrap_or(0.0);
                    current_y += obj_to_f64(&op.operands[1]).unwrap_or(0.0);
                }
            }
            "Tm" => {
                if op.operands.len() >= 6 {
                    current_x = obj_to_f64(&op.operands[4]).unwrap_or(0.0);
                    current_y = obj_to_f64(&op.operands[5]).unwrap_or(0.0);
                }
            }
            "Tj" => {
                if in_text_object {
                    if let Some(str_obj) = op.operands.first() {
                        let text = decode_text_object(str_obj, &current_font_name, &fonts_cmap);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            let width = calculate_text_width(trimmed, current_font_size);
                            blocks.push(TextBlock {
                                text: trimmed.to_string(),
                                bbox: BBox {
                                    x: (current_x * 10.0_f64).round() / 10.0_f64,
                                    y: (current_y * 10.0_f64).round() / 10.0_f64,
                                    width: (width * 10.0_f64).round() / 10.0_f64,
                                    height: (current_font_size * 10.0_f64).round() / 10.0_f64,
                                },
                                font_size: current_font_size,
                            });
                        }
                    }
                }
            }
            "TJ" => {
                if in_text_object {
                    if let Some(Object::Array(arr)) = op.operands.first() {
                        let mut combined_text = String::new();
                        for item in arr {
                            if let Object::String(bytes, _) = item {
                                let decoded = if let Some(cmap) = fonts_cmap.get(&current_font_name) {
                                    cmap.decode_bytes(bytes)
                                } else {
                                    CMapTable::new().decode_bytes(bytes)
                                };
                                combined_text.push_str(&decoded);
                            }
                        }
                        let trimmed = combined_text.trim();
                        if !trimmed.is_empty() {
                            let width = calculate_text_width(trimmed, current_font_size);
                            blocks.push(TextBlock {
                                text: trimmed.to_string(),
                                bbox: BBox {
                                    x: (current_x * 10.0_f64).round() / 10.0_f64,
                                    y: (current_y * 10.0_f64).round() / 10.0_f64,
                                    width: (width * 10.0_f64).round() / 10.0_f64,
                                    height: (current_font_size * 10.0_f64).round() / 10.0_f64,
                                },
                                font_size: current_font_size,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(blocks)
}

fn calculate_text_width(text: &str, font_size: f64) -> f64 {
    let mut w = 0.0;
    for c in text.chars() {
        if c.is_ascii() {
            w += font_size * 0.55;
        } else {
            w += font_size * 1.0;
        }
    }
    w
}

fn detect_overlap_anomalies(blocks: &[TextBlock]) -> Vec<OverlapAnomaly> {
    let mut anomalies = Vec::new();

    for i in 0..blocks.len() {
        for j in (i + 1)..blocks.len() {
            let a = &blocks[i];
            let b = &blocks[j];

            if (a.bbox.y - b.bbox.y).abs() < 2.0 {
                let x_overlap_start = a.bbox.x.max(b.bbox.x);
                let x_overlap_end = (a.bbox.x + a.bbox.width).min(b.bbox.x + b.bbox.width);

                if x_overlap_end > x_overlap_start {
                    let overlap_len = x_overlap_end - x_overlap_start;
                    let min_width = a.bbox.width.min(b.bbox.width);

                    if min_width > 0.0 && (overlap_len / min_width) > 0.6 && a.text != b.text {
                        anomalies.push(OverlapAnomaly {
                            text_a: a.text.clone(),
                            text_b: b.text.clone(),
                            bbox: BBox {
                                x: x_overlap_start,
                                y: a.bbox.y,
                                width: overlap_len,
                                height: a.bbox.height.max(b.bbox.height),
                            },
                            message: "检测到图层重叠异常，可能存在伪造覆盖或双层排版干扰".to_string(),
                        });
                    }
                }
            }
        }
    }

    anomalies
}

fn cluster_columns(blocks: &[TextBlock]) -> Vec<TableColumn> {
    if blocks.is_empty() {
        return Vec::new();
    }

    let mut x_coords: Vec<f64> = blocks.iter().map(|b| b.bbox.x).collect();
    x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut clusters: Vec<(f64, usize)> = Vec::new();

    for x in x_coords {
        if let Some(last) = clusters.last_mut() {
            if (x - last.0).abs() < 18.0 {
                last.0 = (last.0 * last.1 as f64 + x) / (last.1 + 1) as f64;
                last.1 += 1;
                continue;
            }
        }
        clusters.push((x, 1));
    }

    let valid_anchors: Vec<f64> = clusters
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(x, _)| (x * 10.0_f64).round() / 10.0_f64)
        .collect();

    let mut columns = Vec::new();
    for (i, &x) in valid_anchors.iter().enumerate() {
        let x_end = if i + 1 < valid_anchors.len() {
            valid_anchors[i + 1] - 5.0
        } else {
            x + 200.0
        };
        columns.push(TableColumn {
            col_index: i,
            x_start: x,
            x_end,
        });
    }

    columns
}

fn build_structured_rows(blocks: &[TextBlock], columns: &[TableColumn]) -> Vec<StructuredRow> {
    if blocks.is_empty() || columns.is_empty() {
        return Vec::new();
    }

    let mut rows_map: BTreeMap<i64, Vec<&TextBlock>> = BTreeMap::new();
    for b in blocks {
        let key = (b.bbox.y / 4.0).round() as i64;
        rows_map.entry(key).or_default().push(b);
    }

    let mut structured_rows = Vec::new();
    let first_col_start = columns.first().map(|c| c.x_start).unwrap_or(0.0);

    for (_key, mut row_blocks) in rows_map.into_iter().rev() {
        row_blocks.sort_by(|a, b| a.bbox.x.partial_cmp(&b.bbox.x).unwrap_or(std::cmp::Ordering::Equal));

        let y_val = row_blocks.first().map(|b| b.bbox.y).unwrap_or(0.0);
        let mut cells = vec![String::new(); columns.len()];
        let mut indent_level = 0;

        if let Some(first_block) = row_blocks.iter().find(|b| b.bbox.x < (first_col_start + 100.0)) {
            let offset = first_block.bbox.x - first_col_start;
            if offset > 8.0 {
                indent_level = (offset / 12.0).round() as usize;
            }
        }

        for block in row_blocks {
            if let Some(col) = columns.iter().find(|c| block.bbox.x >= (c.x_start - 10.0) && block.bbox.x < c.x_end) {
                if !cells[col.col_index].is_empty() {
                    cells[col.col_index].push(' ');
                }
                cells[col.col_index].push_str(&block.text);
            }
        }

        if cells.iter().any(|c| !c.trim().is_empty()) {
            structured_rows.push(StructuredRow {
                y: (y_val * 10.0_f64).round() / 10.0_f64,
                indent_level,
                cells,
            });
        }
    }

    structured_rows
}

fn extract_page_fonts_cmap(doc: &Document, page_dict: &lopdf::Dictionary) -> BTreeMap<String, CMapTable> {
    let mut fonts = BTreeMap::new();
    if let Ok(resources_obj) = page_dict.get(b"Resources") {
        let resources = match resources_obj {
            Object::Dictionary(d) => Some(d),
            Object::Reference(r) => doc.get_dictionary(*r).ok(),
            _ => None,
        };

        if let Some(res) = resources {
            if let Ok(font_obj) = res.get(b"Font") {
                let font_dict = match font_obj {
                    Object::Dictionary(d) => Some(d),
                    Object::Reference(r) => doc.get_dictionary(*r).ok(),
                    _ => None,
                };

                if let Some(fonts_map) = font_dict {
                    for (name_bytes, item_obj) in fonts_map {
                        let font_name = String::from_utf8_lossy(name_bytes).to_string();
                        if let Ok(font_ref) = item_obj.as_reference() {
                            if let Ok(single_font_dict) = doc.get_dictionary(font_ref) {
                                if let Ok(to_unicode_ref) = single_font_dict.get(b"ToUnicode").and_then(Object::as_reference) {
                                    if let Ok(stream) = doc.get_object(to_unicode_ref).and_then(Object::as_stream) {
                                        if let Ok(decompressed) = stream.decompressed_content() {
                                            let cmap_str = String::from_utf8_lossy(&decompressed);
                                            fonts.insert(font_name, CMapTable::parse_to_unicode(&cmap_str));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fonts
}

fn decode_text_object(obj: &Object, font_name: &str, fonts_cmap: &BTreeMap<String, CMapTable>) -> String {
    match obj {
        Object::String(bytes, _) => {
            if let Some(cmap) = fonts_cmap.get(font_name) {
                cmap.decode_bytes(bytes)
            } else {
                CMapTable::new().decode_bytes(bytes)
            }
        }
        _ => String::new(),
    }
}