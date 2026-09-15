use lopdf::{Document, Object, ObjectId};
use serde::Serialize;
use std::collections::HashMap;

use crate::error::PdfCliError;

#[derive(Debug, Serialize, Clone)]
pub struct MetadataInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub mod_date: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PageDimension {
    pub page: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub orientation: String,
    pub size_standard: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct OutlineNode {
    pub level: usize,
    pub title: String,
    pub target_page: Option<usize>,
    pub children: Vec<OutlineNode>,
}

#[derive(Debug, Serialize)]
pub struct InspectResponse {
    pub status: String,
    pub file: String,
    pub pdf_version: String,
    pub is_encrypted: bool,
    pub total_pages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages_layout: Option<Vec<PageDimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlines: Option<Vec<OutlineNode>>,
}

pub fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(r) => Some(*r as f64),
        Object::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

/// 执行宏观自省检查
pub fn inspect_pdf(file_path: &str, only: Option<&str>) -> Result<InspectResponse, PdfCliError> {
    let doc = Document::load(file_path).map_err(|e| {
        let msg = e.to_string();
        if msg.to_lowercase().contains("password") || msg.to_lowercase().contains("encrypted") {
            PdfCliError::encrypted_pdf(format!("文档受密码保护: {}", msg))
        } else {
            PdfCliError::corrupted_document(format!("无法加载 PDF 文档: {}", msg))
        }
    })?;

    let is_encrypted = doc.is_encrypted();
    let pdf_version = doc.version.clone();
    let pages = doc.get_pages();
    let total_pages = pages.len();

    let mut page_id_to_num = HashMap::new();
    for (&page_num, &page_id) in &pages {
        page_id_to_num.insert(page_id, page_num as usize);
    }

    let metadata = extract_metadata(&doc);
    let pages_layout = extract_pages_layout(&doc, &pages);
    let outlines = extract_outlines(&doc, &page_id_to_num);

    let (show_meta, show_layout, show_outlines) = match only {
        Some("metadata") => (Some(metadata), None, None),
        Some("layout") => (None, Some(pages_layout), None),
        Some("outline") => (None, None, Some(outlines)),
        _ => (Some(metadata), Some(pages_layout), Some(outlines)),
    };

    Ok(InspectResponse {
        status: "success".to_string(),
        file: file_path.to_string(),
        pdf_version,
        is_encrypted,
        total_pages,
        metadata: show_meta,
        pages_layout: show_layout,
        outlines: show_outlines,
    })
}

fn extract_metadata(doc: &Document) -> MetadataInfo {
    let mut meta = MetadataInfo {
        title: None,
        author: None,
        creator: None,
        producer: None,
        creation_date: None,
        mod_date: None,
    };

    if let Ok(info_ref) = doc.trailer.get(b"Info").and_then(Object::as_reference) {
        if let Ok(info_dict) = doc.get_dictionary(info_ref) {
            meta.title = get_string_from_dict(info_dict, b"Title");
            meta.author = get_string_from_dict(info_dict, b"Author");
            meta.creator = get_string_from_dict(info_dict, b"Creator");
            meta.producer = get_string_from_dict(info_dict, b"Producer");
            meta.creation_date = get_string_from_dict(info_dict, b"CreationDate");
            meta.mod_date = get_string_from_dict(info_dict, b"ModDate");
        }
    }

    meta
}

fn get_string_from_dict(dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key).ok().and_then(|obj| match obj {
        Object::String(bytes, _) => {
            if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                let u16_vec: Vec<u16> = bytes[2..]
                    .chunks_exact(2)
                    .map(|c| ((c[0] as u16) << 8) | (c[1] as u16))
                    .collect();
                Some(String::from_utf16_lossy(&u16_vec))
            } else {
                Some(String::from_utf8_lossy(bytes).to_string())
            }
        }
        _ => None,
    })
}

