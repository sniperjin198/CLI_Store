mod chunker;
mod cmap;
mod error;
mod extractor;
mod inspect;
mod io_channel;
mod layout;
mod media;
mod transform;
mod watermark;

use clap::{Parser, Subcommand};
use std::process::ExitCode;
use crate::error::PdfCliError;
use crate::io_channel::CommonInputArgs;

#[cfg(windows)]
fn init_windows_utf8_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCP(wCodePageID: u32) -> i32;
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }
    unsafe {
        SetConsoleCP(65001);
        SetConsoleOutputCP(65001);
    }
}

#[cfg(not(windows))]
fn init_windows_utf8_console() {}

#[derive(Parser)]
#[command(name = "pdf_cli", version = "0.2.0", about = "PDF CLI - Agent-First 高性能自动化套件")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 宏观感知驾驶舱
    Inspect {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(long)]
        only: Option<String>,
    },

    /// 文本提取
    ExtractText {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        page: Option<usize>,
        #[arg(short, long, visible_alias = "pages")]
        range: Option<String>,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long, default_value_t = false)]
        clean_noise: bool,
        #[arg(long, default_value_t = false)]
        with_bbox: bool,
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// 页面版式几何自省
    InspectLayout {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        page: usize,
    },

    /// RAG 知识库语义切片
    Chunk {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(long, default_value = "heading")]
        strategy: String,
        #[arg(long, default_value_t = 500)]
        max_chars: usize,
        #[arg(long, default_value_t = 80)]
        overlap: usize,
        #[arg(long, default_value_t = true)]
        clean_noise: bool,
    },

    /// 导出内嵌图像
    ExtractImages {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        page: Option<usize>,
        #[arg(short, long, default_value = "./extracted_images")]
        out_dir: String,
    },

    /// 高清 300DPI 渲染
    RenderPage {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        page: usize,
        #[arg(long, default_value_t = 300)]
        dpi: u32,
        #[arg(short, long, visible_alias = "output")]
        out: String,
    },

    /// 图片打包为 PDF（支持单图/多图 --files / 目录 --dir / 清单 --list）
    #[command(name = "images-to-pdf", visible_alias = "img2pdf")]
    ImagesToPdf {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long, visible_alias = "out")]
        output: String,
    },

    /// 物理合并（支持 --dir / --list / --files）
    Merge {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        output: String,
    },

    /// 物理抽取
    ExtractPages {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long, visible_alias = "range")]
        pages: String,
        #[arg(short, long)]
        output: String,
    },

    /// 页面旋转（支持单文件或 --dir 批量处理）
    Rotate {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long, visible_alias = "range")]
        pages: Option<String>,
        #[arg(short, long, visible_alias = "degrees")]
        angle: i32,
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 水印印章注入（支持单文件或 --dir 批量注入）
    Watermark {
        #[command(flatten)]
        input: CommonInputArgs,
        #[arg(short, long)]
        text: String,
        #[arg(long, default_value_t = 0.2)]
        opacity: f64,
        #[arg(long, default_value_t = 42.0)]
        font_size: f64,
        #[arg(long, default_value_t = 45.0)]
        rotation: f64,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn run_app(cli: Cli) -> Result<String, PdfCliError> {
    match cli.command {
        Commands::Inspect { input, only } => {
            let files = input.resolve_inputs(None)?;
            inspect::inspect_pdf(&files[0].to_string_lossy(), only.as_deref())
                .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractText {
            input,
            page,
            range,
            heading,
            clean_noise,
            with_bbox,
            format,
        } => {
            let files = input.resolve_inputs(None)?;
            extractor::extract_text(
                &files[0].to_string_lossy(),
                page,
                range.as_deref(),
                heading.as_deref(),
                clean_noise,
                with_bbox,
                &format,
            )
            .and_then(|res| to_json_output(&res))
        }
        Commands::InspectLayout { input, page } => {
            let files = input.resolve_inputs(None)?;
            layout::inspect_page_layout(&files[0].to_string_lossy(), page)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Chunk {
            input,
            strategy,
            max_chars,
            overlap,
            clean_noise,
        } => {
            let files = input.resolve_inputs(None)?;
            chunker::chunk_document(
                &files[0].to_string_lossy(),
                &strategy,
                max_chars,
                overlap,
                clean_noise,
            )
            .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractImages {
            input,
            page,
            out_dir,
        } => {
            let files = input.resolve_inputs(None)?;
            media::extract_images(&files[0].to_string_lossy(), page, &out_dir)
                .and_then(|res| to_json_output(&res))
        }
        Commands::RenderPage {
            input,
            page,
            dpi,
            out,
        } => {
            let files = input.resolve_inputs(None)?;
            media::render_page(&files[0].to_string_lossy(), page, dpi, &out)
                .and_then(|res| to_json_output(&res))
        }
        Commands::ImagesToPdf { input, output } => {
            let files = input.resolve_image_inputs()?;
            transform::images_to_pdf(&files, &output)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Merge { input, output } => {
            let files = input.resolve_inputs(Some(&output))?;
            transform::merge_resolved_pdfs(&files, &output)
                .and_then(|res| to_json_output(&res))
        }
        Commands::ExtractPages {
            input,
            pages,
            output,
        } => {
            let files = input.resolve_inputs(Some(&output))?;
            transform::extract_pages(&files[0].to_string_lossy(), &pages, &output)
                .and_then(|res| to_json_output(&res))
        }
        Commands::Rotate {
            input,
            pages,
            angle,
            output,
        } => {
            let files = input.resolve_inputs(output.as_deref())?;
            transform::rotate_pages_batch(&files, pages.as_deref(), angle, output.as_deref())
                .and_then(|res| to_json_output(&res))
        }
        Commands::Watermark {
            input,
            text,
            opacity,
            font_size,
            rotation,
            output,
        } => {
            let files = input.resolve_inputs(output.as_deref())?;
            watermark::watermark_batch(
                &files,
                &text,
                opacity,
                font_size,
                rotation,
                output.as_deref(),
            )
            .and_then(|res| to_json_output(&res))
        }
    }
}

fn main() -> ExitCode {
    init_windows_utf8_console();
    let cli = Cli::parse();

    match run_app(cli) {
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