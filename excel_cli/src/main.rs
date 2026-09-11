use clap::Parser;

mod cli;
mod error;
mod formatter;
mod reader;
mod writer;

use cli::{Cli, Commands};
use error::ApiResponse;

fn main() {
    let args = Cli::parse();

    match args.command {
        // 阶段二：只读与检索
        Commands::Inspect(arg) => {
            match reader::inspect_file(&arg.file, arg.sheet.as_deref()) {
                Ok(data) => ApiResponse::success("inspect", serde_json::to_value(data).unwrap(), "检查成功").print_and_exit(),
                Err(e) => ApiResponse::fail("inspect", e.to_string()).print_and_exit(),
            }
        }

        Commands::Search(arg) => {
            let limit = arg.limit.unwrap_or(0);
            match reader::search_file(&arg.file, &arg.keyword, arg.sheet.as_deref(), limit) {
                Ok(data) => {
                    ApiResponse::success("search", data, "Search completed").print_and_exit()
                }
                Err(e) => ApiResponse::fail("search", e.to_string()).print_and_exit(),
            }
        }

        Commands::Read(arg) => {
            match reader::read_range(
                &arg.file,
                arg.sheet.as_deref(),
                arg.range.as_deref(),
                arg.head,
            ) {
                Ok(data) => {
                    ApiResponse::success("read", data, "Read success").print_and_exit()
                }
                Err(e) => ApiResponse::fail("read", e.to_string()).print_and_exit(),
            }
        }

        Commands::Convert(arg) => {
            if arg.output.to_lowercase().ends_with(".csv") {
                match reader::export_to_csv(&arg.input, arg.sheet.as_deref(), &arg.output) {
                    Ok(data) => {
                        ApiResponse::success("convert", data, "Convert success").print_and_exit()
                    }
                    Err(e) => ApiResponse::fail("convert", e.to_string()).print_and_exit(),
                }
            } else {
                ApiResponse::fail("convert", "Only .csv format supported").print_and_exit();
            }
        }

        // 阶段三：写入与生命周期
        Commands::Create(arg) => {
            match writer::create_workbook(&arg.file, arg.sheet.as_deref()) {
                Ok(msg) => {
                    ApiResponse::success("create", serde_json::json!({ "file": arg.file }), msg)
                        .print_and_exit()
                }
                Err(e) => ApiResponse::fail("create", e.to_string()).print_and_exit(),
            }
        }

        Commands::Cell(arg) => {
            match writer::write_cell(&arg.file, arg.sheet.as_deref(), &arg.coord, &arg.value) {
                Ok(msg) => ApiResponse::success(
                    "cell",
                    serde_json::json!({ "file": arg.file, "coord": arg.coord, "value": arg.value }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("cell", e.to_string()).print_and_exit(),
            }
        }

        Commands::Append(arg) => {
            match writer::append_row(&arg.file, arg.sheet.as_deref(), &arg.values) {
                Ok(target_row) => ApiResponse::success(
                    "append",
                    serde_json::json!({ "file": arg.file, "appended_row": target_row }),
                    format!("数据已追加至第 {} 行", target_row),
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("append", e.to_string()).print_and_exit(),
            }
        }

        Commands::Sheet(arg) => {
            match writer::manage_sheet(
                &arg.file,
                &arg.action,
                &arg.sheet,
                arg.new_name.as_deref(),
            ) {
                Ok(msg) => ApiResponse::success(
                    "sheet",
                    serde_json::json!({ "file": arg.file, "sheet": arg.sheet, "action": arg.action }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("sheet", e.to_string()).print_and_exit(),
            }
        }

        // 阶段四：排版与样式
        Commands::Style(arg) => {
            match formatter::style_range(
                &arg.file,
                arg.sheet.as_deref(),
                &arg.coord,
                arg.bold,
                arg.font_size,
				arg.font_name.as_deref(),
                arg.font_color.as_deref(),
                arg.bg_color.as_deref(),
                arg.align_h.as_deref(),
                arg.align_v.as_deref(),
                arg.border,
            ) {
                Ok(msg) => ApiResponse::success(
                    "style",
                    serde_json::json!({ "file": arg.file, "coord": arg.coord }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("style", e.to_string()).print_and_exit(),
            }
        }

		// 提取单元格或区域样式
		Commands::StyleGet(arg) => {
            match reader::get_style_info(&arg.file, arg.sheet.as_deref(), &arg.coord) {
                Ok(data) => {
                    ApiResponse::success("style-get", data, "Style retrieved successfully")
                        .print_and_exit()
                }
                Err(e) => ApiResponse::fail("style-get", e.to_string()).print_and_exit(),
            }
        }

        Commands::Dimension(arg) => {
            match formatter::set_dimensions(
                &arg.file,
                arg.sheet.as_deref(),
                arg.col.as_deref(),
                arg.width,
                arg.row,
                arg.height,
            ) {
                Ok(msg) => ApiResponse::success(
                    "dimension",
                    serde_json::json!({ "file": arg.file }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("dimension", e.to_string()).print_and_exit(),
            }
        }

        Commands::Merge(arg) => {
            match formatter::merge_cells(&arg.file, arg.sheet.as_deref(), &arg.range) {
                Ok(msg) => ApiResponse::success(
                    "merge",
                    serde_json::json!({ "file": arg.file, "range": arg.range }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("merge", e.to_string()).print_and_exit(),
            }
        }
		
		Commands::Grid(arg) => {
            match formatter::modify_grid(
                &arg.file,
                arg.sheet.as_deref(),
                &arg.action,
                &arg.target,
                arg.count,
            ) {
                Ok(msg) => ApiResponse::success(
                    "grid",
                    serde_json::json!({ "file": arg.file, "action": arg.action, "target": arg.target, "count": arg.count }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("grid", e.to_string()).print_and_exit(),
            }
        }
		
		Commands::Autofit(arg) => {
            match formatter::autofit_dimensions(
                &arg.file,
                arg.sheet.as_deref(),
                arg.col,
                arg.row,
                arg.min_width,
                arg.max_width,
            ) {
                Ok(msg) => ApiResponse::success(
                    "autofit",
                    serde_json::json!({ "file": arg.file, "auto_col": arg.col, "auto_row": arg.row }),
                    msg,
                )
                .print_and_exit(),
                Err(e) => ApiResponse::fail("autofit", e.to_string()).print_and_exit(),
            }
        }
    }
}
