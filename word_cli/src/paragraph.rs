use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::io::Cursor;

use crate::error::WordCliError;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize)]
pub struct ParaOpResponse {
    pub status: String,
    pub command: String,
    pub affected_index: usize,
    pub file: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MergeResponse {
    pub status: String,
    pub command: String,
    pub base_file: String,
    pub appended_file: String,
    pub message: String,
}

/// 在指定段落索引前插入新段落
pub fn insert_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_idx: usize,
    text: &str,
    style_id: Option<&str>,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let new_p_xml = build_paragraph_xml(text, style_id);
    let updated_xml = insert_p_at_index(&doc_xml, target_idx, &new_p_xml)?;

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "insert-para".to_string(),
        affected_index: target_idx,
        file: file_path.to_string(),
        message: format!("已在段落索引 {} 前插入新段落", target_idx),
    })
}

/// 在正文末尾追加段落
pub fn append_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    text: &str,
    style_id: Option<&str>,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let new_p_xml = build_paragraph_xml(text, style_id);

    let sect_tag = "<w:sectPr";
    let body_end_tag = "</w:body>";

    let updated_xml = if let Some(pos) = doc_xml.rfind(sect_tag) {
        let (head, tail) = doc_xml.split_at(pos);
        format!("{}{}{}", head, new_p_xml, tail)
    } else if let Some(pos) = doc_xml.rfind(body_end_tag) {
        let (head, tail) = doc_xml.split_at(pos);
        format!("{}{}{}", head, new_p_xml, tail)
    } else {
        return Err(WordCliError::corrupted_document("未找到有效的正文结构"));
    };

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "append-para".to_string(),
        affected_index: 0,
        file: file_path.to_string(),
        message: "已在文档末尾追加段落".to_string(),
    })
}

/// 删除指定索引段落
pub fn delete_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_idx: usize,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let updated_xml = remove_p_at_index(&doc_xml, target_idx)?;

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "delete-para".to_string(),
        affected_index: target_idx,
        file: file_path.to_string(),
        message: format!("已删除索引为 {} 的段落", target_idx),
    })
}

/// 格式调整：支持首行缩进、行距、字体名称、字号(pt)、加粗及野格式刷平
pub fn format_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_idx: usize,
    indent_chars: Option<f64>,
    line_spacing_times: Option<f64>,
    font: Option<&str>,
    size_pt: Option<f64>,
    bold: Option<bool>,
    strip_formatting: bool,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let updated_xml = update_p_format(
        &doc_xml,
        target_idx,
        indent_chars,
        line_spacing_times,
        font,
        size_pt,
        bold,
        strip_formatting,
    )?;

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "para-format".to_string(),
        affected_index: target_idx,
        file: file_path.to_string(),
        message: format!("已更新索引为 {} 的段落格式", target_idx),
    })
}

/// 套用母版样式
pub fn apply_style_to_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_idx: usize,
    style_id: &str,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let updated_xml = update_p_style(&doc_xml, target_idx, style_id)?;

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "apply-style".to_string(),
        affected_index: target_idx,
        file: file_path.to_string(),
        message: format!("已将段落 {} 样式设置为 [{}]", target_idx, style_id),
    })
}

/// 外部文档追加合并
pub fn merge_documents(
    base_pkg: &mut DocxPackage,
    base_path: &str,
    append_path: &str,
) -> Result<MergeResponse, WordCliError> {
    let append_pkg = DocxPackage::from_file(append_path)?;
    let append_xml = append_pkg.get_text("word/document.xml")?;

    let body_start = append_xml.find("<w:body>").ok_or_else(|| {
        WordCliError::corrupted_document("待合并文档缺少 <w:body> 节点")
    })? + 8;

    let body_end = if let Some(sect) = append_xml.rfind("<w:sectPr") {
        sect
    } else if let Some(bend) = append_xml.rfind("</w:body>") {
        bend
    } else {
        append_xml.len()
    };

    let append_content = &append_xml[body_start..body_end];

    let base_xml = base_pkg.get_text("word/document.xml")?;
    let insert_pos = if let Some(sect) = base_xml.rfind("<w:sectPr") {
        sect
    } else if let Some(bend) = base_xml.rfind("</w:body>") {
        bend
    } else {
        return Err(WordCliError::corrupted_document("主文档结构异常"));
    };

    let (head, tail) = base_xml.split_at(insert_pos);
    let merged_xml = format!("{}{}{}", head, append_content, tail);

    base_pkg.set_text("word/document.xml", merged_xml);
    base_pkg.save_to_file(base_path)?;

    Ok(MergeResponse {
        status: "success".to_string(),
        command: "merge".to_string(),
        base_file: base_path.to_string(),
        appended_file: append_path.to_string(),
        message: "文档合并完成".to_string(),
    })
}

