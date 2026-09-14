use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::collections::HashSet;
use std::io::Cursor;

use crate::error::WordCliError;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize)]
pub struct ParaOpResponse {
    pub status: String,
    pub command: String,
    pub affected_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_count: Option<usize>,
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
        affected_count: Some(1),
        file: file_path.to_string(),
        message: format!("已在段落索引 {} 前插入新段落", target_idx),
    })
}

/// 在正文末尾追加段落（安全定位 </w:body> 前）
pub fn append_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    text: &str,
    style_id: Option<&str>,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let new_p_xml = build_paragraph_xml(text, style_id);

    // 严谨定位：始终插入在文档最后一个节属性 <w:sectPr 前；若无则插入在 </w:body> 前
    let updated_xml = if let Some(body_end) = doc_xml.rfind("</w:body>") {
        let before_body_end = &doc_xml[..body_end];
        if let Some(sect_pos) = before_body_end.rfind("<w:sectPr") {
            // 确保找到的是顶层的 sectPr，而不是 pPr 里的行内分节
            let (head, tail) = doc_xml.split_at(sect_pos);
            format!("{}{}{}", head, new_p_xml, tail)
        } else {
            let (head, tail) = doc_xml.split_at(body_end);
            format!("{}{}{}", head, new_p_xml, tail)
        }
    } else {
        return Err(WordCliError::corrupted_document("未找到有效的 </w:body> 结构"));
    };

    pkg.set_text("word/document.xml", updated_xml);
    pkg.save_to_file(file_path)?;

    Ok(ParaOpResponse {
        status: "success".to_string(),
        command: "append-para".to_string(),
        affected_index: 0,
        affected_count: Some(1),
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
        affected_count: Some(1),
        file: file_path.to_string(),
        message: format!("已删除索引为 {} 的段落", target_idx),
    })
}

/// 格式调整：支持单个索引或批量范围，智能过滤标题保护正文
pub fn format_paragraph(
    pkg: &mut DocxPackage,
    file_path: &str,
    index: Option<usize>,
    range: Option<&str>,
    target: &str,
    indent_chars: Option<f64>,
    line_spacing_times: Option<f64>,
    font: Option<&str>,
    size_pt: Option<f64>,
    bold: Option<bool>,
    strip_formatting: bool,
) -> Result<ParaOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;

    // 1. 先统计所有段落的总数与各段落的样式（用于过滤标题）
    let (total_paras, heading_indices) = scan_paragraphs_and_headings(&doc_xml)?;

    // 2. 解析待处理的段落下标集合
    let mut target_set = HashSet::new();
    if let Some(single) = index {
        target_set.insert(single);
    } else if let Some(r) = range {
        if r.eq_ignore_ascii_case("all") {
            for i in 0..total_paras {
                target_set.insert(i);
            }
        } else if let Some((start_s, end_s)) = r.split_once(':') {
            let start = start_s.parse::<usize>().unwrap_or(0);
            let end = end_s.parse::<usize>().unwrap_or(total_paras.saturating_sub(1));
            for i in start..=end.min(total_paras.saturating_sub(1)) {
                target_set.insert(i);
            }
        } else if let Ok(idx) = r.parse::<usize>() {
            target_set.insert(idx);
        }
    } else {
        return Err(WordCliError::invalid_parameter(
            "必须提供 --index 指定单段，或提供 --range (如 '0:25' / 'all') 进行批量排版",
        ));
    }

    // 3. 若 target 为 "body"（默认），自动滤除大纲标题段落
    if target.eq_ignore_ascii_case("body") {
        for h_idx in &heading_indices {
            target_set.remove(h_idx);
        }
    }

    if target_set.is_empty() {
        return Ok(ParaOpResponse {
            status: "success".to_string(),
            command: "para-format".to_string(),
            affected_index: index.unwrap_or(0),
            affected_count: Some(0),
            file: file_path.to_string(),
            message: "未匹配到需要格式化的正文段落（可能目标范围内全为标题段落）".to_string(),
        });
    }

    let affected_count = target_set.len();
    let min_index = *target_set.iter().min().unwrap_or(&0);

    // 4. 执行批量 XML 流式注入重构
    let updated_xml = batch_update_p_format(
        &doc_xml,
        &target_set,
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
        affected_index: min_index,
        affected_count: Some(affected_count),
        file: file_path.to_string(),
        message: format!(
            "已成功批量更新 {} 个段落的格式（范围模式: target={}, 已保护标题）",
            affected_count, target
        ),
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
        affected_count: Some(1),
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

/// 快速预扫描：获取总段落数与所有标题段落的下标集合
fn scan_paragraphs_and_headings(xml: &str) -> Result<(usize, HashSet<usize>), WordCliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut count = 0;
    let mut in_para = false;
    let mut headings = HashSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"p" => {
                in_para = true;
            }
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if in_para && e.local_name().as_ref() == b"pStyle" => {
                for attr in e.attributes().flatten() {
                    if attr.key.local_name().as_ref() == b"val" {
                        let val = String::from_utf8_lossy(&attr.value).to_lowercase();
                        if val.contains("heading") || val.contains("title") || val.contains("标题") {
                            headings.insert(count);
                        }
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => {
                in_para = false;
                count += 1;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok((count, headings))
}

/// 批量流式重写段落格式
fn batch_update_p_format(
    xml: &str,
    target_indices: &HashSet<usize>,
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
                if target_indices.contains(&current_idx) {
                    in_target_p = true;
                }
                writer
                    .write_event(Event::Start(e.clone()))
                    .map_err(|err| WordCliError::xml_parse(err.to_string()))?;

                if in_target_p {
                    let mut format_xml = String::new();
                    if let Some(ic) = indent_chars {
                        let val = (ic * 100.0) as u32;
                        format_xml.push_str(&format!(r#"<w:ind w:firstLineChars="{}"/>"#, val));
                    }
                    if let Some(ls) = line_spacing_times {
                        let val = (ls * 240.0) as u32;
                        format_xml.push_str(&format!(
                            r#"<w:spacing w:line="{}" w:lineRule="auto"/>"#,
                            val
                        ));
                    }
                    if !format_xml.is_empty() {
                        let p_pr_frag = format!("<w:pPr>{}</w:pPr>", format_xml);
                        writer
                            .write_event(Event::Text(BytesText::from_escaped(p_pr_frag)))
                            .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                    }
                }
            }
            Ok(Event::Start(ref e)) if in_target_p && e.local_name().as_ref() == b"r" => {
                in_run = true;
                writer
                    .write_event(Event::Start(e.clone()))
                    .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                if !rpr_frag.is_empty() {
                    let full_rpr = format!("<w:rPr>{}</w:rPr>", rpr_frag);
                    writer
                        .write_event(Event::Text(BytesText::from_escaped(full_rpr)))
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
                // 开启刷平或注入新属性时吞掉旧属性
                if !rpr_frag.is_empty() || strip_formatting {
                } else {
                }
            }
            Ok(Event::End(ref e)) if in_target_p && e.local_name().as_ref() == b"r" => {
                in_run = false;
                writer
                    .write_event(Event::End(e.clone()))
                    .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"p" => {
                in_target_p = false;
                writer
                    .write_event(Event::End(e.clone()))
                    .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                current_idx += 1;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !(in_target_p && in_run_pr && (!rpr_frag.is_empty() || strip_formatting)) {
                    writer
                        .write_event(e)
                        .map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))
}