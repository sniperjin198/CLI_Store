use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::io::Cursor;

use crate::error::WordCliError;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize)]
pub struct LayoutOpResponse {
    pub status: String,
    pub command: String,
    pub file: String,
    pub message: String,
}

/// 在指定段落前插入硬分页符
pub fn insert_page_break_before(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_idx: usize,
) -> Result<LayoutOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let mut reader = Reader::from_str(&doc_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut inserted = false;
    let break_p_xml = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"p" => {
                if current_idx == target_idx && !inserted {
                    writer
                        .write_event(Event::Text(BytesText::from_escaped(break_p_xml)))
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
            "分页符插入索引越界: 目标索引 {} 超出最大段落数 {}",
            target_idx, current_idx
        )));
    }

    let result = writer.into_inner().into_inner();
    let updated = String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))?;
    pkg.set_text("word/document.xml", updated);
    pkg.save_to_file(file_path)?;

    Ok(LayoutOpResponse {
        status: "success".to_string(),
        command: "break".to_string(),
        file: file_path.to_string(),
        message: format!("已在段落 {} 前插入分页符", target_idx),
    })
}

/// 配置文档页眉或页脚（支持文本、动态页码域、从第 1 页重新起算、解绑继承链）
pub fn set_header_or_footer(
    pkg: &mut DocxPackage,
    file_path: &str,
    is_header: bool,
    text: Option<&str>,
    page_num: bool,
    restart_page: Option<u32>,
    unlink: bool,
) -> Result<LayoutOpResponse, WordCliError> {
    let part_name = if is_header { "header1" } else { "footer1" };
    let xml_path = format!("word/{}.xml", part_name);
    let rel_type = if is_header {
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
    } else {
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
    };

    // 1. 若 unlink 为 true 且未提供内容，则从节中解绑
    if unlink && text.is_none() && !page_num {
        let mut doc_xml = pkg.get_text("word/document.xml")?;
        let tag_name = if is_header { "w:headerReference" } else { "w:footerReference" };
        if let Some(start) = doc_xml.find(&format!("<{}", tag_name)) {
            if let Some(end) = doc_xml[start..].find("/>") {
                doc_xml.replace_range(start..start + end + 2, "");
                pkg.set_text("word/document.xml", doc_xml);
                pkg.save_to_file(file_path)?;
                return Ok(LayoutOpResponse {
                    status: "success".to_string(),
                    command: if is_header { "header".to_string() } else { "footer".to_string() },
                    file: file_path.to_string(),
                    message: format!("已解除当前节的{}绑定", if is_header { "页眉" } else { "页脚" }),
                });
            }
        }
    }

    // 2. 生成 Header / Footer XML（支持动态页码域）
    let tag = if is_header { "hdr" } else { "ftr" };
    let mut inner_p = String::from(r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr>"#);

    if let Some(t) = text {
        inner_p.push_str(&format!(r#"<w:r><w:t xml:space="preserve">{} </w:t></w:r>"#, escape_xml(t)));
    }

    if page_num {
        // 动态页码域: <w:fldSimple w:instr="PAGE"/>
        inner_p.push_str(r#"<w:fldSimple w:instr="PAGE"><w:r><w:t>1</w:t></w:r></w:fldSimple>"#);
    }

    inner_p.push_str("</w:p>");

    let content_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}</w:{}>"#,
        tag, inner_p, tag
    );
    pkg.set_text(&xml_path, content_xml);

    // 3. 注册 Content Types
    let ct_type = if is_header {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"
    } else {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"
    };
    let mut ct_xml = pkg.get_text("[Content_Types].xml")?;
    let override_entry = format!(r#"<Override PartName="/{}" ContentType="{}"/>"#, xml_path, ct_type);
    if !ct_xml.contains(&format!("PartName=\"/{}\"", xml_path)) {
        if let Some(pos) = ct_xml.find("</Types>") {
            let (head, tail) = ct_xml.split_at(pos);
            ct_xml = format!("{}{}{}", head, override_entry, tail);
            pkg.set_text("[Content_Types].xml", ct_xml);
        }
    }

    // 4. 注册 Relationships
    let rid = if is_header { "rIdHdr1" } else { "rIdFtr1" };
    let mut rels_xml = pkg.get_text("word/_rels/document.xml.rels")?;
    let rel_entry = format!(
        r#"<Relationship Id="{}" Type="{}" Target="{}.xml"/>"#,
        rid, rel_type, part_name
    );
    if !rels_xml.contains(&format!("\"{}\"", rid)) {
        if let Some(pos) = rels_xml.rfind("</Relationships>") {
            let (head, tail) = rels_xml.split_at(pos);
            rels_xml = format!("{}{}{}", head, rel_entry, tail);
            pkg.set_text("word/_rels/document.xml.rels", rels_xml);
        }
    }

    // 5. 挂载到 document.xml 中的 sectPr 并处理 restart 页码
    let mut doc_xml = pkg.get_text("word/document.xml")?;
    let ref_tag = if is_header {
        format!(r#"<w:headerReference w:type="default" r:id="{}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>"#, rid)
    } else {
        format!(r#"<w:footerReference w:type="default" r:id="{}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>"#, rid)
    };

    if !doc_xml.contains(&format!("r:id=\"{}\"", rid)) {
        if let Some(pos) = doc_xml.rfind("<w:sectPr") {
            let insert_at = doc_xml[pos..].find('>').map(|idx| pos + idx + 1).unwrap_or(pos);
            let (head, tail) = doc_xml.split_at(insert_at);
            doc_xml = format!("{}{}{}", head, ref_tag, tail);
        }
    }

    // 处理起始页码重新起算（例如正文第1页开始）
    if let Some(start_num) = restart_page {
        let pg_num_type = format!(r#"<w:pgNumType w:start="{}"/>"#, start_num);
        if let Some(pos) = doc_xml.rfind("<w:sectPr") {
            let insert_at = doc_xml[pos..].find('>').map(|idx| pos + idx + 1).unwrap_or(pos);
            let (head, tail) = doc_xml.split_at(insert_at);
            doc_xml = format!("{}{}{}", head, pg_num_type, tail);
        }
    }

    pkg.set_text("word/document.xml", doc_xml);
    pkg.save_to_file(file_path)?;

    Ok(LayoutOpResponse {
        status: "success".to_string(),
        command: if is_header { "header".to_string() } else { "footer".to_string() },
        file: file_path.to_string(),
        message: format!(
            "已成功配置{} (动态页码: {}, 起始页码: {:?})",
            if is_header { "页眉" } else { "页脚" },
            page_num,
            restart_page
        ),
    })
}