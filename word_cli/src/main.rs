mod content;
mod error;
mod header_footer;
mod importer;
mod inspect;
mod lifecycle;
mod media;
mod package;
mod page;
mod paragraph;
mod styles;
mod table;

use clap::{Parser, Subcommand};
use serde::Serialize;
use std::process::ExitCode;

use crate::content::{read_paragraphs, replace_text, set_highlight};
use crate::error::WordCliError;
use crate::header_footer::{insert_page_break_before, set_header_or_footer};
use crate::importer::import_markdown;
use crate::inspect::inspect_document;
use crate::lifecycle::{create_document, save_as};
use crate::media::{extract_all_images, insert_image_at_para, replace_image_binary};
use crate::package::DocxPackage;
use crate::page::setup_page_layout;
use crate::paragraph::{
    append_paragraph, apply_style_to_paragraph, delete_paragraph, format_paragraph,
    insert_paragraph, merge_documents,
};
use crate::styles::list_styles;
use crate::table::{delete_table_row, insert_table_row, read_table_data, set_cell_value};

#[derive(Parser, Debug)]
#[command(
    name = "word_cli",
    version = "0.6.0",
    about = "高性能、CLI驱动的Word文档自动化处理工具",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 新建合规标准空白文档或克隆母版模板
    Create {
        #[arg(long)]
        file: String,
        #[arg(long)]
        template: Option<String>,
    },

    /// 复制另存为新文档
    SaveAs {
        #[arg(long)]
        file: String,
        #[arg(long)]
        output: String,
    },

    /// 全局宏观感知驾驶舱（大纲树、统计、表格、图片、高亮）
    Inspect {
        #[arg(long)]
        file: String,
        #[arg(long)]
        only: Option<String>,
    },

    /// 查询文档中已定义样式映射
    Styles {
        #[arg(long)]
        file: String,
    },

    /// 将 Markdown/TXT 格式直接转换为 Word 文档
    Import {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        template: Option<String>,
    },

    /// 读取指定段落范围或指定标题下的正文
    Read {
        #[arg(long)]
        file: String,
        #[arg(long)]
        range: Option<String>,
        #[arg(long)]
        heading: Option<String>,
    },

    /// 跨 Run 占位符/文本安全替换
    Replace {
        #[arg(long)]
        file: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        pairs: Option<String>,
    },

    /// 突出高亮控制（黄/绿/青或 none 清除）
    Highlight {
        #[arg(long)]
        file: String,
        #[arg(long)]
        text: String,
        #[arg(long, default_value = "yellow")]
        color: String,
    },

    /// 在指定段落前插入新段落
    InsertPara {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
        #[arg(long)]
        text: String,
        #[arg(long)]
        style: Option<String>,
    },

    /// 在文档末尾追加新段落
    AppendPara {
        #[arg(long)]
        file: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        style: Option<String>,
    },

    /// 删除指定索引段落
    DeletePara {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
    },

    /// 段落格式刷平与排版调整
    ParaFormat {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
        /// 首行缩进字符数，例如 2.0
        #[arg(long)]
        indent: Option<f64>,
        /// 行距倍数，例如 1.5
        #[arg(long)]
        spacing: Option<f64>,
        /// 字体名称，例如 "黑体"、"仿宋"
        #[arg(long)]
        font: Option<String>,
        /// 字号磅值 (pt)，例如小二是 18.0，三号是 16.0，四号是 14.0
        #[arg(long)]
        size: Option<f64>,
        /// 是否加粗
        #[arg(long)]
        bold: Option<bool>,
        /// 刷平野杂色高亮与字体格式
        #[arg(long, default_value_t = false)]
        strip_formatting: bool,
    },

    /// 套用母版样式
    ApplyStyle {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
        #[arg(long)]
        style: String,
    },

    /// 合并外部 docx 内容
    Merge {
        #[arg(long)]
        file: String,
        #[arg(long)]
        append: String,
    },

    /// 读取表格二维数据（支持 json, markdown, csv）
    ReadTable {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = 0)]
        index: usize,
        /// 输出格式: json, markdown (或 md), csv
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// 修改单元格文本
    SetCell {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = 0)]
        index: usize,
        #[arg(long)]
        row: usize,
        #[arg(long)]
        col: usize,
        #[arg(long)]
        val: String,
    },

    /// 在指定行前插入新行
    InsertRow {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = 0)]
        index: usize,
        #[arg(long)]
        row: usize,
        #[arg(long)]
        cells: String,
    },

    /// 删除指定行
    DeleteRow {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = 0)]
        index: usize,
        #[arg(long)]
        row: usize,
    },

    /// 提取文档中的所有图片
    ExtractImages {
        #[arg(long)]
        file: String,
        #[arg(long)]
        out_dir: String,
    },

    /// 替换文档中特定图片二进制
    ReplaceImage {
        #[arg(long)]
        file: String,
        #[arg(long)]
        rid: Option<String>,
        #[arg(long)]
        filename: Option<String>,
        #[arg(long)]
        image: String,
    },

    /// 在指定段落前插入图片
    InsertImage {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
        #[arg(long)]
        image: String,
        #[arg(long, default_value_t = 100.0)]
        width: f64,
        #[arg(long, default_value_t = 70.0)]
        height: f64,
    },

    /// 页面规格与方向控制
    Page {
        #[arg(long)]
        file: String,
        /// 页面方向: portrait 或 landscape
        #[arg(long, default_value = "portrait")]
        orient: String,
        /// 纸张规格: A4, A3
        #[arg(long, default_value = "A4")]
        size: String,
    },

    /// 在指定段落前插入分页符
    InsertPageBreak {
        #[arg(long)]
        file: String,
        #[arg(long)]
        index: usize,
    },

    /// 设置文档页眉
    SetHeader {
        #[arg(long)]
        file: String,
        #[arg(long)]
        text: Option<String>,
        /// 解除继承绑定
        #[arg(long, default_value_t = false)]
        unlink: bool,
    },

    /// 设置文档页脚与动态页码
    SetFooter {
        #[arg(long)]
        file: String,
        #[arg(long)]
        text: Option<String>,
        /// 开启动态页码域
        #[arg(long, default_value_t = false)]
        page_num: bool,
        /// 起始页码重新编号（例如 1）
        #[arg(long)]
        restart: Option<u32>,
        /// 解除继承绑定
        #[arg(long, default_value_t = false)]
        unlink: bool,
    },
}

