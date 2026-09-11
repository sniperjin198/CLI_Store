use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::io::Cursor;

use crate::error::WordCliError;
use crate::package::DocxPackage;

#[derive(Debug, Serialize)]
pub struct PageSetupResponse {
    pub status: String,
    pub command: String,
    pub file: String,
    pub orientation: String,
    pub paper_size: String,
    pub message: String,
}

/// 调整文档（默认全局末尾节，或指定分节）的页面纸张尺寸与横纵方向
pub fn setup_page_layout(
    pkg: &mut DocxPackage,
    file_path: &str,
    orientation: &str,
    paper_size: &str,
) -> Result<PageSetupResponse, WordCliError> {
    let is_landscape = matches!(orientation.to_lowercase().as_str(), "landscape" | "horizontal" | "横向");
    
    // A4 标准规范 (单位: dxa, 1/20 pt)
    // 纵向: 宽 11906, 高 16838; 横向: 宽 16838, 高 11906
    let (width, height) = match paper_size.to_uppercase().as_str() {
        "A3" => {
            if is_landscape { (23811, 16838) } else { (16838, 23811) }
        }
        _ => {
            // 默认 A4
            if is_landscape { (16838, 11906) } else { (11906, 16838) }
        }
    };

    let pgsz_xml = if is_landscape {
        format!(r#"<w:pgSz w:w="{}" w:h="{}" w:orient="landscape"/>"#, width, height)
    } else {
        format!(r#"<w:pgSz w:w="{}" w:h="{}"/>"#, width, height)
    };

    let doc_xml = pkg.get_text("word/document.xml")?;
    let updated_doc = update_sect_pgsz(&doc_xml, &pgsz_xml)?;

    pkg.set_text("word/document.xml", updated_doc);
    pkg.save_to_file(file_path)?;

    Ok(PageSetupResponse {
        status: "success".to_string(),
        command: "page".to_string(),
        file: file_path.to_string(),
        orientation: if is_landscape { "landscape".to_string() } else { "portrait".to_string() },
        paper_size: paper_size.to_uppercase(),
        message: format!(
            "已成功将页面方向设置为 [{}], 纸张规格 [{}]",
            if is_landscape { "横向 (landscape)" } else { "纵向 (portrait)" },
            paper_size.to_uppercase()
        ),
    })
}

fn update_sect_pgsz(xml: &str, new_pgsz_xml: &str) -> Result<String, WordCliError> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_sect_pr = false;
    let mut written_pgsz = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"sectPr" => {
                in_sect_pr = true;
                written_pgsz = false;
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_sect_pr && e.local_name().as_ref() == b"pgSz" => {
                // 吞掉旧的 pgSz 起始
            }
            Ok(Event::Empty(ref e)) if in_sect_pr && e.local_name().as_ref() == b"pgSz" => {
                // 吞掉旧的自闭合 pgSz
            }
            Ok(Event::End(ref e)) if in_sect_pr && e.local_name().as_ref() == b"pgSz" => {
                // 吞掉旧的 pgSz 闭合
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"sectPr" => {
                if !written_pgsz {
                    writer
                        .write_event(Event::Text(BytesText::from_escaped(new_pgsz_xml)))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    written_pgsz = true;
                }
                in_sect_pr = false;
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
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