fn build_paragraph_xml(text: &str, style_id: Option<&str>) -> String {
    let mut style_xml = String::new();
    if let Some(s) = style_id {
        style_xml = format!(r#"<w:pStyle w:val="{}"/>"#, s);
    }
    format!(
        r#"<w:p><w:pPr>{}</w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
        style_xml,
        escape_xml(text)
    )
}

fn insert_p_at_index(xml: &str, target_idx: usize, new_p_xml: &str) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut inserted = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx && !inserted {
                    writer
                        .write_event(Event::Text(quick_xml::events::BytesText::from_escaped(
                            new_p_xml,
                        )))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    inserted = true;
                }
                writer
                    .write_event(Event::Start(e.clone()))
                    .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                writer
                    .write_event(Event::End(e.clone()))
                    .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                current_idx += 1;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer
                    .write_event(e)
                    .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    if !inserted {
        return Err(WordCliError::invalid_parameter(format!(
            "段落索引越界: target_idx ({}) 超出文档最大段落数 ({})",
            target_idx, current_idx
        )));
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}

fn remove_p_at_index(xml: &str, target_idx: usize) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut deleting = false;
    let mut found = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx {
                    deleting = true;
                    found = true;
                } else {
                    writer
                        .write_event(Event::Start(e.clone()))
                        .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                if deleting {
                    deleting = false;
                } else {
                    writer
                        .write_event(Event::End(e.clone()))
                        .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
                current_idx += 1;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !deleting {
                    writer
                        .write_event(e)
                        .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    if !found {
        return Err(WordCliError::invalid_parameter(format!(
            "要删除的段落索引不存在: {}",
            target_idx
        )));
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}

fn update_p_style(xml: &str, target_idx: usize, style_id: &str) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut in_target_p = false;
    let mut has_p_pr = false;
    let mut written_style = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx {
                    in_target_p = true;
                    has_p_pr = false;
                    written_style = false;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_target_p && e.local_name().as_ref() == b"pPr" => {
                has_p_pr = true;
                writer.write_event(Event::Start(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                let p_style = format!(r#"<w:pStyle w:val="{}"/>"#, style_id);
                writer.write_event(Event::Text(quick_xml::events::BytesText::from_escaped(p_style))).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                written_style = true;
            }
            Ok(Event::Empty(ref e)) if in_target_p && e.local_name().as_ref() == b"pStyle" => {}
            Ok(Event::Start(ref e)) if in_target_p && !has_p_pr && !written_style && e.local_name().as_ref() != b"pPr" => {
                let p_pr = format!(r#"<w:pPr><w:pStyle w:val="{}"/></w:pPr>"#, style_id);
                writer.write_event(Event::Text(quick_xml::events::BytesText::from_escaped(p_pr))).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                written_style = true;
                writer.write_event(Event::Start(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                in_target_p = false;
                writer.write_event(Event::End(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
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

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}

fn update_p_format(
    xml: &str,
    target_idx: usize,
    indent_chars: Option<f64>,
    line_spacing_times: Option<f64>,
    font: Option<&str>,
    size_pt: Option<f64>,
    bold: Option<bool>,
    strip_formatting: bool,
) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut in_target_p = false;
    let mut in_run = false;
    let mut in_run_pr = false;

    // 构造运行级属性（字体、字号、加粗）
    let mut rpr_frag = String::new();
    if let Some(f) = font {
        rpr_frag.push_str(&format!(
            r#"<w:rFonts w:ascii="{0}" w:hAnsi="{0}" w:eastAsia="{0}"/>"#,
            escape_xml(f)
        ));
    }
    if let Some(pt) = size_pt {
        let half_pts = (pt * 2.0) as u32;
        rpr_frag.push_str(&format!(
            r#"<w:sz w:val="{0}"/><w:szCs w:val="{0}"/>"#,
            half_pts
        ));
    }
    if let Some(b) = bold {
        if b {
            rpr_frag.push_str("<w:b/><w:bCs/>");
        }
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx {
                    in_target_p = true;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;

                if in_target_p {
                    let mut format_xml = String::new();
                    if let Some(ic) = indent_chars {
                        let val = (ic * 100.0) as u32;
                        format_xml.push_str(&format!(r#"<w:ind w:firstLineChars="{}"/>"#, val));
                    }
                    if let Some(ls) = line_spacing_times {
                        let val = (ls * 240.0) as u32;
                        format_xml.push_str(&format!(r#"<w:spacing w:line="{}" w:lineRule="auto"/>"#, val));
                    }
                    if !format_xml.is_empty() {
                        let p_pr_frag = format!("<w:pPr>{}</w:pPr>", format_xml);
                        writer.write_event(Event::Text(BytesText::from_escaped(p_pr_frag)))
                            .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                    }
                }
            }
            Ok(Event::Start(ref e)) if in_target_p && e.local_name().as_ref() == b"r" => {
                in_run = true;
                writer.write_event(Event::Start(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                // 若指定了字体/字号/加粗，注入到 run 开头
                if !rpr_frag.is_empty() {
                    let full_rpr = format!("<w:rPr>{}</w:rPr>", rpr_frag);
                    writer.write_event(Event::Text(BytesText::from_escaped(full_rpr)))
                        .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Ok(Event::Start(ref e)) if in_target_p && in_run && e.local_name().as_ref() == b"rPr" => {
                in_run_pr = true;
            }
            Ok(Event::End(ref e)) if in_target_p && in_run && e.local_name().as_ref() == b"rPr" => {
                in_run_pr = false;
            }
            Ok(Event::Start(_)) | Ok(Event::Empty(_)) if in_target_p && in_run_pr => {
                // 若开启刷平或注入了新 rPr，过滤原有的 rPr 子节点
                if !rpr_frag.is_empty() || strip_formatting {
                    // 吞掉旧属性
                } else {
                    // 保留
                }
            }
            Ok(Event::End(ref e)) if in_target_p && e.local_name().as_ref() == b"r" => {
                in_run = false;
                writer.write_event(Event::End(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                in_target_p = false;
                writer.write_event(Event::End(e.clone())).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                current_idx += 1;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !(in_target_p && in_run_pr && (!rpr_frag.is_empty() || strip_formatting)) {
                    writer.write_event(e).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}