fn print_json<T: Serialize>(value: &T) {
    if let Ok(json_str) = serde_json::to_string_pretty(value) {
        println!("{}", json_str);
    } else {
        eprintln!(
            r#"{{"status":"error","error_type":"SerializationError","message":"输出JSON序列化失败"}}"#
        );
    }
}

fn run() -> Result<(), WordCliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { file, template } => {
            let resp = create_document(&file, template.as_deref())?;
            print_json(&resp);
        }
        Commands::SaveAs { file, output } => {
            let resp = save_as(&file, &output)?;
            print_json(&resp);
        }
        Commands::Inspect { file, only } => {
            let pkg = DocxPackage::from_file(&file)?;
            let resp = inspect_document(&pkg, only.as_deref())?;
            print_json(&resp);
        }
        Commands::Styles { file } => {
            let pkg = DocxPackage::from_file(&file)?;
            let resp = list_styles(&pkg)?;
            print_json(&resp);
        }
        Commands::Import {
            input,
            output,
            template,
        } => {
            let resp = import_markdown(&input, &output, template.as_deref())?;
            print_json(&resp);
        }
        Commands::Read {
            file,
            range,
            heading,
        } => {
            let pkg = DocxPackage::from_file(&file)?;
            let resp = read_paragraphs(&pkg, range.as_deref(), heading.as_deref())?;
            print_json(&resp);
        }
        Commands::Replace {
            file,
            from,
            to,
            pairs,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = replace_text(
                &mut pkg,
                &file,
                from.as_deref(),
                to.as_deref(),
                pairs.as_deref(),
            )?;
            print_json(&resp);
        }
        Commands::Highlight { file, text, color } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = set_highlight(&mut pkg, &file, &text, &color)?;
            print_json(&resp);
        }
        Commands::InsertPara {
            file,
            index,
            text,
            style,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp =
                insert_paragraph(&mut pkg, &file, index, &text, style.as_deref())?;
            print_json(&resp);
        }
        Commands::AppendPara { file, text, style } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = append_paragraph(&mut pkg, &file, &text, style.as_deref())?;
            print_json(&resp);
        }
        Commands::DeletePara { file, index } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = delete_paragraph(&mut pkg, &file, index)?;
            print_json(&resp);
        }
        Commands::ParaFormat {
            file,
            index,
            indent,
            spacing,
            font,
            size,
            bold,
            strip_formatting,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = format_paragraph(
                &mut pkg,
                &file,
                index,
                indent,
                spacing,
                font.as_deref(),
                size,
                bold,
                strip_formatting,
            )?;
            print_json(&resp);
        }
        Commands::ApplyStyle { file, index, style } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = apply_style_to_paragraph(&mut pkg, &file, index, &style)?;
            print_json(&resp);
        }
        Commands::Merge { file, append } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = merge_documents(&mut pkg, &file, &append)?;
            print_json(&resp);
        }
        Commands::ReadTable { file, index, format } => {
            let pkg = DocxPackage::from_file(&file)?;
            let resp = read_table_data(&pkg, index, &format)?;
            print_json(&resp);
        }
        Commands::SetCell {
            file,
            index,
            row,
            col,
            val,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = set_cell_value(&mut pkg, &file, index, row, col, &val)?;
            print_json(&resp);
        }
        Commands::InsertRow {
            file,
            index,
            row,
            cells,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let cell_list: Vec<String> = cells.split(',').map(|s| s.trim().to_string()).collect();
            let resp = insert_table_row(&mut pkg, &file, index, row, &cell_list)?;
            print_json(&resp);
        }
        Commands::DeleteRow { file, index, row } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = delete_table_row(&mut pkg, &file, index, row)?;
            print_json(&resp);
        }
        Commands::ExtractImages { file, out_dir } => {
            let pkg = DocxPackage::from_file(&file)?;
            let resp = extract_all_images(&pkg, &out_dir)?;
            print_json(&resp);
        }
        Commands::ReplaceImage {
            file,
            rid,
            filename,
            image,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = replace_image_binary(
                &mut pkg,
                &file,
                rid.as_deref(),
                filename.as_deref(),
                &image,
            )?;
            print_json(&resp);
        }
        Commands::InsertImage {
            file,
            index,
            image,
            width,
            height,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = insert_image_at_para(&mut pkg, &file, index, &image, width, height)?;
            print_json(&resp);
        }
        Commands::Page { file, orient, size } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = setup_page_layout(&mut pkg, &file, &orient, &size)?;
            print_json(&resp);
        }
        Commands::InsertPageBreak { file, index } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = insert_page_break_before(&mut pkg, &file, index)?;
            print_json(&resp);
        }
        Commands::SetHeader { file, text, unlink } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = set_header_or_footer(&mut pkg, &file, true, text.as_deref(), false, None, unlink)?;
            print_json(&resp);
        }
        Commands::SetFooter {
            file,
            text,
            page_num,
            restart,
            unlink,
        } => {
            let mut pkg = DocxPackage::from_file(&file)?;
            let resp = set_header_or_footer(&mut pkg, &file, false, text.as_deref(), page_num, restart, unlink)?;
            print_json(&resp);
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("{}", err.to_json());
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}