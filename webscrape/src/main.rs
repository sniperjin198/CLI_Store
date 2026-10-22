mod client;
mod engine;
mod error;
mod links;
mod markdown;
mod readability;
mod table_extract;

use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::client::{build_http_client, fetch_html};
use crate::engine::{run_aria2c_download, run_monolith_archive, run_ytdlp_download};
use crate::error::WebScrapeError;

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
#[command(
    name = "webscrape",
    version = "0.2.0",
    about = "WebScrape CLI - Agent-First 高性能网页内容抓取、纯内存浏览与引擎调度门面"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 目标网页或资源 URL（顶层大模型直调模式）
    #[arg(short, long, global = true)]
    url: Option<String>,

    /// 输出文件完整路径（顶层大模型直调模式）
    #[arg(short, long, global = true)]
    output: Option<String>,

    /// 存盘格式：md、html、bin、audio 或 video
    #[arg(short, long, global = true)]
    format: Option<String>,

    /// 视频下载指定最大分辨率（如 1080、720）
    #[arg(long, global = true)]
    resolution: Option<u32>,

    /// 是否本地化下载插图（仅存盘为 md 时生效）
    #[arg(long, default_value_t = false, global = true)]
    with_images: bool,

    #[command(flatten)]
    net: CommonNetArgs,
}

#[derive(Args, Debug, Clone)]
pub struct CommonNetArgs {
    #[arg(short, long, default_value_t = 30, global = true)]
    pub timeout: u64,

    #[arg(short, long, global = true)]
    pub proxy: Option<String>,

    #[arg(long, global = true)]
    pub cookie: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 网页纯读浏览：提取纯净正文直接输出至控制台（零写盘，大模型主阅读通道）
    Get {
        #[arg(index = 1)]
        target_url: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[command(flatten)]
        net: CommonNetArgs,
    },

    /// 资源物理存盘：内部自动路由至 aria2c / monolith / yt-dlp 或本地 Markdown 引擎
    Save {
        #[arg(index = 1)]
        target_url: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(short, long)]
        format: Option<String>,

        #[arg(long)]
        resolution: Option<u32>,

        #[arg(long, default_value_t = false)]
        with_images: bool,

        #[command(flatten)]
        net: CommonNetArgs,
    },

    /// 链接流与要闻公告监控采集
    Links {
        #[arg(index = 1)]
        target_url: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[arg(short, long)]
        limit: Option<usize>,

        #[arg(short, long)]
        output: Option<String>,

        #[command(flatten)]
        net: CommonNetArgs,
    },

    /// 结构化卡片与数据表格提取
    Extract {
        #[arg(index = 1)]
        target_url: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[arg(short, long)]
        selector: Option<String>,

        #[arg(short, long)]
        limit: Option<usize>,

        #[arg(short, long)]
        output: Option<String>,

        #[command(flatten)]
        net: CommonNetArgs,
    },
}

fn is_binary_extension(ext: &str) -> bool {
    matches!(
        ext,
        "xlsx"
            | "xls"
            | "doc"
            | "docx"
            | "pdf"
            | "zip"
            | "rar"
            | "7z"
            | "tar"
            | "gz"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "csv"
            | "exe"
            | "bin"
    )
}

fn is_video_target(url: &str, format: Option<&str>) -> bool {
    if let Some(f) = format {
        if f.eq_ignore_ascii_case("video") || f.eq_ignore_ascii_case("audio") {
            return true;
        }
    }
    let lower = url.to_lowercase();
    lower.contains("bilibili.com")
        || lower.contains("youtube.com")
        || lower.contains("youtu.be")
        || lower.ends_with(".mp4")
        || lower.ends_with(".mkv")
        || lower.ends_with(".flv")
}

