use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::WebScrapeError;

/// 严格在同级 bin\ 目录下查找专用工具，未找到直接报错引导，杜绝系统 PATH 污染
pub fn resolve_engine_tool(tool_name: &str) -> Result<PathBuf, WebScrapeError> {
    let mut exe_dir = std::env::current_exe().map_err(|e| {
        WebScrapeError::io(format!("无法定位 webscrape 自身安装路径: {}", e))
    })?;
    exe_dir.pop(); // 回退到 E:\.Cli\ 目录

    let engine_path = exe_dir.join("bin").join(tool_name);
    if engine_path.exists() && engine_path.is_file() {
        Ok(engine_path)
    } else {
        Err(WebScrapeError::unsupported(format!(
            "缺少核心下载引擎组件: 未找到 '{}'。请将该工具放置于专用依赖目录 '{}' 下后重试。",
            tool_name,
            engine_path.display()
        )))
    }
}

/// 委托 aria2c.exe 执行多线程断点续传下载（针对 xlsx, pdf, zip, docx 等大文件附件）
pub fn run_aria2c_download(
    url: &str,
    output_path_str: &str,
    proxy: Option<&str>,
) -> Result<usize, WebScrapeError> {
    let aria2_exe = resolve_engine_tool("aria2c.exe")?;
    let path_obj = Path::new(output_path_str);

    let output_dir = path_obj
        .parent()
        .and_then(|p| p.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(".");

    let file_name = path_obj
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| WebScrapeError::invalid_parameter("无效的输出文件名"))?;

    let mut cmd = Command::new(&aria2_exe);
    cmd.args([
        "-x", "4",
        "-s", "4",
        "-k", "1M",
        "--connect-timeout=5",
        "--timeout=15",
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
        "--auto-file-renaming=false",
        "--allow-overwrite=true",
        "-d", output_dir,
        "-o", file_name,
    ]);

    if let Some(p) = proxy {
        cmd.arg(format!("--all-proxy={}", p));
    }

    cmd.arg(url);

    let status = cmd.status().map_err(|e| {
        WebScrapeError::io(format!("执行 aria2c.exe 启动失败: {}", e))
    })?;

    if !status.success() {
        return Err(WebScrapeError::io(format!(
            "aria2c 下载异常终止，退出代码: {:?}",
            status.code()
        )));
    }

    let file_size = std::fs::metadata(path_obj).map(|m| m.len() as usize).unwrap_or(0);
    Ok(file_size)
}

/// 委托 monolith.exe 执行全要素网页单文件 HTML 离线打包
pub fn run_monolith_archive(
    url: &str,
    output_path_str: &str,
) -> Result<(), WebScrapeError> {
    let monolith_exe = resolve_engine_tool("monolith.exe")?;

    let mut cmd = Command::new(&monolith_exe);
    cmd.args([
        "-o", output_path_str,
        "-I", // 忽略不安全资源
        "-a", // 自动阻止内嵌大型音视频
        url,
    ]);

    let status = cmd.status().map_err(|e| {
        WebScrapeError::io(format!("执行 monolith.exe 启动失败: {}", e))
    })?;

    if !status.success() {
        return Err(WebScrapeError::io(format!(
            "monolith 打包异常终止，退出代码: {:?}",
            status.code()
        )));
    }

    Ok(())
}

/// 委托 yt-dlp.exe 执行音视频流媒体下载
pub fn run_ytdlp_download(
    url: &str,
    output_path_str: &str,
    format_type: Option<&str>,
    resolution: Option<u32>,
    proxy: Option<&str>,
) -> Result<(), WebScrapeError> {
    let ytdlp_exe = resolve_engine_tool("yt-dlp.exe")?;
    let path_obj = Path::new(output_path_str);

    let output_dir = path_obj
        .parent()
        .and_then(|p| p.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(".");

    let mut cmd = Command::new(&ytdlp_exe);
    cmd.args(["-P", output_dir]);

    if let Some(file_name) = path_obj.file_name().and_then(|f| f.to_str()) {
        cmd.args(["-o", file_name]);
    }

    if let Some("audio") = format_type {
        cmd.args(["-x", "--audio-format", "mp3"]);
    } else if let Some(res) = resolution {
        cmd.args(["-f", &format!("bestvideo[height<={res}]+bestaudio/best[height<={res}]/best")]);
    }

    if let Some(p) = proxy {
        cmd.args(["--proxy", p]);
    }

    cmd.arg(url);

    let status = cmd.status().map_err(|e| {
        WebScrapeError::io(format!("执行 yt-dlp.exe 启动失败: {}", e))
    })?;

    if !status.success() {
        return Err(WebScrapeError::io(format!(
            "yt-dlp 视频下载异常终止，退出代码: {:?}",
            status.code()
        )));
    }

    Ok(())
}