use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::WordCliError;
use crate::package::DocxPackage;

#[derive(Debug, Serialize, Clone)]
pub struct SummaryInfo {
    pub total_paragraphs: usize,
    pub total_tables: usize,
    pub total_images: usize,
    pub total_highlights: usize,
    pub total_sections: usize,
    pub page_orientation: String,
    pub page_size: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub para_idx: usize,
    pub style_id: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TableOverview {
    pub index: usize,
    pub rows: usize,
    pub cols: usize,
    pub preview_first_row: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImageOverview {
    pub index: usize,
    pub rel_id: String,
    pub filename: String,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct PageSectionOverview {
    pub page_index: usize,
    pub start_para_idx: usize,
    pub end_para_idx: usize,
    pub headings: Vec<String>,
    pub preview: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct HighlightItem {
    pub para_idx: usize,
    pub color: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct InspectResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<SummaryInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<Vec<OutlineItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<TableOverview>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageOverview>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<Vec<HighlightItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<PageSectionOverview>>,
}

pub fn inspect_document(
    pkg: &DocxPackage,
    only: Option<&str>,
) -> Result<InspectResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml").map_err(|_| {
        WordCliError::not_found("文档中未找到 word/document.xml 主体内容")
    })?;

    let rels_map = parse_relationships(pkg);

    let mut reader = Reader::from_str(&doc_xml);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut outline = Vec::new();
    let mut tables = Vec::new();
    let mut images = Vec::new();
    let mut highlights = Vec::new();
    let mut pages = Vec::new();

    let mut current_para_idx: usize = 0;
    let mut total_sections = 0;
    let mut page_orientation = "portrait".to_string();
    let mut page_size = "A4".to_string();

    let mut in_para = false;
    let mut current_para_text = String::new();
    let mut current_style_id = String::new();

    let mut in_run = false;
    let mut current_run_highlight: Option<String> = None;
    let mut current_run_text = String::new();

    let mut in_table = false;
    let mut table_rows = 0;
    let mut table_first_row = Vec::new();
    let mut in_row = false;
    let mut in_cell = false;
    let mut current_cell_text = String::new();
	
	// 页面切片跟踪
	let mut current_page_idx: usize = 1;
	let mut page_start_para: usize = 0;
	let mut page_headings = Vec::new();
	let mut page_preview = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"p" => {
                    in_para = true;
                    current_para_text.clear();
                    current_style_id.clear();
                }
                b"pStyle" if in_para => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_style_id =
                                String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                b"r" => {
                    in_run = true;
                    current_run_highlight = None;
                    current_run_text.clear();
                }
                b"highlight" if in_run => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_run_highlight =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
                b"t" if in_run => {}
                b"tbl" => {
                    in_table = true;
                    table_rows = 0;
                    table_first_row.clear();
                }
                b"tr" if in_table => {
                    in_row = true;
                    table_rows += 1;
                }
                b"tc" if in_row => {
                    in_cell = true;
                    current_cell_text.clear();
                }
                b"extent" => {
                    let mut cx = 0f64;
                    let mut cy = 0f64;
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"cx" {
                            if let Ok(val) =
                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                            {
                                cx = (val / 36000.0 * 10.0).round() / 10.0;
                            }
                        } else if attr.key.local_name().as_ref() == b"cy" {
                            if let Ok(val) =
                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                            {
                                cy = (val / 36000.0 * 10.0).round() / 10.0;
                            }
                        }
                    }
                    if cx > 0.0 && cy > 0.0 {
                        let idx = images.len();
                        images.push(ImageOverview {
                            index: idx,
                            rel_id: String::new(),
                            filename: format!("image_{}.bin", idx),
                            width_mm: cx,
                            height_mm: cy,
                        });
                    }
                }
                b"blip" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"embed" {
                            let rid = String::from_utf8_lossy(&attr.value).to_string();
                            if let Some(last) = images.last_mut() {
                                if last.rel_id.is_empty() {
                                    last.rel_id = rid.clone();
                                    if let Some(target) = rels_map.get(&rid) {
                                        last.filename = target.clone();
                                    }
                                }
                            }
                        }
                    }
                }
                b"sectPr" => {
                    total_sections += 1;
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"br" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"type"
                            && attr.value.as_ref() == b"page"
                        {
                            pages.push(PageSectionOverview {
                                page_index: current_page_idx,
                                start_para_idx: page_start_para,
                                end_para_idx: current_para_idx.saturating_sub(1),
                                headings: page_headings.clone(),
                                preview: page_preview.chars().take(80).collect(),
                            });
                            current_page_idx += 1;
                            page_start_para = current_para_idx;
                            page_headings.clear();
                            page_preview.clear();
                        }
                    }
                }
                b"pStyle" if in_para => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_style_id =
                                String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                b"highlight" if in_run => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_run_highlight =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
                b"pgSz" => {
                    let mut w = 0u32;
                    let mut h = 0u32;
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"orient" => {
                                page_orientation =
                                    String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"w" => {
                                w = String::from_utf8_lossy(&attr.value)
                                    .parse::<u32>()
                                    .unwrap_or(0);
                            }
                            b"h" => {
                                h = String::from_utf8_lossy(&attr.value)
                                    .parse::<u32>()
                                    .unwrap_or(0);
                            }
                            _ => {}
                        }
                    }
                    if page_orientation == "portrait" && w > h && w > 0 && h > 0 {
                        page_orientation = "landscape".to_string();
                    }
                    let max_dim = w.max(h);
                    let min_dim = w.min(h);
                    page_size = if (max_dim as i32 - 16838).abs() < 100 && (min_dim as i32 - 11906).abs() < 100 {
                        "A4".to_string()
                    } else if (max_dim as i32 - 23811).abs() < 100 && (min_dim as i32 - 16838).abs() < 100 {
                        "A3".to_string()
                    } else if (max_dim as i32 - 15840).abs() < 100 && (min_dim as i32 - 12240).abs() < 100 {
                        "Letter".to_string()
                    } else {
                        "Custom".to_string()
                    };
                }
                b"extent" => {
                    let mut cx = 0f64;
                    let mut cy = 0f64;
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"cx" {
                            if let Ok(val) =
                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                            {
                                cx = (val / 36000.0 * 10.0).round() / 10.0;
                            }
                        } else if attr.key.local_name().as_ref() == b"cy" {
                            if let Ok(val) =
                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                            {
                                cy = (val / 36000.0 * 10.0).round() / 10.0;
                            }
                        }
                    }
                    if cx > 0.0 && cy > 0.0 {
                        let idx = images.len();
                        images.push(ImageOverview {
                            index: idx,
                            rel_id: String::new(),
                            filename: format!("image_{}.bin", idx),
                            width_mm: cx,
                            height_mm: cy,
                        });
                    }
                }
                b"blip" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"embed" {
                            let rid = String::from_utf8_lossy(&attr.value).to_string();
                            if let Some(last) = images.last_mut() {
                                if last.rel_id.is_empty() {
                                    last.rel_id = rid.clone();
                                    if let Some(target) = rels_map.get(&rid) {
                                        last.filename = target.clone();
                                    }
                                }
                            }
                        }
                    }
                }
                b"sectPr" => {
                    total_sections += 1;
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_run {
                    current_run_text.push_str(&text);
                }
                if in_para {
                    current_para_text.push_str(&text);
                }
                if in_cell {
                    current_cell_text.push_str(&text);
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"r" => {
                    in_run = false;
                    if let Some(color) = current_run_highlight.take() {
                        if !current_run_text.trim().is_empty() {
                            highlights.push(HighlightItem {
                                para_idx: current_para_idx,
                                color,
                                text: current_run_text.clone(),
                            });
                        }
                    }
                }
                b"p" => {
                    in_para = false;
                    let trimmed = current_para_text.trim().to_string();
                    if !trimmed.is_empty() {
                        if let Some(level) = detect_heading_level(&current_style_id) {
                            outline.push(OutlineItem {
                                level,
                                text: trimmed.clone(),
                                para_idx: current_para_idx,
                                style_id: current_style_id.clone(),
                            });
                            page_headings.push(trimmed.clone());
                        } else if page_preview.len() < 120 {
                            if !page_preview.is_empty() {
                                page_preview.push(' ');
                            }
                            page_preview.push_str(&trimmed);
                        }
                    }
                    current_para_idx += 1;
                }
                b"tc" if in_row => {
                    in_cell = false;
                    if table_rows == 1 {
                        table_first_row.push(current_cell_text.trim().to_string());
                    }
                }
                b"tr" if in_table => {
                    in_row = false;
                }
                b"tbl" => {
                    in_table = false;
                    let idx = tables.len();
                    let cols = table_first_row.len();
                    tables.push(TableOverview {
                        index: idx,
                        rows: table_rows,
                        cols,
                        preview_first_row: table_first_row.clone(),
                    });
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WordCliError::xml_parse(format!(
                    "解析 document.xml 结构失败: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    // 闭合最后一页切片
    if current_para_idx > page_start_para || pages.is_empty() {
        pages.push(PageSectionOverview {
            page_index: current_page_idx,
            start_para_idx: page_start_para,
            end_para_idx: current_para_idx.saturating_sub(1),
            headings: page_headings,
            preview: page_preview.chars().take(80).collect(),
        });
    }

    let summary = SummaryInfo {
        total_paragraphs: current_para_idx,
        total_tables: tables.len(),
        total_images: images.len(),
        total_highlights: highlights.len(),
        total_sections: if total_sections == 0 { 1 } else { total_sections },
        page_orientation,
        page_size,
    };

    match only {
        Some("summary") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: Some(summary),
            outline: None,
            tables: None,
            images: None,
            highlights: None,
            pages: None,
        }),
        Some("outline") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: None,
            outline: Some(outline),
            tables: None,
            images: None,
            highlights: None,
            pages: None,
        }),
        Some("tables") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: None,
            outline: None,
            tables: Some(tables),
            images: None,
            highlights: None,
            pages: None,
        }),
        Some("images") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: None,
            outline: None,
            tables: None,
            images: Some(images),
            highlights: None,
            pages: None,
        }),
        Some("highlights") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: None,
            outline: None,
            tables: None,
            images: None,
            highlights: Some(highlights),
            pages: None,
        }),
        Some("pages") => Ok(InspectResponse {
            status: "success".to_string(),
            summary: None,
            outline: None,
            tables: None,
            images: None,
            highlights: None,
            pages: Some(pages),
        }),
        _ => Ok(InspectResponse {
            status: "success".to_string(),
            summary: Some(summary),
            outline: Some(outline),
            tables: Some(tables),
            images: Some(images),
            highlights: Some(highlights),
            pages: Some(pages),
        }),
    }
}

