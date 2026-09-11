use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::error::WordCliError;
use crate::package::DocxPackage;

#[derive(Debug, Serialize)]
pub struct MediaOpResponse {
    pub status: String,
    pub command: String,
    pub file: String,
    pub affected_count: usize,
    pub message: String,
}

/// 导出文档中所有图片到目标目录
pub fn extract_all_images(pkg: &DocxPackage, out_dir: &str) -> Result<MediaOpResponse, WordCliError> {
    let out_path = Path::new(out_dir);
    if !out_path.exists() {
        fs::create_dir_all(out_path).map_err(|e| {
            WordCliError::file_io(format!("创建图片导出目录失败: {}", e))
        })?;
    }

    let mut count = 0;
    for (name, bytes) in &pkg.entries {
        if name.starts_with("word/media/") {
            let filename = Path::new(name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.bin");
            let target_file = out_path.join(filename);
            let mut f = File::create(&target_file).map_err(|e| {
                WordCliError::file_io(format!("无法写入导出图片 {:?}: {}", target_file, e))
            })?;
            f.write_all(bytes).map_err(|e| {
                WordCliError::file_io(format!("写入图片数据失败: {}", e))
            })?;
            count += 1;
        }
    }

    Ok(MediaOpResponse {
        status: "success".to_string(),
        command: "extract-images".to_string(),
        file: out_dir.to_string(),
        affected_count: count,
        message: format!("已成功导出 {} 张图片至 {}", count, out_dir),
    })
}

/// 根据 rId 或文件名替换特定图片的物理二进制
pub fn replace_image_binary(
    pkg: &mut DocxPackage,
    doc_path: &str,
    target_rel_id: Option<&str>,
    target_filename: Option<&str>,
    new_image_path: &str,
) -> Result<MediaOpResponse, WordCliError> {
    let mut new_img_file = File::open(new_image_path).map_err(|e| {
        WordCliError::file_io(format!("无法打开新图片文件 {}: {}", new_image_path, e))
    })?;
    let mut new_bytes = Vec::new();
    new_img_file.read_to_end(&mut new_bytes).map_err(|e| {
        WordCliError::file_io(format!("读取新图片字节失败: {}", e))
    })?;

    let mut entry_key: Option<String> = None;

    if let Some(filename) = target_filename {
        let expected = format!("word/media/{}", filename);
        if pkg.entries.contains_key(&expected) {
            entry_key = Some(expected);
        } else if pkg.entries.contains_key(filename) {
            entry_key = Some(filename.to_string());
        }
    } else if let Some(rid) = target_rel_id {
        if let Ok(rels_xml) = pkg.get_text("word/_rels/document.xml.rels") {
            let mut reader = Reader::from_str(&rels_xml);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                        if e.local_name().as_ref() == b"Relationship" {
                            let mut current_id = String::new();
                            let mut current_target = String::new();
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"Id" {
                                    current_id = String::from_utf8_lossy(&attr.value).to_string();
                                } else if attr.key.local_name().as_ref() == b"Target" {
                                    current_target = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                            if current_id == rid {
                                entry_key = Some(format!("word/{}", current_target.trim_start_matches('/')));
                                break;
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }

    let key = entry_key.ok_or_else(|| {
        WordCliError::not_found("未匹配到目标图片对象，请核对 rId 或文件名")
    })?;

    pkg.entries.insert(key.clone(), new_bytes);
    pkg.save_to_file(doc_path)?;

    Ok(MediaOpResponse {
        status: "success".to_string(),
        command: "replace-image".to_string(),
        file: doc_path.to_string(),
        affected_count: 1,
        message: format!("已成功用 {} 替换图片条目 {}", new_image_path, key),
    })
}

/// 在指定段落索引前插入图片
pub fn insert_image_at_para(
    pkg: &mut DocxPackage,
    doc_path: &str,
    target_idx: usize,
    img_path: &str,
    width_mm: f64,
    height_mm: f64,
) -> Result<MediaOpResponse, WordCliError> {
    let mut img_file = File::open(img_path).map_err(|e| {
        WordCliError::file_io(format!("无法打开图片文件 {}: {}", img_path, e))
    })?;
    let mut img_bytes = Vec::new();
    img_file.read_to_end(&mut img_bytes).map_err(|e| {
        WordCliError::file_io(format!("读取图片数据失败: {}", e))
    })?;

    let ext = Path::new(img_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    // 1. 生成媒体包内唯一文件名
    let mut media_idx = 1;
    let mut media_entry_name = format!("word/media/image{}.{}", media_idx, ext);
    while pkg.entries.contains_key(&media_entry_name) {
        media_idx += 1;
        media_entry_name = format!("word/media/image{}.{}", media_idx, ext);
    }
    let media_target_rel = format!("media/image{}.{}", media_idx, ext);
    pkg.entries.insert(media_entry_name, img_bytes);

    // 2. 注册 Relationships
    let mut rel_idx = 100;
    let mut rel_id = format!("rId{}", rel_idx);
    let mut rels_xml = pkg.get_text("word/_rels/document.xml.rels").unwrap_or_else(|_| {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.to_string()
    });

    while rels_xml.contains(&format!("\"{}\"", rel_id)) {
        rel_idx += 1;
        rel_id = format!("rId{}", rel_idx);
    }

    let rel_entry = format!(
        r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{}"/>"#,
        rel_id, media_target_rel
    );

    if let Some(pos) = rels_xml.rfind("</Relationships>") {
        let (head, tail) = rels_xml.split_at(pos);
        rels_xml = format!("{}{}{}", head, rel_entry, tail);
        pkg.set_text("word/_rels/document.xml.rels", rels_xml);
    }

    // 3. 注册 [Content_Types].xml 拓展名支持
    let mut ct_xml = pkg.get_text("[Content_Types].xml")?;
    let ext_tag = format!("Extension=\"{}\"", ext);
    if !ct_xml.contains(&ext_tag) {
        let mime = if ext == "jpg" || ext == "jpeg" {
            "image/jpeg"
        } else {
            "image/png"
        };
        let def_entry = format!(r#"<Default Extension="{}" ContentType="{}"/>"#, ext, mime);
        if let Some(pos) = ct_xml.find("</Types>") {
            let (head, tail) = ct_xml.split_at(pos);
            ct_xml = format!("{}{}{}", head, def_entry, tail);
            pkg.set_text("[Content_Types].xml", ct_xml);
        }
    }

    // 4. 构造 DrawingML 并在段落前插入
    let cx = (width_mm * 36000.0) as u64;
    let cy = (height_mm * 36000.0) as u64;
    let p_drawing_xml = build_drawing_p_xml(&rel_id, cx, cy);

    let doc_xml = pkg.get_text("word/document.xml")?;
    let updated_doc = insert_xml_before_para(&doc_xml, target_idx, &p_drawing_xml)?;

    pkg.set_text("word/document.xml", updated_doc);
    pkg.save_to_file(doc_path)?;

    Ok(MediaOpResponse {
        status: "success".to_string(),
        command: "insert-image".to_string(),
        file: doc_path.to_string(),
        affected_count: 1,
        message: format!("已成功插入图片 (rId: {}, 宽: {}mm, 高: {}mm)", rel_id, width_mm, height_mm),
    })
}

fn build_drawing_p_xml(rel_id: &str, cx: u64, cy: u64) -> String {
    format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Picture"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="Picture"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rel_id}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#,
        cx = cx,
        cy = cy,
        rel_id = rel_id
    )
}

fn insert_xml_before_para(doc_xml: &str, target_idx: usize, insert_content: &str) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(doc_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut inserted = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx && !inserted {
                    writer.write_event(Event::Text(BytesText::from_escaped(insert_content)))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    inserted = true;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                current_idx += 1;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    if !inserted {
        return Err(WordCliError::invalid_parameter(format!(
            "指定段落索引越界: 索引 {} 超出段落上限 {}",
            target_idx, current_idx
        )));
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}