use calamine::{open_workbook_auto, Data, Range, Reader};
use csv::WriterBuilder;
use regex::Regex;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use umya_spreadsheet::EnumTrait;

use crate::error::AppError;

#[derive(Serialize)]
pub struct SheetMeta {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Serialize)]
pub struct InspectData {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_sheets: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheets: Option<Vec<SheetMeta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sheet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

#[derive(Serialize)]
pub struct SearchHit {
    pub sheet: String,
    pub row: usize,
    pub col: usize,
    pub matched_value: String,
    pub row_data: Vec<String>,
}

#[derive(Serialize)]
pub struct ReadData {
    pub sheet: String,
    pub total_rows: usize,
    pub total_cols: usize,
    pub rows: Vec<Vec<String>>,
}

#[derive(Serialize)]
pub struct ConvertData {
    pub input_file: String,
    pub output_file: String,
    pub exported_sheet: String,
    pub total_rows: usize,
}

/// 将 calamine::Data 枚举直接格式化为可读 UTF-8 字符串
fn data_to_string(val: &Data) -> String {
    match val {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if (f.fract() == 0.0) && (f.abs() <= i64::MAX as f64) {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => format!("{:.4}", d.as_f64()),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{:?}", e),
    }
}

/// 1. 检查与元数据提取
pub fn inspect_file(file_path: &str, target_sheet: Option<&str>) -> Result<InspectData, AppError> {
    let clean_path = file_path.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    let path = Path::new(clean_path);
    if !path.exists() {
        return Err(AppError::FileNotFound(clean_path.to_string()));
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::Calamine(format!("无法打开文件，可能已被占用或格式损坏: {}", e)))?;

    if let Some(target) = target_sheet {
        let range = workbook
            .worksheet_range(target)
            .map_err(|_| AppError::SheetNotFound(target.to_string()))?;

        let (rows, cols) = range.get_size();
        let end_col = if cols > 0 { crate::writer::col_to_letter(cols as u32) } else { "A".to_string() };
        let end_row = if rows > 0 { rows } else { 1 };

        Ok(InspectData {
            file: file_path.to_string(),
            total_sheets: None,
            sheets: None,
            target_sheet: Some(target.to_string()),
            rows: Some(rows),
            cols: Some(cols),
            range: Some(format!("A1:{}{}", end_col, end_row)),
        })
    } else {
        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let (rows, cols) = range.get_size();
                sheets.push(SheetMeta {
                    name: name.clone(),
                    rows,
                    cols,
                });
            } else {
                sheets.push(SheetMeta {
                    name: name.clone(),
                    rows: 0,
                    cols: 0,
                });
            }
        }

        Ok(InspectData {
            file: file_path.to_string(),
            total_sheets: Some(sheets.len()),
            sheets: Some(sheets),
            target_sheet: None,
            rows: None,
            cols: None,
            range: None,
        })
    }
}

/// 2. 全文检索
pub fn search_file(
    file_path: &str,
    keyword: &str,
    target_sheet: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchHit>, AppError> {
    let clean_path = file_path.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    let path = Path::new(clean_path);
    if !path.exists() {
        return Err(AppError::FileNotFound(clean_path.to_string()));
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::Calamine(format!("打开文件失败: {}", e)))?;

    let sheets_to_search: Vec<String> = match target_sheet {
        Some(s) => vec![s.to_string()],
        None => workbook.sheet_names().to_vec(),
    };

    let mut hits = Vec::new();

    for sheet_name in sheets_to_search {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for (row_idx, row) in range.rows().enumerate() {
                let row_strings: Vec<String> = row.iter().map(data_to_string).collect();
                for (col_idx, cell_str) in row_strings.iter().enumerate() {
                    if cell_str.contains(keyword) {
                        hits.push(SearchHit {
                            sheet: sheet_name.clone(),
                            row: row_idx + 1,
                            col: col_idx + 1,
                            matched_value: cell_str.clone(),
                            row_data: row_strings.clone(),
                        });

                        if limit > 0 && hits.len() >= limit {
                            return Ok(hits);
                        }
                    }
                }
            }
        } else if target_sheet.is_some() {
            return Err(AppError::SheetNotFound(target_sheet.unwrap().to_string()));
        }
    }

    Ok(hits)
}

