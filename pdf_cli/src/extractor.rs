use lopdf::content::Content;
use lopdf::{Document, Object, ObjectId};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::cmap::CMapTable;
use crate::error::PdfCliError;
use crate::inspect::{inspect_pdf, obj_to_f64, OutlineNode};
use crate::layout::BBox;

#[derive(Debug, Serialize, Clone)]
pub struct TextBlockItem {
    pub text: String,
    pub bbox: BBox,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExtractedPageText {
    pub page: usize,
    pub text: String,
    pub word_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<TextBlockItem>>,
}

#[derive(Debug, Serialize)]
pub struct ExtractTextResponse {
    pub status: String,
    pub file: String,
    pub total_extracted_pages: usize,
    pub format: String,
    pub pages: Vec<ExtractedPageText>,
}

#[derive(Debug, Clone)]
struct TextElement {
    x: f64,
    y: f64,
    text: String,
}

pub fn extract_text(
    file_path: &str,
    page: Option<usize>,
    range: Option<&str>,
    heading: Option<&str>,
    clean_noise: bool,
    with_bbox: bool,
    format: &str,
) -> Result<ExtractTextResponse, PdfCliError> {
    let doc = Document::load(file_path).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("password") || msg.to_lowercase().contains("encrypted") {
            PdfCliError::encrypted_pdf(format!("文档受密码保护: {}", msg))
        } else {
            PdfCliError::corrupted_document(format!("无法加载 PDF 文档: {}", msg))
        }
    })?;

    let all_pages = doc.get_pages();
    let total_pages = all_pages.len();

    let target_pages = if let Some(h) = heading {
        resolve_pages_by_heading(file_path, h, total_pages)?
    } else {
        resolve_page_targets(page, range, total_pages)?
    };

    let mut pages_result = Vec::new();

    for page_num in target_pages {
        if let Some(&page_id) = all_pages.get(&(page_num as u32)) {
            let (page_text, raw_elements) = extract_single_page_raw(&doc, page_id, clean_noise)?;
            let final_text = if format.eq_ignore_ascii_case("md") || format.eq_ignore_ascii_case("markdown") {
                format_as_markdown(&page_text)
            } else {
                page_text
            };

            // 如果请求了 with_bbox，按行聚合生成干净的 TextBlockItem，避免单字碎片导致几千行输出超限
            let blocks = if with_bbox {
                Some(build_line_bboxes(raw_elements))
            } else {
                None
            };

            let word_count = final_text.chars().filter(|c| !c.is_whitespace()).count();
            pages_result.push(ExtractedPageText {
                page: page_num,
                text: final_text,
                word_count,
                blocks,
            });
        }
    }

    Ok(ExtractTextResponse {
        status: "success".to_string(),
        file: file_path.to_string(),
        total_extracted_pages: pages_result.len(),
        format: format.to_string(),
        pages: pages_result,
    })
}

