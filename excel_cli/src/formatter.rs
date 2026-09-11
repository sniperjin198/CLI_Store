use std::fs;
use std::path::Path;
use umya_spreadsheet::structs::PatternFill;
use umya_spreadsheet::{
    reader, writer, Border, Color, Fill, HorizontalAlignmentValues, PatternValues, Spreadsheet,
    VerticalAlignmentValues,
};

use crate::error::AppError;

/// 原子保存
fn safe_save(book: &Spreadsheet, file_path: &str) -> Result<(), AppError> {
    let path = Path::new(file_path);
    let temp_path = format!("{}.tmp", file_path);
    let temp_p = Path::new(&temp_path);

    writer::xlsx::write(book, temp_p)
        .map_err(|e| AppError::General(format!("临时文件写入失败：{}", e)))?;

    if path.exists() {
        if let Err(e) = fs::remove_file(path) {
            let _ = fs::remove_file(temp_p);
            return Err(AppError::General(format!(
                "无法覆盖目标文件，可能被 Excel 打开占用：{}",
                e
            )));
        }
    }

    fs::rename(temp_p, path).map_err(|e| {
        AppError::General(format!("文件重命名替换失败：{}", e))
    })?;

    Ok(())
}

fn open_workbook(file_path: &str) -> Result<Spreadsheet, AppError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::FileNotFound(file_path.to_string()));
    }
    reader::xlsx::read(path)
        .map_err(|e| AppError::General(format!("读取工作簿失败：{}", e)))
}

/// 解析 6 位 Hex 颜色 (例如 "4F81BD" 或 "#4F81BD")
fn parse_hex_color(hex: &str) -> String {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        format!("FF{}", clean.to_ascii_uppercase())
    } else {
        clean.to_ascii_uppercase()
    }
}

