use std::fs;
use std::path::Path;
use umya_spreadsheet::{new_file, reader, writer, Spreadsheet};

use crate::error::AppError;

/// 原子写入：先写临时文件再重命名，防御 Windows 文件锁与中途崩溃
fn safe_save(book: &Spreadsheet, file_path: &str) -> Result<(), AppError> {
    let path = Path::new(file_path);
    let temp_path = format!("{}.tmp", file_path);
    let temp_p = Path::new(&temp_path);

    writer::xlsx::write(book, temp_p)
        .map_err(|e| AppError::General(format!("临时文件写入失败: {}", e)))?;

    if path.exists() {
        if let Err(e) = fs::remove_file(path) {
            let _ = fs::remove_file(temp_p);
            return Err(AppError::General(format!(
                "无法覆盖目标文件，可能被 Excel 打开占用: {}",
                e
            )));
        }
    }

    fs::rename(temp_p, path).map_err(|e| {
        AppError::General(format!("文件重命名替换失败: {}", e))
    })?;

    Ok(())
}

/// 打开现有文件或新建文件
fn open_or_create(file_path: &str) -> Result<Spreadsheet, AppError> {
    let path = Path::new(file_path);
    if path.exists() {
        reader::xlsx::read(path)
            .map_err(|e| AppError::General(format!("读取工作簿失败，文件可能已损坏: {}", e)))
    } else {
        let mut book = new_file();
        let _ = book.set_sheet_name(0, "Sheet1");
        Ok(book)
    }
}

/// 将数字列号转为 Excel 字母列名 (1 -> "A", 28 -> "AB")
pub fn col_to_letter(mut col: u32) -> String {
    let mut s = String::new();
    while col > 0 {
        col -= 1;
        let rem = (col % 26) as u8;
        s.insert(0, (b'A' + rem) as char);
        col /= 26;
    }
    s
}

/// 1. 新建空白工作簿
pub fn create_workbook(file_path: &str, sheet_name: Option<&str>) -> Result<String, AppError> {
    let mut book = new_file();
    let target_sheet = sheet_name.unwrap_or("Sheet1");
    let _ = book.set_sheet_name(0, target_sheet);
    safe_save(&book, file_path)?;
    Ok(format!("工作簿创建成功: {} (Sheet: {})", file_path, target_sheet))
}

/// 2. 单元格设值（支持公式、数值、布尔与普通文本自动推断）
pub fn write_cell(
    file_path: &str,
    sheet_name: Option<&str>,
    coord: &str,
    value: &str,
) -> Result<String, AppError> {
    let mut book = open_or_create(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    let cell = sheet.get_cell_mut(coord);
    if value.starts_with('=') {
        cell.set_formula(value);
    } else if let Ok(num) = value.parse::<i64>() {
        cell.set_value_number(num as f64);
    } else if let Ok(num) = value.parse::<f64>() {
        cell.set_value_number(num);
    } else if let Ok(b) = value.parse::<bool>() {
        cell.set_value_bool(b);
    } else {
        cell.set_value_string(value);
    }

    safe_save(&book, file_path)?;
    Ok(format!("单元格 {} 写入成功: {}", coord, value))
}

/// 3. 追加一行数据
pub fn append_row(
    file_path: &str,
    sheet_name: Option<&str>,
    values: &[String],
) -> Result<usize, AppError> {
    let mut book = open_or_create(file_path)?;
    let sheet = match sheet_name {
        Some(name) => book
            .get_sheet_by_name_mut(name)
            .ok_or_else(|| AppError::SheetNotFound(name.to_string()))?,
        None => book
            .get_sheet_mut(&0)
            .ok_or_else(|| AppError::General("未找到有效工作表".to_string()))?,
    };

    let highest_row = sheet.get_highest_row();
    let target_row = if highest_row == 0 { 1 } else { highest_row + 1 };

    for (idx, val) in values.iter().enumerate() {
        let col_str = col_to_letter((idx + 1) as u32);
        let cell_coord = format!("{}{}", col_str, target_row);
        let cell = sheet.get_cell_mut(cell_coord.as_str());

        if val.starts_with('=') {
            cell.set_formula(val);
        } else if let Ok(num) = val.parse::<i64>() {
            cell.set_value_number(num as f64);
        } else if let Ok(num) = val.parse::<f64>() {
            cell.set_value_number(num);
        } else if let Ok(b) = val.parse::<bool>() {
            cell.set_value_bool(b);
        } else {
            cell.set_value_string(val);
        }
    }

    safe_save(&book, file_path)?;
    Ok(target_row as usize)
}

/// 4. 工作表管理
pub fn manage_sheet(
    file_path: &str,
    action: &str,
    sheet_name: &str,
    new_name: Option<&str>,
) -> Result<String, AppError> {
    let mut book = open_or_create(file_path)?;

    match action {
        "add" => {
            let _ = book.new_sheet(sheet_name);
            safe_save(&book, file_path)?;
            Ok(format!("工作表 {} 创建成功", sheet_name))
        }
        "remove" => {
            if book.get_sheet_collection().len() <= 1 {
                return Err(AppError::General("无法删除仅存的最后一个工作表".to_string()));
            }
            book.remove_sheet_by_name(sheet_name)
                .map_err(|_| AppError::SheetNotFound(sheet_name.to_string()))?;
            safe_save(&book, file_path)?;
            Ok(format!("工作表 {} 删除成功", sheet_name))
        }
        "rename" => {
            let n_name = new_name.ok_or_else(|| {
                AppError::General("重命名工作表必须提供 new_name".to_string())
            })?;
            let sheet_idx = book
                .get_sheet_collection()
                .iter()
                .position(|s| s.get_name() == sheet_name)
                .ok_or_else(|| AppError::SheetNotFound(sheet_name.to_string()))?;
            let _ = book.set_sheet_name(sheet_idx, n_name);
            safe_save(&book, file_path)?;
            Ok(format!("工作表已重命名: {} -> {}", sheet_name, n_name))
        }
        _ => Err(AppError::General(format!("不支持的工作表操作: {}", action))),
    }
}