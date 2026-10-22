use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::client::fetch_bytes;
use crate::error::WebScrapeError;
use crate::readability::ExtractedArticle;

#[derive(Debug, Serialize)]
pub struct MarkdownOutputResponse {
    pub status: String,
    pub title: String,
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub output_file: Option<String>,
    pub images_downloaded: usize,
    pub content: String,
    pub message: String,
}

/// 处理文章转 Markdown，支持纯内存提取（output_path 为 None）或本地物理存盘（output_path 为 Some）
pub fn process_article_markdown(
    client: &Client,
    article: &ExtractedArticle,
    output_path: Option<&str>,
    with_images: bool,
) -> Result<MarkdownOutputResponse, WebScrapeError> {
    let mut downloaded_count = 0;

    // 存盘且需要离线下载图片时，直接在 DOM 上完成图片替换
    let final_html_for_md = if let (Some(out_str), true) = (output_path, with_images) {
        let out_file_path = PathBuf::from(out_str);
        let stem = out_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("article");
        let parent_dir = out_file_path.parent().unwrap_or_else(|| Path::new("."));
        let images_dir_name = format!("{}_images", stem);
        let images_dir_path = parent_dir.join(&images_dir_name);

        if !images_dir_path.exists() {
            fs::create_dir_all(&images_dir_path).map_err(|e| {
                WebScrapeError::io(format!("无法创建图片资源目录 '{}': {}", images_dir_path.display(), e))
            })?;
        }

        // 解析 DOM，按出现顺序逐一提取并替换
        let doc = Html::parse_fragment(&article.clean_html);
        let img_sel = Selector::parse("img").map_err(|e| {
            WebScrapeError::io(format!("解析 img 标签选择器失败: {:?}", e))
        })?;

        // 收集所有有效图片链接（保持正文中出现的先后顺序）
        let mut ordered_images = Vec::new();
        for el in doc.select(&img_sel) {
            let src = el.value().attr("data-src")
                .or_else(|| el.value().attr("src"))
                .map(|s| s.trim().to_string());
            if let Some(url) = src {
                if url.starts_with("http://") || url.starts_with("https://") {
                    ordered_images.push(url);
                }
            }
        }

        // 构造替换映射表 (原始URL/实体转义URL -> 本地相对路径)
        let mut replace_pairs = Vec::new();
        for (idx, original_url) in ordered_images.iter().enumerate() {
            let clean_fetch_url = original_url.replace("&amp;", "&");
            if let Ok((bytes, content_type)) = fetch_bytes(client, &clean_fetch_url) {
                let ext = detect_image_extension(&bytes, content_type.as_deref());
                let img_filename = format!("img_{:03}.{}", idx + 1, ext);
                let local_file_path = images_dir_path.join(&img_filename);

                if let Ok(mut f) = File::create(&local_file_path) {
                    if f.write_all(&bytes).is_ok() {
                        downloaded_count += 1;
                        // 不做 URL 百分号转义，直接使用原生相对路径，兼容各种 Markdown 阅读器
                        let relative_ref = format!("./{}/{}", images_dir_name, img_filename);
                        replace_pairs.push((original_url.clone(), relative_ref.clone()));
                        replace_pairs.push((clean_fetch_url, relative_ref.clone()));
                        replace_pairs.push((original_url.replace('&', "&amp;"), relative_ref));
                    }
                }
            }
        }

        let mut transformed = article.clean_html.clone();
        for (from, to) in replace_pairs {
            if !from.is_empty() {
                transformed = transformed.replace(&from, &to);
            }
        }
        transformed
    } else {
        article.clean_html.clone()
    };

    // 将 HTML 转为 Markdown
    let md_body = html2md::parse_html(&final_html_for_md);
    let mut final_markdown = String::new();
    final_markdown.push_str(&format!("# {}\n\n", article.title));

    if let Some(ref author) = article.author {
        final_markdown.push_str(&format!("**作者**：{}  \n", author));
    }
    if let Some(ref date) = article.publish_date {
        final_markdown.push_str(&format!("**发布时间**：{}  \n", date));
    }
    if article.author.is_some() || article.publish_date.is_some() {
        final_markdown.push_str("\n---\n\n");
    }
    final_markdown.push_str(&md_body);

    // 存盘分支判断
    if let Some(out_str) = output_path {
        let out_file_path = PathBuf::from(out_str);
        if let Some(parent) = out_file_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    WebScrapeError::io(format!("无法创建输出目录 '{}': {}", parent.display(), e))
                })?;
            }
        }
        let mut file = File::create(&out_file_path).map_err(|e| {
            WebScrapeError::io(format!("无法写入文件 '{}': {}", out_str, e))
        })?;
        file.write_all(final_markdown.as_bytes()).map_err(|e| {
            WebScrapeError::io(format!("保存内容失败: {}", e))
        })?;

        Ok(MarkdownOutputResponse {
            status: "success".to_string(),
            title: article.title.clone(),
            author: article.author.clone(),
            publish_date: article.publish_date.clone(),
            output_file: Some(out_str.replace('\\', "/")),
            images_downloaded: downloaded_count,
            content: final_markdown,
            message: format!("已成功提取并保存至本地文件（本地化图片 {} 张）", downloaded_count),
        })
    } else {
        Ok(MarkdownOutputResponse {
            status: "success".to_string(),
            title: article.title.clone(),
            author: article.author.clone(),
            publish_date: article.publish_date.clone(),
            output_file: None,
            images_downloaded: 0,
            content: final_markdown,
            message: "网页正文提取成功（纯内存直读模式，未写入磁盘）".to_string(),
        })
    }
}

fn detect_image_extension(bytes: &[u8], content_type: Option<&str>) -> &'static str {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return "jpg";
    }
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        return "png";
    }
    if bytes.len() >= 4 && &bytes[0..4] == b"RIFF" && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        return "webp";
    }
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return "gif";
    }
    if bytes.len() >= 2 && &bytes[0..2] == b"BM" {
        return "bmp";
    }

    if let Some(ct) = content_type {
        let lower = ct.to_lowercase();
        if lower.contains("jpeg") || lower.contains("jpg") {
            return "jpg";
        }
        if lower.contains("png") {
            return "png";
        }
        if lower.contains("webp") {
            return "webp";
        }
        if lower.contains("gif") {
            return "gif";
        }
    }

    "jpg"
}