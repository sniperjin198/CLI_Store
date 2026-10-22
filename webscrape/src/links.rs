use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use url::Url;

use crate::error::WebScrapeError;

#[derive(Debug, Serialize, Clone)]
pub struct LinkItem {
    pub index: usize,
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinksResponse {
    pub status: String,
    pub source_url: String,
    pub total_links: usize,
    pub links: Vec<LinkItem>,
}

/// 从 HTML 页面中提取有效超链接、标题与发布日期
pub fn extract_links(
    raw_html: &str,
    base_url_str: &str,
    limit: Option<usize>,
    output_file: Option<&str>,
) -> Result<LinksResponse, WebScrapeError> {
    let base_url = Url::parse(base_url_str).map_err(|e| {
        WebScrapeError::invalid_parameter(format!("无效的基础 URL '{}': {}", base_url_str, e))
    })?;

    let document = Html::parse_document(raw_html);
    let a_selector = Selector::parse("a").map_err(|e| {
        WebScrapeError::parse_error(format!("选择器语法错误: {:?}", e))
    })?;

    let mut result_links = Vec::new();
    let mut visited_urls = HashSet::new();
    let mut current_idx = 1;

    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            let trimmed_href = href.trim();

            // 过滤无效伪链接
            if trimmed_href.is_empty()
                || trimmed_href.starts_with('#')
                || trimmed_href.starts_with("javascript:")
                || trimmed_href.starts_with("mailto:")
                || trimmed_href.starts_with("tel:")
            {
                continue;
            }

            // 补全绝对路径
            let full_url = match base_url.join(trimmed_href) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };

            // URL 去重
            if !visited_urls.insert(full_url.clone()) {
                continue;
            }

            // 提取链接标题文本
            let text = element.text().collect::<String>().trim().to_string();
            // 过滤过短或纯占位符的无意义文字
            if text.chars().count() < 2 {
                continue;
            }

            // 智能启发式探测关联的发布时间
            let date = detect_associated_date(&element);

            result_links.push(LinkItem {
                index: current_idx,
                title: text,
                url: full_url,
                date,
            });

            current_idx += 1;

            // 若指定了条数限制则提前截断
            if let Some(max_count) = limit {
                if result_links.len() >= max_count {
                    break;
                }
            }
        }
    }

    let total = result_links.len();
    let response = LinksResponse {
        status: "success".to_string(),
        source_url: base_url_str.to_string(),
        total_links: total,
        links: result_links,
    };

    // 如果指定了输出文件路径，则保存为 JSON 文件
    if let Some(out_path) = output_file {
        let json_data = serde_json::to_string_pretty(&response).map_err(|e| {
            WebScrapeError::unsupported(format!("序列化 JSON 失败: {}", e))
        })?;
        let mut file = File::create(out_path).map_err(|e| {
            WebScrapeError::io(format!("无法写入文件 '{}': {}", out_path, e))
        })?;
        file.write_all(json_data.as_bytes()).map_err(|e| {
            WebScrapeError::io(format!("写入链接数据失败: {}", e))
        })?;
    }

    Ok(response)
}

/// 启发式识别链接周边伴随的日期字符串（如父节点 `<li>` 内的 `[2026-03-09]` 或 `<span>` 标签）
fn detect_associated_date(element: &scraper::ElementRef) -> Option<String> {
    // 优先从父级容器（常见于 <li> 包含 <a> 和 <span>日期</span>）中探测
    if let Some(parent) = element.parent().and_then(scraper::ElementRef::wrap) {
        let parent_text = parent.text().collect::<String>();
        if let Some(date_str) = find_date_in_text(&parent_text) {
            return Some(date_str);
        }
    }

    // 探测当前 a 标签自身的文本
    let self_text = element.text().collect::<String>();
    find_date_in_text(&self_text)
}

/// 匹配常见的中文和国际标准日期格式：2026-03-09, 2026/03/09, 2026.03.09, 2026年03月09日
fn find_date_in_text(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 8 {
        return None;
    }

    // 滑动窗口寻找标准 4 位年份开头的日期模式
    for i in 0..=chars.len().saturating_sub(8) {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
        {
            let sep1 = chars[i + 4];
            if sep1 == '-' || sep1 == '/' || sep1 == '.' || sep1 == '年' {
                // 截取潜在的日期长度（通常 10 ~ 11 个字符）
                let end = (i + 11).min(chars.len());
                let slice: String = chars[i..end].iter().collect();
                let clean = slice.trim().trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')');
                if clean.len() >= 8 {
                    return Some(clean.to_string());
                }
            }
        }
    }

    None
}