fn extract_pages_layout(doc: &Document, pages: &std::collections::BTreeMap<u32, ObjectId>) -> Vec<PageDimension> {
    let mut layout = Vec::new();

    for (&page_num, &page_id) in pages {
        if let Ok(page_dict) = doc.get_dictionary(page_id) {
            let media_box = page_dict.get(b"MediaBox").ok().and_then(|obj| match obj {
                Object::Array(arr) => Some(arr.clone()),
                _ => None,
            });

            let mut w: f64 = 595.28;
            let mut h: f64 = 841.89;

            if let Some(arr) = media_box {
                if arr.len() >= 4 {
                    let x0 = obj_to_f64(&arr[0]).unwrap_or(0.0);
                    let y0 = obj_to_f64(&arr[1]).unwrap_or(0.0);
                    let x1 = obj_to_f64(&arr[2]).unwrap_or(595.28);
                    let y1 = obj_to_f64(&arr[3]).unwrap_or(841.89);
                    w = (x1 - x0).abs();
                    h = (y1 - y0).abs();
                }
            }

            let orientation = if w > h { "landscape" } else { "portrait" }.to_string();
            let max_d = w.max(h);
            let min_d = w.min(h);

            let size_standard = if (max_d - 841.89).abs() < 10.0 && (min_d - 595.28).abs() < 10.0 {
                "A4".to_string()
            } else if (max_d - 1190.55).abs() < 10.0 && (min_d - 841.89).abs() < 10.0 {
                "A3".to_string()
            } else if (max_d - 792.0).abs() < 10.0 && (min_d - 612.0).abs() < 10.0 {
                "Letter".to_string()
            } else {
                "Custom".to_string()
            };

            layout.push(PageDimension {
                page: page_num as usize,
                width_pt: (w * 100.0).round() / 100.0,
                height_pt: (h * 100.0).round() / 100.0,
                orientation,
                size_standard,
            });
        }
    }

    layout
}

fn extract_outlines(doc: &Document, page_id_map: &HashMap<ObjectId, usize>) -> Vec<OutlineNode> {
    let mut roots = Vec::new();

    let root_ref = doc.trailer.get(b"Root").and_then(Object::as_reference);
    if let Ok(catalog_id) = root_ref {
        if let Ok(catalog) = doc.get_dictionary(catalog_id) {
            if let Ok(outlines_ref) = catalog.get(b"Outlines").and_then(Object::as_reference) {
                if let Ok(outlines_dict) = doc.get_dictionary(outlines_ref) {
                    if let Ok(first_ref) = outlines_dict.get(b"First").and_then(Object::as_reference) {
                        traverse_outline_items(doc, first_ref, 1, page_id_map, &mut roots);
                    }
                }
            }
        }
    }

    roots
}

fn traverse_outline_items(
    doc: &Document,
    mut current_ref: ObjectId,
    level: usize,
    page_id_map: &HashMap<ObjectId, usize>,
    nodes: &mut Vec<OutlineNode>,
) {
    loop {
        if let Ok(item_dict) = doc.get_dictionary(current_ref) {
            let title = get_string_from_dict(item_dict, b"Title").unwrap_or_else(|| "未命名书签".to_string());
            let target_page = resolve_target_page(doc, item_dict, page_id_map);

            let mut children = Vec::new();
            if let Ok(first_child) = item_dict.get(b"First").and_then(Object::as_reference) {
                traverse_outline_items(doc, first_child, level + 1, page_id_map, &mut children);
            }

            nodes.push(OutlineNode {
                level,
                title,
                target_page,
                children,
            });

            if let Ok(next_ref) = item_dict.get(b"Next").and_then(Object::as_reference) {
                current_ref = next_ref;
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn resolve_target_page(
    doc: &Document,
    item_dict: &lopdf::Dictionary,
    page_id_map: &HashMap<ObjectId, usize>,
) -> Option<usize> {
    if let Ok(dest) = item_dict.get(b"Dest") {
        return resolve_dest_object(dest, page_id_map);
    }

    if let Ok(action_dict) = item_dict.get(b"A").and_then(Object::as_dict) {
        if let Ok(dest) = action_dict.get(b"D") {
            return resolve_dest_object(dest, page_id_map);
        }
    } else if let Ok(action_ref) = item_dict.get(b"A").and_then(Object::as_reference) {
        if let Ok(action_dict) = doc.get_dictionary(action_ref) {
            if let Ok(dest) = action_dict.get(b"D") {
                return resolve_dest_object(dest, page_id_map);
            }
        }
    }

    None
}

fn resolve_dest_object(dest: &Object, page_id_map: &HashMap<ObjectId, usize>) -> Option<usize> {
    match dest {
        Object::Array(arr) => {
            if let Some(target) = arr.first() {
                if let Ok(target_id) = target.as_reference() {
                    return page_id_map.get(&target_id).copied();
                } else if let Ok(page_idx) = target.as_i64() {
                    return Some((page_idx + 1) as usize);
                }
            }
            None
        }
        Object::Reference(target_id) => page_id_map.get(target_id).copied(),
        _ => None,
    }
}