/// 解析类似 "A1:D10" 或 "B2" 的坐标区域，返回 (start_row, start_col, end_row, end_col) 从 0 计数
fn parse_range_str(range_str: &str) -> Option<(usize, usize, usize, usize)> {
    let re = Regex::new(r"^([A-Za-z]+)(\d+)(?::([A-Za-z]+)(\d+))?$").ok()?;
    let caps = re.captures(range_str)?;

    let col_to_idx = |col: &str| -> usize {
        let mut idx = 0;
        for c in col.to_ascii_uppercase().chars() {
            if c.is_ascii_alphabetic() {
                idx = idx * 26 + (c as usize - 'A' as usize + 1);
            }
        }
        idx.saturating_sub(1)
    };

    let c1 = col_to_idx(&caps[1]);
    let r1 = caps[2].parse::<usize>().ok()?.saturating_sub(1);

    if let (Some(col2), Some(row2)) = (caps.get(3), caps.get(4)) {
        let c2 = col_to_idx(col2.as_str());
        let r2 = row2.as_str().parse::<usize>().ok()?.saturating_sub(1);
        Some((r1.min(r2), c1.min(c2), r1.max(r2), c1.max(c2)))
    } else {
        Some((r1, c1, r1, c1))
    }
}

/// 3. 范围切片读取与预览
pub fn read_range(
    file_path: &str,
    sheet_name: Option<&str>,
    range_str: Option<&str>,
    head: Option<usize>,
) -> Result<ReadData, AppError> {
    let clean_path = file_path.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    let path = Path::new(clean_path);
    if !path.exists() {
        return Err(AppError::FileNotFound(clean_path.to_string()));
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::Calamine(format!("打开文件失败: {}", e)))?;

    let sheet = match sheet_name {
        Some(name) => name.to_string(),
        None => workbook
            .sheet_names()
            .first()
            .ok_or_else(|| AppError::General("工作簿内不存在任何工作表".to_string()))?
            .clone(),
    };

    let range: Range<Data> = workbook
        .worksheet_range(&sheet)
        .map_err(|e| AppError::Calamine(format!("读取工作表数据失败: {}", e)))?;

    let (max_r, max_c) = range.get_size();
    if max_r == 0 || max_c == 0 {
        return Ok(ReadData {
            sheet,
            total_rows: 0,
            total_cols: 0,
            rows: Vec::new(),
        });
    }

    let mut all_rows: Vec<Vec<String>> = range
        .rows()
        .map(|r| r.iter().map(data_to_string).collect())
        .collect();

    if let Some(coord_str) = range_str {
        if let Some((r1, c1, r2, c2)) = parse_range_str(coord_str) {
            let mut sliced_rows = Vec::new();
            for r in r1..=r2.min(all_rows.len().saturating_sub(1)) {
                let row_len = all_rows[r].len();
                if c1 < row_len {
                    let end_c = (c2 + 1).min(row_len);
                    sliced_rows.push(all_rows[r][c1..end_c].to_vec());
                } else {
                    sliced_rows.push(Vec::new());
                }
            }
            all_rows = sliced_rows;
        } else {
            return Err(AppError::InvalidCoordinate(format!(
                "无效的范围坐标表达式: {}",
                coord_str
            )));
        }
    }

    if let Some(n) = head {
        if n < all_rows.len() {
            all_rows.truncate(n);
        }
    }

    let total_rows = all_rows.len();
    let total_cols = all_rows.first().map_or(0, |r| r.len());

    Ok(ReadData {
        sheet,
        total_rows,
        total_cols,
        rows: all_rows,
    })
}

