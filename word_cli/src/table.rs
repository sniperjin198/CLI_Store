use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::Serialize;
use std::io::Cursor;

use crate::error::WordCliError;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize)]
pub struct TableDataResponse {
    pub status: String,
    pub table_index: usize,
    pub total_rows: usize,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TableOpResponse {
    pub status: String,
    pub command: String,
    pub table_index: usize,
    pub file: String,
    pub message: String,
}

/// 读取指定表格的二维数据，支持 json / markdown / csv 格式输出
pub fn read_table_data(
    pkg: &DocxPackage,
    target_tbl_idx: usize,
    format_type: &str,
) -> Result<TableDataResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let mut reader = Reader::from_str(&doc_xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut current_tbl_idx = 0;
    let mut in_target_tbl = false;

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell_text = String::new();
    let mut in_cell = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"tbl" => {
                    if current_tbl_idx == target_tbl_idx {
                        in_target_tbl = true;
                    }
                }
                b"tr" if in_target_tbl => {
                    current_row.clear();
                }
                b"tc" if in_target_tbl => {
                    in_cell = true;
                    current_cell_text.clear();
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) if in_target_tbl && in_cell => {
                current_cell_text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"tc" if in_target_tbl => {
                    in_cell = false;
                    current_row.push(current_cell_text.trim().to_string());
                }
                b"tr" if in_target_tbl => {
                    rows.push(current_row.clone());
                }
                b"tbl" => {
                    if in_target_tbl {
                        break;
                    }
                    current_tbl_idx += 1;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    if !in_target_tbl {
        return Err(WordCliError::not_found(format!(
            "未找到索引为 {} 的表格 (当前文档最大表格数: {})",
            target_tbl_idx, current_tbl_idx
        )));
    }

    match format_type.to_lowercase().as_str() {
        "markdown" | "md" => {
            let md_str = format_as_markdown(&rows);
            Ok(TableDataResponse {
                status: "success".to_string(),
                table_index: target_tbl_idx,
                total_rows: rows.len(),
                format: "markdown".to_string(),
                data: None,
                content: Some(md_str),
            })
        }
        "csv" => {
            let csv_str = format_as_csv(&rows);
            Ok(TableDataResponse {
                status: "success".to_string(),
                table_index: target_tbl_idx,
                total_rows: rows.len(),
                format: "csv".to_string(),
                data: None,
                content: Some(csv_str),
            })
        }
        _ => Ok(TableDataResponse {
            status: "success".to_string(),
            table_index: target_tbl_idx,
            total_rows: rows.len(),
            format: "json".to_string(),
            data: Some(rows),
            content: None,
        }),
    }
}

fn format_as_markdown(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    // 表头
    let header = &rows[0];
    out.push_str(&format!("| {} |\n", header.join(" | ")));
    // 分割线
    let sep: Vec<String> = header.iter().map(|_| "---".to_string()).collect();
    out.push_str(&format!("| {} |\n", sep.join(" | ")));
    // 数据行
    for row in rows.iter().skip(1) {
        out.push_str(&format!("| {} |\n", row.join(" | ")));
    }
    out
}

fn format_as_csv(rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    for row in rows {
        let escaped_cells: Vec<String> = row
            .iter()
            .map(|cell| {
                if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                    format!("\"{}\"", cell.replace('"', "\"\""))
                } else {
                    cell.clone()
                }
            })
            .collect();
        out.push_str(&escaped_cells.join(","));
        out.push('\n');
    }
    out
}

/// 修改指定单元格的内容
pub fn set_cell_value(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_tbl_idx: usize,
    row: usize,
    col: usize,
    value: &str,
) -> Result<TableOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let mut reader = Reader::from_str(&doc_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_tbl = 0;
    let mut in_target_tbl = false;
    let mut current_row = 0;
    let mut current_col = 0;
    let mut in_target_cell = false;
    let mut target_cell_written = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if current_tbl == target_tbl_idx {
                    in_target_tbl = true;
                    current_row = 0;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                current_col = 0;
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tc" => {
                if current_row == row && current_col == col {
                    in_target_cell = true;
                    let cell_xml = format!(
                        r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p></w:tc>"#,
                        escape_xml(value)
                    );
                    writer.write_event(Event::Text(BytesText::from_escaped(cell_xml)))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    target_cell_written = true;
                } else {
                    writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                }
            }
            Ok(Event::End(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tc" => {
                if in_target_cell {
                    in_target_cell = false;
                } else {
                    writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                }
                current_col += 1;
            }
            Ok(Event::End(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                current_row += 1;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if in_target_tbl {
                    in_target_tbl = false;
                }
                current_tbl += 1;
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !in_target_cell {
                    writer.write_event(e).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    if !target_cell_written {
        return Err(WordCliError::invalid_parameter(format!(
            "未找到目标单元格 [行:{}, 列:{}]",
            row, col
        )));
    }

    let result = writer.into_inner().into_inner();
    let updated = String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))?;
    pkg.set_text("word/document.xml", updated);
    pkg.save_to_file(file_path)?;

    Ok(TableOpResponse {
        status: "success".to_string(),
        command: "set-cell".to_string(),
        table_index: target_tbl_idx,
        file: file_path.to_string(),
        message: format!("已成功更新单元格 [{}, {}] 内容", row, col),
    })
}

/// 在指定表格指定行前插入新行
pub fn insert_table_row(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_tbl_idx: usize,
    target_row_idx: usize,
    cells: &[String],
) -> Result<TableOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let mut reader = Reader::from_str(&doc_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_tbl = 0;
    let mut in_target_tbl = false;
    let mut current_row = 0;
    let mut row_inserted = false;

    let new_row_xml = build_row_xml(cells);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if current_tbl == target_tbl_idx {
                    in_target_tbl = true;
                    current_row = 0;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                if current_row == target_row_idx && !row_inserted {
                    writer.write_event(Event::Text(BytesText::from_escaped(&new_row_xml)))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    row_inserted = true;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::End(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                current_row += 1;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if in_target_tbl && !row_inserted && current_row == target_row_idx {
                    writer.write_event(Event::Text(BytesText::from_escaped(&new_row_xml)))
                        .map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                    row_inserted = true;
                }
                if in_target_tbl {
                    in_target_tbl = false;
                }
                current_tbl += 1;
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

    if !row_inserted {
        return Err(WordCliError::invalid_parameter(format!(
            "插入行索引越界: 行索引 {} 超出表格总行数 {}",
            target_row_idx, current_row
        )));
    }

    let result = writer.into_inner().into_inner();
    let updated = String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))?;
    pkg.set_text("word/document.xml", updated);
    pkg.save_to_file(file_path)?;

    Ok(TableOpResponse {
        status: "success".to_string(),
        command: "insert-row".to_string(),
        table_index: target_tbl_idx,
        file: file_path.to_string(),
        message: format!("已在表格 {} 的行 {} 插入新行", target_tbl_idx, target_row_idx),
    })
}

/// 删除指定表格中的指定行
pub fn delete_table_row(
    pkg: &mut DocxPackage,
    file_path: &str,
    target_tbl_idx: usize,
    target_row_idx: usize,
) -> Result<TableOpResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let mut reader = Reader::from_str(&doc_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut current_tbl = 0;
    let mut in_target_tbl = false;
    let mut current_row = 0;
    let mut in_deleting_row = false;
    let mut deleted = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if current_tbl == target_tbl_idx {
                    in_target_tbl = true;
                    current_row = 0;
                }
                writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Start(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                if current_row == target_row_idx {
                    in_deleting_row = true;
                    deleted = true;
                } else {
                    writer.write_event(Event::Start(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                }
            }
            Ok(Event::End(ref e)) if in_target_tbl && e.local_name().as_ref() == b"tr" => {
                if in_deleting_row {
                    in_deleting_row = false;
                } else {
                    writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
                }
                current_row += 1;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"tbl" => {
                if in_target_tbl {
                    in_target_tbl = false;
                }
                current_tbl += 1;
                writer.write_event(Event::End(e.clone())).map_err(|e| WordCliError::xml_parse(e.to_string()))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !in_deleting_row {
                    writer.write_event(e).map_err(|err| WordCliError::xml_parse(err.to_string()))?;
                }
            }
            Err(e) => return Err(WordCliError::xml_parse(e.to_string())),
        }
        buf.clear();
    }

    if !deleted {
        return Err(WordCliError::invalid_parameter(format!(
            "要删除的行不存在: 行索引 {}",
            target_row_idx
        )));
    }

    let result = writer.into_inner().into_inner();
    let updated = String::from_utf8(result).map_err(|e| WordCliError::corrupted_document(e.to_string()))?;
    pkg.set_text("word/document.xml", updated);
    pkg.save_to_file(file_path)?;

    Ok(TableOpResponse {
        status: "success".to_string(),
        command: "delete-row".to_string(),
        table_index: target_tbl_idx,
        file: file_path.to_string(),
        message: format!("已删除表格 {} 的行 {}", target_tbl_idx, target_row_idx),
    })
}

fn build_row_xml(cells: &[String]) -> String {
    let mut row = String::from("<w:tr>");
    for cell in cells {
        row.push_str(&format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p></w:tc>"#,
            escape_xml(cell)
        ));
    }
    row.push_str("</w:tr>");
    row
}