mod chunker;
mod cmap;
mod error;
mod extractor;
mod inspect;
mod layout;
mod media;
mod transform;
mod watermark;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

use crate::error::PdfCliError;

#[derive(Parser)]
#[command(
    name = "pdf_cli",
    about = "PDF CLI - 高性能、轻量化单二进制 PDF 几何自省、文本提取与装配手术刀",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 宏观感知驾驶舱：查看元数据、总页数、纸张尺寸与书签树
    Inspect {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 过滤只输出某项数据 (metadata | layout | outline)
        #[arg(long)]
        only: Option<String>,
    },

    /// 文本提取：支持按页码/范围/书签标题、坐标输出、降噪过滤及 Markdown 模式
    ExtractText {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 提取指定单页 (1-based)
        #[arg(short, long)]
        page: Option<usize>,

        /// 提取页码范围 (如 "1:5", "all")
        #[arg(short, long, visible_alias = "pages")]
        range: Option<String>,

        /// 目录定向精读：指定大纲书签标题自动定位起止页
        #[arg(long)]
        heading: Option<String>,

        /// 自动过滤页眉、页脚与页码噪音
        #[arg(long, default_value_t = false)]
        clean_noise: bool,

        /// 输出包含每个文本块 [x, y, width, height] 物理坐标
        #[arg(long, default_value_t = false)]
        with_bbox: bool,

        /// 输出格式 (text | md)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// 几何版式自省与财报审计：推算无框表格列宽投影与重叠文字异常
    InspectLayout {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 分析的目标页码 (1-based)
        #[arg(short, long)]
        page: usize,
    },

    /// RAG 知识库语义切片：支持大纲层级与滑动窗口分块
    Chunk {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 切片策略 (heading | sliding)
        #[arg(long, default_value = "heading")]
        strategy: String,

        /// 单块目标字数
        #[arg(long, default_value_t = 500, visible_alias = "chunk-size")]
        max_chars: usize,

        /// 滑动窗口重叠字数
        #[arg(long, default_value_t = 80)]
        overlap: usize,

        /// 自动清洗页眉页脚噪音
        #[arg(long, default_value_t = true)]
        clean_noise: bool,
    },

    /// 无损抽取内嵌图像：导出 PNG/JPEG 图片至目标文件夹
    ExtractImages {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 指定单页导出 (留空提取全文)
        #[arg(short, long)]
        page: Option<usize>,

        /// 图片输出目录
        #[arg(short, long, default_value = "./extracted_images")]
        out_dir: String,
    },

    /// 页面光栅化渲染：将指定页转换为 300DPI PNG 图像 (支撑 OCR / 视觉模型)
    RenderPage {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 目标渲染页码 (1-based)
        #[arg(short, long)]
        page: usize,

        /// 渲染分辨率 (DPI，默认 300)
        #[arg(long, default_value_t = 300)]
        dpi: u32,

        /// 输出图像路径
        #[arg(short, long, visible_alias = "output")]
        out: String,
    },

    /// 物理合并：将多个 PDF 拼接为单份交付件
    Merge {
        /// 输入的 PDF 文件列表 (逗号分隔或传入多个)
        #[arg(short, long, value_delimiter = ',', visible_alias = "inputs")]
        files: Vec<String>,

        /// 输出文件路径
        #[arg(short, long)]
        output: String,
    },

    /// 物理抽取：从原文档抽取指定页并重组为独立 PDF
    ExtractPages {
        /// 原 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 抽取页码范围 (如 "1:3", "5", "all")
        #[arg(short, long, visible_alias = "range")]
        pages: String,

        /// 导出的新 PDF 路径
        #[arg(short, long)]
        output: String,
    },

    /// 页面旋转：旋转图纸与横向版面 (90, 180, 270)
    Rotate {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 旋转指定页码区间 (如 "2,4", "1:3", "all")
        #[arg(short, long, visible_alias = "range")]
        pages: Option<String>,

        /// 顺时针旋转角度 (90, 180, 270)
        #[arg(short, long, visible_alias = "degrees")]
        angle: i32,

        /// 另存为目标文件 (留空直接覆盖原文件)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 注入半透明水印印章
    Watermark {
        /// 目标 PDF 文件路径
        #[arg(short, long)]
        file: String,

        /// 水印文本内容
        #[arg(short, long)]
        text: String,

        /// 透明度 (0.05 ~ 1.0)
        #[arg(long, default_value_t = 0.2)]
        opacity: f64,

        /// 字体字号
        #[arg(long, default_value_t = 42.0)]
        font_size: f64,

        /// 水印旋转倾角
        #[arg(long, default_value_t = 45.0)]
        rotation: f64,

        /// 另存为目标文件 (留空直接覆盖原文件)
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Inspect { file, only } => {
            let target = normalize_path(&file);
            inspect::inspect_pdf(&target, only.as_deref())
                .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractText {
            file,
            page,
            range,
            heading,
            clean_noise,
            with_bbox,
            format,
        } => {
            let target = normalize_path(&file);
            extractor::extract_text(
                &target,
                page,
                range.as_deref(),
                heading.as_deref(),
                clean_noise,
                with_bbox,
                &format,
            )
            .and_then(|res| to_json_output(&res))
        }
        Commands::InspectLayout { file, page } => {
            let target = normalize_path(&file);
            layout::inspect_page_layout(&target, page)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Chunk {
            file,
            strategy,
            max_chars,
            overlap,
            clean_noise,
        } => {
            let target = normalize_path(&file);
            chunker::chunk_document(&target, &strategy, max_chars, overlap, clean_noise)
                .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractImages {
            file,
            page,
            out_dir,
        } => {
            let target = normalize_path(&file);
            let out_target = normalize_path(&out_dir);
            media::extract_images(&target, page, &out_target)
                .and_then(|res| to_json_output(&res))
        }
        Commands::RenderPage {
            file,
            page,
            dpi,
            out,
        } => {
            let target = normalize_path(&file);
            let out_target = normalize_path(&out);
            media::render_page(&target, page, dpi, &out_target)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Merge { files, output } => {
            let cleaned_inputs: Vec<String> = files.iter().map(|p| normalize_path(p)).collect();
            let out_target = normalize_path(&output);
            transform::merge_pdfs(&cleaned_inputs, &out_target)
                .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractPages {
            file,
            pages,
            output,
        } => {
            let target = normalize_path(&file);
            let out_target = normalize_path(&output);
            transform::extract_pages(&target, &pages, &out_target)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Rotate {
            file,
            pages,
            angle,
            output,
        } => {
            let target = normalize_path(&file);
            let out_target = output.as_deref().map(normalize_path);
            transform::rotate_pages(&target, None, pages.as_deref(), angle, out_target.as_deref())
                .and_then(|res| to_json_output(&res))
        }
        Commands::Watermark {
            file,
            text,
            opacity,
            font_size,
            rotation,
            output,
        } => {
            let target = normalize_path(&file);
            let out_target = output.as_deref().map(normalize_path);
            watermark::add_watermark(&target, &text, opacity, font_size, rotation, out_target.as_deref())
                .and_then(|res| to_json_output(&res))
        }
    };

    match result {
        Ok(json_str) => {
            println!("{}", json_str);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.to_json());
            ExitCode::FAILURE
        }
    }
}

fn to_json_output<T: serde::Serialize>(val: &T) -> Result<String, PdfCliError> {
    serde_json::to_string_pretty(val).map_err(|e| {
        PdfCliError::unsupported(format!("JSON 序列化失败: {}", e))
    })
}

fn normalize_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/")
}