/// 1. 设置单元格或区域样式 (包含字体名称、粗细、大小、颜色、背景色、对齐与边框)
pub fn style_range(
    file_path: &str,
    sheet_name: Option<&str>,
    range_coord: &str,
    bold: Option<bool>,
    font_size: Option<f64>,
    font_name: Option<&str>,
    font_color: Option<&str>,
    bg_color: Option<&str>,
    align_h: Option<&str>,
    align_v: Option<&str>,
    border: Option<bool>,
) -> Result<String, AppError> {
    let mut book = open_workbook(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    let style = sheet.get_style_mut(range_coord);

    if let Some(b) = bold {
        style.get_font_mut().set_bold(b);
    }
    if let Some(size) = font_size {
        style.get_font_mut().set_size(size);
    }
    if let Some(name) = font_name {
        style.get_font_mut().set_name(name);
    }
    if let Some(c) = font_color {
        let argb = parse_hex_color(c);
        let mut color = Color::default();
        color.set_argb(argb);
        style.get_font_mut().set_color(color);
    }
    if let Some(bg) = bg_color {
        let argb = parse_hex_color(bg);
        let mut pattern_fill = PatternFill::default();
        pattern_fill.set_pattern_type(PatternValues::Solid);
        let mut color = Color::default();
        color.set_argb(argb);
        pattern_fill.set_foreground_color(color);

        let mut fill = Fill::default();
        fill.set_pattern_fill(pattern_fill);
        style.set_fill(fill);
    }
    if let Some(h) = align_h {
        match h.to_lowercase().as_str() {
            "left" => {
                style
                    .get_alignment_mut()
                    .set_horizontal(HorizontalAlignmentValues::Left);
            }
            "center" | "centre" => {
                style
                    .get_alignment_mut()
                    .set_horizontal(HorizontalAlignmentValues::Center);
            }
            "right" => {
                style
                    .get_alignment_mut()
                    .set_horizontal(HorizontalAlignmentValues::Right);
            }
            _ => {}
        }
    }
    if let Some(v) = align_v {
        match v.to_lowercase().as_str() {
            "top" => {
                style
                    .get_alignment_mut()
                    .set_vertical(VerticalAlignmentValues::Top);
            }
            "center" | "centre" => {
                style
                    .get_alignment_mut()
                    .set_vertical(VerticalAlignmentValues::Center);
            }
            "bottom" => {
                style
                    .get_alignment_mut()
                    .set_vertical(VerticalAlignmentValues::Bottom);
            }
            _ => {}
        }
    }
    if let Some(has_border) = border {
        if has_border {
            let mut b = Border::default();
            b.set_border_style(Border::BORDER_THIN);
            style.get_borders_mut().set_top(b.clone());
            style.get_borders_mut().set_bottom(b.clone());
            style.get_borders_mut().set_left(b.clone());
            style.get_borders_mut().set_right(b);
        }
    }

    safe_save(&book, file_path)?;
    Ok(format!("区域 {} 样式应用成功", range_coord))
}

/// 2. 设置行高与列宽
pub fn set_dimensions(
    file_path: &str,
    sheet_name: Option<&str>,
    col: Option<&str>,
    width: Option<f64>,
    row: Option<u32>,
    height: Option<f64>,
) -> Result<String, AppError> {
    let mut book = open_workbook(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    if let (Some(c), Some(w)) = (col, width) {
        sheet.get_column_dimension_mut(c).set_width(w);
    }
    if let (Some(r), Some(h)) = (row, height) {
        sheet.get_row_dimension_mut(&r).set_height(h);
    }

    safe_save(&book, file_path)?;
    Ok("尺寸调整成功".to_string())
}

/// 3. 合并单元格
pub fn merge_cells(
    file_path: &str,
    sheet_name: Option<&str>,
    range_coord: &str,
) -> Result<String, AppError> {
    let mut book = open_workbook(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    sheet.add_merge_cells(range_coord);
    safe_save(&book, file_path)?;
    Ok(format!("单元格合并成功：{}", range_coord))
}

/// 4. 将 Excel 字母列转为数字 (A -> 1, Z -> 26, AA -> 27)
fn letter_to_col(letter: &str) -> u32 {
    letter
        .to_uppercase()
        .chars()
        .fold(0, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1))
}

/// 5. 插入与删除行/列
pub fn modify_grid(
    file_path: &str,
    sheet_name: Option<&str>,
    action: &str,
    target: &str,
    count: u32,
) -> Result<String, AppError> {
    let mut book = open_workbook(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    match action {
        "insert-row" => {
            let row: u32 = target
                .parse()
                .map_err(|_| AppError::General("行号必须是正整数".to_string()))?;
            sheet.insert_new_row(&row, &count);
            safe_save(&book, file_path)?;
            Ok(format!("已在第 {} 行前插入 {} 行", row, count))
        }
        "delete-row" => {
            let row: u32 = target
                .parse()
                .map_err(|_| AppError::General("行号必须是正整数".to_string()))?;
            sheet.remove_row(&row, &count);
            safe_save(&book, file_path)?;
            Ok(format!("已从第 {} 行开始删除 {} 行", row, count))
        }
        "insert-col" => {
            let col_str = target.to_uppercase();
            sheet.insert_new_column(&col_str, &count);
            safe_save(&book, file_path)?;
            Ok(format!("已在列 {} 前插入 {} 列", col_str, count))
        }
        "delete-col" => {
            let col_num = if let Ok(n) = target.parse::<u32>() {
                n
            } else {
                letter_to_col(target)
            };
            sheet.remove_column_by_index(&col_num, &count);
            safe_save(&book, file_path)?;
            Ok(format!("已从列 {} 开始删除 {} 列", target, count))
        }
        _ => Err(AppError::General(format!(
            "不支持的操作: {}。可选: insert-row, delete-row, insert-col, delete-col",
            action
        ))),
    }
}

/// 估算字符串在 Excel 中的视觉字符宽度（中文字符计为 2.1，西文字符计为 1.1）
fn estimate_text_width(text: &str) -> f64 {
    let mut width = 0.0;
    for ch in text.chars() {
        if ch == '\n' || ch == '\r' {
            continue;
        }
        if (ch as u32) > 127 {
            width += 2.1;
        } else {
            width += 1.1;
        }
    }
    width
}

/// 6. 自动适应行高与列宽
pub fn autofit_dimensions(
    file_path: &str,
    sheet_name: Option<&str>,
    auto_col: bool,
    auto_row: bool,
    min_col_width: Option<f64>,
    max_col_width: Option<f64>,
) -> Result<String, AppError> {
    let mut book = open_workbook(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    let (max_col_idx, max_row_idx) = sheet.get_highest_column_and_row();
    let min_w = min_col_width.unwrap_or(10.0);
    let max_w = max_col_width.unwrap_or(60.0);

    // 自适应列宽
    if auto_col {
        for col in 1..=max_col_idx {
            let col_letter = umya_spreadsheet::helper::coordinate::string_from_column_index(&col);
            let mut max_len: f64 = 0.0;

            for row in 1..=max_row_idx {
                if let Some(cell) = sheet.get_cell((col, row)) {
                    let val = cell.get_value();
                    for line in val.split('\n') {
                        let len = estimate_text_width(line);
                        if len > max_len {
                            max_len = len;
                        }
                    }
                }
            }

            if max_len > 0.0 {
                let fitted_width = (max_len + 3.0).clamp(min_w, max_w);
                sheet.get_column_dimension_mut(&col_letter).set_width(fitted_width);
            } else {
                sheet.get_column_dimension_mut(&col_letter).set_width(min_w);
            }
        }
    }

    // 自适应行高
    if auto_row {
        for row in 1..=max_row_idx {
            let mut max_lines = 1;
            for col in 1..=max_col_idx {
                if let Some(cell) = sheet.get_cell((col, row)) {
                    let val = cell.get_value();
                    let line_count = val.lines().count();
                    if line_count > max_lines {
                        max_lines = line_count;
                    }
                }
            }
            let fitted_height = 18.0 + (max_lines.saturating_sub(1) as f64 * 14.0);
            sheet.get_row_dimension_mut(&row).set_height(fitted_height);
        }
    }

    safe_save(&book, file_path)?;
    Ok("行高/列宽自适应调整完成".to_string())
}