fn parse_relationships(pkg: &DocxPackage) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(rels_xml) = pkg.get_text("word/_rels/document.xml.rels") {
        let mut reader = Reader::from_str(&rels_xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.local_name().as_ref() == b"Relationship" {
                        let mut id = String::new();
                        let mut target = String::new();
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"Id" {
                                id = String::from_utf8_lossy(&attr.value).to_string();
                            } else if attr.key.local_name().as_ref() == b"Target" {
                                target = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                        if !id.is_empty() && !target.is_empty() {
                            map.insert(id, target);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }
    map
}

fn detect_heading_level(style_id: &str) -> Option<u8> {
    let lower = style_id.to_lowercase();
    if lower.contains("heading1") || lower.contains("heading 1") || (lower.contains('1') && lower.contains("标题")) {
        Some(1)
    } else if lower.contains("heading2") || lower.contains("heading 2") || (lower.contains('2') && lower.contains("标题")) {
        Some(2)
    } else if lower.contains("heading3") || lower.contains("heading 3") || (lower.contains('3') && lower.contains("标题")) {
        Some(3)
    } else if lower.contains("heading4") || lower.contains("heading 4") || (lower.contains('4') && lower.contains("标题")) {
        Some(4)
    } else if lower.contains("heading5") || lower.contains("heading 5") || (lower.contains('5') && lower.contains("标题")) {
        Some(5)
    } else if lower.contains("heading6") || lower.contains("heading 6") || (lower.contains('6') && lower.contains("标题")) {
        Some(6)
    } else {
        None
    }
}