/// 4. 导出为 UTF-8 BOM CSV
pub fn export_to_csv(
    file_path: &str,
    sheet_name: Option<&str>,
    output_path: &str,
) -> Result<ConvertData, AppError> {
    let read_result = read_range(file_path, sheet_name, None, None)?;

    let mut out_file = File::create(output_path)?;
    out_file.write_all(&[0xEF, 0xBB, 0xBF])?;

    let mut wtr = WriterBuilder::new().from_writer(out_file);
    for row in &read_result.rows {
        wtr.write_record(row)
            .map_err(|e| AppError::General(format!("CSV 写入错误: {}", e)))?;
    }
    wtr.flush()
        .map_err(|e| AppError::General(format!("CSV 刷新缓冲区失败: {}", e)))?;

    Ok(ConvertData {
        input_file: file_path.to_string(),
        output_file: output_path.to_string(),
        exported_sheet: read_result.sheet,
        total_rows: read_result.total_rows,
    })
}

// 5. 单元格样式提取
use serde_json::{json, Value};

/// 提取指定单元格或区域的样式属性
pub fn get_style_info(
    file_path: &str,
    sheet_name: Option<&str>,
    coord_str: &str,
) -> Result<Value, AppError> {
    let clean_path = file_path.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    let path = Path::new(clean_path);
    if !path.exists() {
        return Err(AppError::FileNotFound(clean_path.to_string()));
    }

    let book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| AppError::General(format!("读取文件样式失败: {}", e)))?;

    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet(&0)
            .ok_or_else(|| AppError::General("工作簿内不存在任何工作表".to_string()))?,
    };

    if let Some((r1, c1, r2, c2)) = parse_range_str(coord_str) {
        if r1 == r2 && c1 == c2 {
            let col = (c1 + 1) as u32;
            let row = (r1 + 1) as u32;
            let style_json = extract_cell_style_json(sheet, col, row);

            Ok(json!({
                "coord": coord_str,
                "sheet": sheet.get_name(),
                "style": style_json
            }))
        } else {
            let mut cells_map = serde_json::Map::new();

            for r in r1..=r2 {
                for c in c1..=c2 {
                    let col = (c + 1) as u32;
                    let row = (r + 1) as u32;
                    let cell_coord = umya_spreadsheet::helper::coordinate::coordinate_from_index(&col, &row);
                    let style_json = extract_cell_style_json(sheet, col, row);
                    cells_map.insert(cell_coord, style_json);
                }
            }

            Ok(json!({
                "range": coord_str,
                "sheet": sheet.get_name(),
                "cells": cells_map
            }))
        }
    } else {
        Err(AppError::InvalidCoordinate(format!(
            "无效的坐标或范围格式: {}",
            coord_str
        )))
    }
}

/// 提取单个单元格样式的核心私有函数
fn extract_cell_style_json(sheet: &umya_spreadsheet::Worksheet, col: u32, row: u32) -> Value {
    let cell = match sheet.get_cell((col, row)) {
        Some(c) => c,
        None => return json!({ "status": "empty_or_default" }),
    };

    let style = cell.get_style();
    let font = style.get_font();
    let align = style.get_alignment();
    let fill = style.get_fill();
    let borders = style.get_borders();

    json!({
        "font": {
            "name": font.map(|f| f.get_name().to_string()).unwrap_or_default(),
            "size": font.map(|f| *f.get_size()).unwrap_or(11.0),
            "bold": font.map(|f| *f.get_bold()).unwrap_or(false),
            "italic": font.map(|f| *f.get_italic()).unwrap_or(false),
            "color": font.map(|f| f.get_color().get_argb().to_string()).unwrap_or_default()
        },
		"alignment": {
            "horizontal": align.map(|a| a.get_horizontal().get_value_string().to_string()).unwrap_or_else(|| "general".to_string()),
            "vertical": align.map(|a| a.get_vertical().get_value_string().to_string()).unwrap_or_else(|| "bottom".to_string())
        },
        "fill": {
            "color": fill.and_then(|f| f.get_pattern_fill().and_then(|p| p.get_foreground_color().map(|c| c.get_argb().to_string()))).unwrap_or_default()
        },
        "border": borders.is_some()
    })
}