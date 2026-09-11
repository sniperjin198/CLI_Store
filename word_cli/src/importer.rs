use pulldown_cmark::{Event as MdEvent, Parser, Tag};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::error::WordCliError;
use crate::lifecycle::create_document;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub status: String,
    pub command: String,
    pub input_file: String,
    pub output_file: String,
    pub template_used: Option<String>,
    pub message: String,
}

pub fn import_markdown(
    input_path: &str,
    output_path: &str,
    template_path: Option<&str>,
) -> Result<ImportResponse, WordCliError> {
    let in_p = Path::new(input_path);
    if !in_p.exists() {
        return Err(WordCliError::not_found(format!(
            "输入的 Markdown/TXT 文件不存在: {}",
            input_path
        )));
    }

    let md_text = fs::read_to_string(in_p).map_err(|e| {
        WordCliError::file_io(format!("无法读取输入文件: {}", e))
    })?;

    // 1. 先用 template 或空白模板生成基础 docx 到 output_path
    create_document(output_path, template_path)?;

    // 2. 解包加载刚生成的文件
    let mut pkg = DocxPackage::from_file(output_path)?;

    // 3. 将 Markdown 转换为 OpenXML 片段
    let body_xml = markdown_to_openxml(&md_text)?;

    // 4. 将生成的 XML 注入到 word/document.xml 中
    inject_body_into_document(&mut pkg, &body_xml)?;

    // 5. 保存回文件
    pkg.save_to_file(output_path)?;

    Ok(ImportResponse {
        status: "success".to_string(),
        command: "import".to_string(),
        input_file: input_path.to_string(),
        output_file: output_path.to_string(),
        template_used: template_path.map(|s| s.to_string()),
        message: "已成功将 Markdown/TXT 内容转换为 Word 文档".to_string(),
    })
}

fn markdown_to_openxml(md_content: &str) -> Result<String, WordCliError> {
    let mut options = pulldown_cmark::Options::empty();
			options.insert(pulldown_cmark::Options::ENABLE_TABLES);
			options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
	let parser = Parser::new_ext(md_content, options);
    let mut out = String::new();

    let mut in_heading: Option<u32> = None;
    let mut in_paragraph = false;
    let mut is_bold = false;
    let mut is_italic = false;

    // 表格状态，显式声明 Vec<String> 避免类型推导异常
    let mut in_table = false;
    let mut in_table_head = false;
    let mut current_cell_xml = String::new();
    let mut current_row_cells: Vec<String> = Vec::new();

    for event in parser {
        match event {
            MdEvent::Start(Tag::Heading(level, _, _)) => {
                let lvl = level as u32;
                in_heading = Some(lvl);
                out.push_str(&format!(
                    r#"<w:p><w:pPr><w:pStyle w:val="Heading{}"/></w:pPr>"#,
                    lvl
                ));
            }
            MdEvent::End(Tag::Heading(_, _, _)) => {
                in_heading = None;
                out.push_str("</w:p>");
            }
            MdEvent::Start(Tag::Paragraph) => {
                if !in_table {
                    in_paragraph = true;
                    out.push_str(r#"<w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr>"#);
                }
            }
            MdEvent::End(Tag::Paragraph) => {
                if !in_table {
                    in_paragraph = false;
                    out.push_str("</w:p>");
                }
            }
            MdEvent::Start(Tag::Strong) => {
                is_bold = true;
            }
            MdEvent::End(Tag::Strong) => {
                is_bold = false;
            }
            MdEvent::Start(Tag::Emphasis) => {
                is_italic = true;
            }
            MdEvent::End(Tag::Emphasis) => {
                is_italic = false;
            }
            MdEvent::Start(Tag::Table(_)) => {
                in_table = true;
                out.push_str(
                    r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr>"#,
                );
            }
            MdEvent::End(Tag::Table(_)) => {
                in_table = false;
                out.push_str("</w:tbl>");
            }
            MdEvent::Start(Tag::TableHead) => {
                in_table_head = true;
                current_row_cells.clear();
            }
            MdEvent::End(Tag::TableHead) => {
                in_table_head = false;
                out.push_str("<w:tr>");
                for cell in &current_row_cells {
                    out.push_str(cell);
                }
                out.push_str("</w:tr>");
                current_row_cells.clear();
            }
            MdEvent::Start(Tag::TableRow) => {
                current_row_cells.clear();
            }
            MdEvent::End(Tag::TableRow) => {
                out.push_str("<w:tr>");
                for cell in &current_row_cells {
                    out.push_str(cell);
                }
                out.push_str("</w:tr>");
                current_row_cells.clear();
            }
            MdEvent::Start(Tag::TableCell) => {
                current_cell_xml.clear();
            }
            MdEvent::End(Tag::TableCell) => {
                let cell_final = format!(
                    r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p>{}</w:p></w:tc>"#,
                    current_cell_xml
                );
                current_row_cells.push(cell_final);
                current_cell_xml.clear();
            }
            MdEvent::Text(text) => {
                let escaped = escape_xml(&text);
                let run_xml = build_run_xml(&escaped, is_bold, is_italic, in_table_head);
                if in_table {
                    current_cell_xml.push_str(&run_xml);
                } else if in_heading.is_some() || in_paragraph {
                    out.push_str(&run_xml);
                } else {
                    out.push_str(&format!(r#"<w:p>{}</w:p>"#, run_xml));
                }
            }
            MdEvent::Code(code) => {
                let escaped = escape_xml(&code);
                let run_xml = format!(
                    r#"<w:r><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
                    escaped
                );
                if in_table {
                    current_cell_xml.push_str(&run_xml);
                } else {
                    out.push_str(&run_xml);
                }
            }
            _ => {}
        }
    }

    Ok(out)
}

fn build_run_xml(escaped_text: &str, bold: bool, italic: bool, table_head: bool) -> String {
    let mut rpr = String::new();
    if bold || table_head {
        rpr.push_str("<w:b/>");
    }
    if italic {
        rpr.push_str("<w:i/>");
    }

    if rpr.is_empty() {
        format!(
            r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
            escaped_text
        )
    } else {
        format!(
            r#"<w:r><w:rPr>{}</w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
            rpr, escaped_text
        )
    }
}

fn inject_body_into_document(pkg: &mut DocxPackage, new_xml: &str) -> Result<(), WordCliError> {
    let orig = pkg.get_text("word/document.xml")?;

    let end_body_tag = "</w:body>";
    let sect_pr_tag = "<w:sectPr";

    if let Some(pos) = orig.rfind(sect_pr_tag) {
        let (head, tail) = orig.split_at(pos);
        let updated = format!("{}{}{}", head, new_xml, tail);
        pkg.set_text("word/document.xml", updated);
    } else if let Some(pos) = orig.rfind(end_body_tag) {
        let (head, tail) = orig.split_at(pos);
        let updated = format!("{}{}{}", head, new_xml, tail);
        pkg.set_text("word/document.xml", updated);
    } else {
        return Err(WordCliError::corrupted_document(
            "文档缺少 </w:body> 闭合标签",
        ));
    }

    Ok(())
}