/// 将细碎字符元素按行合并为高阶文本行 BBox，避免输出体积过大打爆终端
fn build_line_bboxes(mut elements: Vec<TextElement>) -> Vec<TextBlockItem> {
    if elements.is_empty() {
        return Vec::new();
    }

    // 过滤空白或极度异常的点
    elements.retain(|e| !e.text.trim().is_empty());

    // 按纵坐标自上而下粗排
    elements.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    // 行桶聚类 (容差 6.0pt)
    let mut lines: Vec<Vec<TextElement>> = Vec::new();
    for elem in elements {
        let mut placed = false;
        for line in lines.iter_mut().rev() {
            if let Some(anchor) = line.first() {
                if (elem.y - anchor.y).abs() <= 6.0 {
                    line.push(elem.clone());
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            lines.push(vec![elem]);
        }
    }

    let mut result_blocks = Vec::new();

    for mut line in lines {
        // 行内自左向右排序
        line.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let mut combined_text = String::new();
        let min_x = line.first().map(|e| e.x).unwrap_or(0.0);
        let mut max_right = min_x;
        let y_val = line.first().map(|e| e.y).unwrap_or(0.0);

        for e in line {
            if !combined_text.is_empty() && (e.x - max_right) > 3.0 && needs_space(&combined_text, &e.text) {
                combined_text.push(' ');
            }
            combined_text.push_str(&e.text);

            let char_len = e.text.chars().fold(0.0, |acc, c| {
                if c.is_ascii() { acc + 6.5 } else { acc + 12.0 }
            });
            let right = e.x + char_len;
            if right > max_right {
                max_right = right;
            }
        }

        let trimmed = combined_text.trim();
        if !trimmed.is_empty() {
            result_blocks.push(TextBlockItem {
                text: trimmed.to_string(),
                bbox: BBox {
                    x: (min_x * 10.0).round() / 10.0,
                    y: (y_val * 10.0).round() / 10.0,
                    width: ((max_right - min_x) * 10.0).round() / 10.0,
                    height: 12.0,
                },
            });
        }
    }

    result_blocks
}

fn normalize_for_match(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || ('\u{4e00}'..='\u{9fa5}').contains(c))
        .collect::<String>()
        .to_lowercase()
}

fn resolve_pages_by_heading(
    file_path: &str,
    heading_query: &str,
    total_pages: usize,
) -> Result<Vec<usize>, PdfCliError> {
    let inspect_res = inspect_pdf(file_path, Some("outline"))?;
    let outlines = inspect_res.outlines.unwrap_or_default();

    if outlines.is_empty() {
        return Err(PdfCliError::not_found(
            "该文档未包含书签大纲(Outlines)，无法按章节标题定向抽取",
        ));
    }

    let mut flat = Vec::new();
    flatten_nodes(&outlines, &mut flat);

    let norm_query = normalize_for_match(heading_query);

    let target_idx = flat
        .iter()
        .position(|node| {
            let norm_title = normalize_for_match(&node.title);
            if !norm_query.is_empty() && (norm_title.contains(&norm_query) || norm_query.contains(&norm_title)) {
                return true;
            }
            let raw_lower = node.title.to_lowercase();
            let query_lower = heading_query.trim().to_lowercase();
            raw_lower.contains(&query_lower) || query_lower.contains(&raw_lower)
        })
        .ok_or_else(|| {
            PdfCliError::not_found(format!("未在文档大纲中检索到标题: '{}'", heading_query))
        })?;

    let start_page = flat[target_idx].target_page.unwrap_or(1);

    let mut end_page = total_pages;
    for node in &flat[(target_idx + 1)..] {
        if let Some(p) = node.target_page {
            if p > start_page {
                end_page = p - 1;
                break;
            }
        }
    }

    let bounded_end = end_page.min(total_pages).max(start_page);
    Ok((start_page..=bounded_end).collect())
}

fn flatten_nodes(nodes: &[OutlineNode], acc: &mut Vec<OutlineNode>) {
    for n in nodes {
        acc.push(n.clone());
        if !n.children.is_empty() {
            flatten_nodes(&n.children, acc);
        }
    }
}

fn extract_single_page_raw(
    doc: &Document,
    page_id: ObjectId,
    clean_noise: bool,
) -> Result<(String, Vec<TextElement>), PdfCliError> {
    let page_dict = doc.get_dictionary(page_id).map_err(|e| {
        PdfCliError::corrupted_document(format!("无法获取页面字典: {}", e))
    })?;

    let page_height = get_page_height(page_dict);
    let fonts_cmap = extract_page_fonts_cmap(doc, page_dict);

    let content_data = doc.get_page_content(page_id).unwrap_or_default();
    let content = Content::decode(&content_data).map_err(|e| {
        PdfCliError::corrupted_document(format!("解码页面内容流失败: {}", e))
    })?;

    let mut elements: Vec<TextElement> = Vec::new();
    let mut current_font_name = String::new();
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
            }
            "Td" | "TD" => {
                if op.operands.len() >= 2 {
                    let tx = obj_to_f64(&op.operands[0]).unwrap_or(0.0);
                    let ty = obj_to_f64(&op.operands[1]).unwrap_or(0.0);
                    current_x += tx;
                    current_y += ty;
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
                        if !text.is_empty() {
                            elements.push(TextElement {
                                x: current_x,
                                y: current_y,
                                text,
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
                                let decoded = decode_bytes_with_cmap(bytes, &current_font_name, &fonts_cmap);
                                combined_text.push_str(&decoded);
                            }
                        }
                        if !combined_text.is_empty() {
                            elements.push(TextElement {
                                x: current_x,
                                y: current_y,
                                text: combined_text,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let raw_elements_copy = elements.clone();

    if clean_noise && page_height > 100.0 && elements.len() > 15 {
        let max_y = elements.iter().map(|e| e.y).fold(f64::NEG_INFINITY, f64::max);
        let min_y = elements.iter().map(|e| e.y).fold(f64::INFINITY, f64::min);
        let span_y = (max_y - min_y).abs();

        if span_y > 100.0 {
            let header_bound = max_y - (span_y * 0.05);
            let footer_bound = min_y + (span_y * 0.05);

            elements.retain(|elem| {
                let trimmed = elem.text.trim();
                let is_marginal = elem.y >= header_bound || elem.y <= footer_bound;
                if is_marginal {
                    let is_num = trimmed.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '/' || c == ' ');
                    if is_num || trimmed.chars().count() < 10 {
                        return false;
                    }
                }
                true
            });
        }
    }

    let reconstructed_text = sort_and_reconstruct_text(elements);
    Ok((reconstructed_text, raw_elements_copy))
}

fn sort_and_reconstruct_text(mut elements: Vec<TextElement>) -> String {
    if elements.is_empty() {
        return String::new();
    }

    let min_y = elements.iter().map(|e| e.y).fold(f64::INFINITY, f64::min);
    let max_y = elements.iter().map(|e| e.y).fold(f64::NEG_INFINITY, f64::max);
    let is_bottom_origin = max_y > min_y;

    elements.sort_by(|a, b| {
        if is_bottom_origin {
            b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let mut lines: Vec<Vec<TextElement>> = Vec::new();
    for elem in elements {
        let mut placed = false;
        for line in lines.iter_mut().rev() {
            if let Some(anchor) = line.first() {
                if (elem.y - anchor.y).abs() <= 6.0 {
                    line.push(elem.clone());
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            lines.push(vec![elem]);
        }
    }

    let mut formatted_doc = String::new();
    let mut last_line_y = f64::NAN;

    for mut line in lines {
        line.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let current_y = line[0].y;
        let mut line_str = String::new();
        let mut last_elem_right = f64::NAN;

        for elem in line {
            let t = &elem.text;
            if !last_elem_right.is_nan() {
                let gap = elem.x - last_elem_right;
                if gap > 2.5 && needs_space(&line_str, t) {
                    line_str.push(' ');
                }
            }

            line_str.push_str(t);
            let estimated_width = t.chars().fold(0.0, |acc, c| {
                if c.is_ascii() { acc + 6.5 } else { acc + 12.0 }
            });
            last_elem_right = elem.x + estimated_width;
        }

        let clean_line = line_str.trim();
        if !clean_line.is_empty() {
            if !last_line_y.is_nan() {
                let y_gap = (last_line_y - current_y).abs();
                if y_gap > 18.0 {
                    formatted_doc.push_str("\n\n");
                } else {
                    formatted_doc.push('\n');
                }
            }
            formatted_doc.push_str(clean_line);
            last_line_y = current_y;
        }
    }

    formatted_doc
}

fn needs_space(last_str: &str, next_str: &str) -> bool {
    if let (Some(last_c), Some(next_c)) = (last_str.chars().last(), next_str.chars().next()) {
        if last_c.is_ascii_alphanumeric() && next_c.is_ascii_alphanumeric() {
            return true;
        }
    }
    false
}

fn format_as_markdown(raw_text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in raw_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.push(String::new());
            continue;
        }

        if is_likely_heading(trimmed) {
            lines.push(format!("\n### {}\n", trimmed));
        } else {
            lines.push(trimmed.to_string());
        }
    }
    lines.join("\n")
}

fn is_likely_heading(s: &str) -> bool {
    if s.len() > 60 {
        return false;
    }
    if s.starts_with('第') && (s.contains('章') || s.contains('节') || s.contains('篇')) {
        return true;
    }
    if s.chars().next().map_or(false, |c| c.is_ascii_digit()) && s.contains('.') {
        let prefix = s.split_whitespace().next().unwrap_or("");
        if prefix.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return true;
        }
    }
    false
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
                                            let table = CMapTable::parse_to_unicode(&cmap_str);
                                            fonts.insert(font_name, table);
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
        Object::String(bytes, _) => decode_bytes_with_cmap(bytes, font_name, fonts_cmap),
        _ => String::new(),
    }
}

fn decode_bytes_with_cmap(bytes: &[u8], font_name: &str, fonts_cmap: &BTreeMap<String, CMapTable>) -> String {
    if let Some(cmap) = fonts_cmap.get(font_name) {
        cmap.decode_bytes(bytes)
    } else {
        CMapTable::new().decode_bytes(bytes)
    }
}

fn get_page_height(page_dict: &lopdf::Dictionary) -> f64 {
    if let Ok(arr) = page_dict.get(b"MediaBox").and_then(Object::as_array) {
        if arr.len() >= 4 {
            let y0 = obj_to_f64(&arr[1]).unwrap_or(0.0);
            let y1 = obj_to_f64(&arr[3]).unwrap_or(841.89);
            return (y1 - y0).abs();
        }
    }
    841.89
}

fn resolve_page_targets(
    page: Option<usize>,
    range: Option<&str>,
    total_pages: usize,
) -> Result<Vec<usize>, PdfCliError> {
    if let Some(p) = page {
        if p == 0 || p > total_pages {
            return Err(PdfCliError::invalid_parameter(format!(
                "指定页码 {} 越界，文档共 {} 页",
                p, total_pages
            )));
        }
        return Ok(vec![p]);
    }

    if let Some(r) = range {
        if r.eq_ignore_ascii_case("all") {
            return Ok((1..=total_pages).collect());
        }
        if let Some((start_s, end_s)) = r.split_once(':') {
            let start = start_s.parse::<usize>().unwrap_or(1).max(1);
            let end = end_s.parse::<usize>().unwrap_or(total_pages).min(total_pages);
            if start > end {
                return Err(PdfCliError::invalid_parameter(format!(
                    "无效的页码范围: start({}) > end({})",
                    start, end
                )));
            }
            return Ok((start..=end).collect());
        }
        if let Ok(single) = r.parse::<usize>() {
            if single == 0 || single > total_pages {
                return Err(PdfCliError::invalid_parameter(format!(
                    "页码 {} 越界",
                    single
                )));
            }
            return Ok(vec![single]);
        }
    }

    Ok((1..=total_pages).collect())
}