fn execute_save(
    url: &str,
    output: &str,
    format: Option<String>,
    resolution: Option<u32>,
    with_images: bool,
    net: &CommonNetArgs,
) -> Result<String, WebScrapeError> {
    let path_obj = PathBuf::from(output);
    if let Some(parent) = path_obj.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let file_ext = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 路由分支 1：视频流媒体 -> 委托 bin\yt-dlp.exe
    if is_video_target(url, format.as_deref()) {
        run_ytdlp_download(
            url,
            output,
            format.as_deref(),
            resolution,
            net.proxy.as_deref(),
        )?;
        let resp = serde_json::json!({
            "status": "success",
            "engine": "yt-dlp",
            "output_file": output.replace('\\', "/"),
            "format": format.unwrap_or_else(|| "video".to_string()),
            "message": "音视频资源已成功由 yt-dlp 引擎下载完成"
        });
        return to_json_output(&resp);
    }

    // 路由分支 2：大文件/表格附件 -> 委托 bin\aria2c.exe
    let is_bin_format = format.as_deref().map(|s| s == "bin" || s == "raw").unwrap_or(false);
    if is_bin_format || is_binary_extension(&file_ext) {
        let file_size = run_aria2c_download(url, output, net.proxy.as_deref())?;
        let resp = serde_json::json!({
            "status": "success",
            "engine": "aria2c",
            "output_file": output.replace('\\', "/"),
            "format": "binary",
            "size_bytes": file_size,
            "message": format!("二进制附件已通过 aria2c 多线程引擎成功下载（大小：{} 字节）", file_size)
        });
        return to_json_output(&resp);
    }

    // 路由分支 3：单文件完整 HTML 网页归档 -> 委托 bin\monolith.exe
    let is_html_format = format.as_deref().map(|s| s == "html").unwrap_or(false);
    if is_html_format || file_ext == "html" || file_ext == "htm" {
        run_monolith_archive(url, output)?;
        let resp = serde_json::json!({
            "status": "success",
            "engine": "monolith",
            "output_file": output.replace('\\', "/"),
            "format": "html",
            "message": "完整网页已通过 monolith 引擎单文件离线归档完成"
        });
        return to_json_output(&resp);
    }

    // 路由分支 4：图文 Markdown 笔记 -> 使用内置 Rust 原生解析落盘
    let client = build_http_client(
        Some(net.timeout),
        net.proxy.as_deref(),
        net.cookie.as_deref(),
    )?;
    let raw_html = fetch_html(&client, url)?;
    let article = readability::extract_article(&raw_html, url)?;
    let resp = markdown::process_article_markdown(
        &client,
        &article,
        Some(output),
        with_images,
    )?;

    to_json_output(&resp)
}

fn execute_get(url: &str, net: &CommonNetArgs) -> Result<String, WebScrapeError> {
    let client = build_http_client(
        Some(net.timeout),
        net.proxy.as_deref(),
        net.cookie.as_deref(),
    )?;

    let raw_html = fetch_html(&client, url)?;
    let article = readability::extract_article(&raw_html, url)?;
    let resp = markdown::process_article_markdown(&client, &article, None, false)?;

    to_json_output(&resp)
}

fn run_app(cli: Cli) -> Result<String, WebScrapeError> {
    if cli.command.is_none() {
        if let Some(target_url) = cli.url {
            if let Some(out_path) = cli.output {
                return execute_save(
                    &target_url,
                    &out_path,
                    cli.format,
                    cli.resolution,
                    cli.with_images,
                    &cli.net,
                );
            } else {
                return execute_get(&target_url, &cli.net);
            }
        } else {
            return Err(WebScrapeError::unsupported(
                "请提供子命令 (get/save/links/extract) 或指定 --url 参数。可运行 --help 查看用法。",
            ));
        }
    }

    match cli.command.unwrap() {
        Commands::Get {
            target_url,
            url,
            net,
        } => {
            let final_url = url.or(target_url).ok_or_else(|| {
                WebScrapeError::unsupported("缺少目标 URL，请通过位置参数或 --url 传入。")
            })?;
            execute_get(&final_url, &net)
        }

        Commands::Save {
            target_url,
            url,
            output,
            format,
            resolution,
            with_images,
            net,
        } => {
            let final_url = url.or(target_url).ok_or_else(|| {
                WebScrapeError::unsupported("缺少目标 URL，请通过位置参数或 --url 传入。")
            })?;

            let final_output = output
                .or(cli.output)
                .ok_or_else(|| WebScrapeError::unsupported("保存文件必须指定输出路径 (-o 或 --output)。"))?;

            let final_format = format.or(cli.format);
            let final_res = resolution.or(cli.resolution);
            let final_with_images = with_images || cli.with_images;

            execute_save(
                &final_url,
                &final_output,
                final_format,
                final_res,
                final_with_images,
                &net,
            )
        }

        Commands::Links {
            target_url,
            url,
            limit,
            output,
            net,
        } => {
            let final_url = url.or(target_url).ok_or_else(|| {
                WebScrapeError::unsupported("缺少目标 URL，请通过位置参数或 --url 传入。")
            })?;
            let client = build_http_client(
                Some(net.timeout),
                net.proxy.as_deref(),
                net.cookie.as_deref(),
            )?;
            let raw_html = fetch_html(&client, &final_url)?;
            let resp = links::extract_links(&raw_html, &final_url, limit, output.as_deref())?;
            to_json_output(&resp)
        }

        Commands::Extract {
            target_url,
            url,
            selector,
            limit,
            output,
            net,
        } => {
            let final_url = url.or(target_url).ok_or_else(|| {
                WebScrapeError::unsupported("缺少目标 URL，请通过位置参数或 --url 传入。")
            })?;
            let client = build_http_client(
                Some(net.timeout),
                net.proxy.as_deref(),
                net.cookie.as_deref(),
            )?;
            let raw_html = fetch_html(&client, &final_url)?;
            let resp = table_extract::extract_structured_data(
                &raw_html,
                &final_url,
                selector.as_deref(),
                limit,
                output.as_deref(),
            )?;
            to_json_output(&resp)
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

fn to_json_output<T: serde::Serialize>(val: &T) -> Result<String, WebScrapeError> {
    serde_json::to_string_pretty(val).map_err(|e| {
        WebScrapeError::unsupported(format!("JSON 序列化失败: {}", e))
    })
}