use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

use crate::error::WebScrapeError;

#[derive(Debug, Serialize, Clone)]
pub struct ExtractedArticle {
    pub title: String,
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub clean_html: String,
}

/// 常见正文容器选择器（按命中优先级递减）
const CONTENT_SELECTORS: &[&str] = &[
    "#js_content",              // 微信公众号正文容器
    "article",                  // 现代语义化正文
    "main article",
    "#article-content",
    ".article-content",
    ".post-content",
    ".entry-content",
    "#content",
    ".content",
    "#main-content",
    ".main-content",
    ".detail-content",
    ".text",
    "main",
];

/// 必须剔除的噪声标签
const NOISE_TAGS: &[&str] = &[
    "script",
    "style",
    "noscript",
    "iframe",
    "svg",
    "header",
    "footer",
    "nav",
    "aside",
    ".ad",
    ".advertisement",
    ".share-box",
    ".qrcode",
    "#js_toobar3",             // 微信底部工具条
    ".reward_area",            // 微信赞赏区
    ".rich_media_area_extra",  // 微信推荐扩展区
];

/// 解析 HTML 并提取正文区块与元数据
pub fn extract_article(
    raw_html: &str,
    base_url_str: &str,
) -> Result<ExtractedArticle, WebScrapeError> {
    let base_url = Url::parse(base_url_str).ok();
    let document = Html::parse_document(raw_html);

    // 1. 抽取标题（优先 h1，其次 title，再尝试微信特征）
    let title = extract_title(&document);

    // 2. 抽取作者与发布日期
    let author = extract_meta_or_selector(
        &document,
        &["meta[name='author']", "meta[property='article:author']", ".rich_media_meta_text"],
    );
    let publish_date = extract_meta_or_selector(
        &document,
        &[
            "meta[property='article:published_time']",
            "meta[name='pubdate']",
            "meta[name='publishdate']",
            "#publish_time",
            ".publish-time",
            ".time",
        ],
    );

    // 3. 定位正文容器
    let mut selected_element = None;
    for selector_str in CONTENT_SELECTORS {
        if let Ok(sel) = Selector::parse(selector_str) {
            if let Some(el) = document.select(&sel).next() {
                let text_len = el.text().collect::<String>().trim().chars().count();
                if text_len > 30 {
                    selected_element = Some(el);
                    break;
                }
            }
        }
    }

    // 4. 若无特定容器匹配，回退取 body
    let body_html = if let Some(el) = selected_element {
        el.html()
    } else if let Ok(body_sel) = Selector::parse("body") {
        document.select(&body_sel).next().map(|b| b.html()).unwrap_or_else(|| raw_html.to_string())
    } else {
        raw_html.to_string()
    };

    // 5. 正文清洗与绝对路径补全（已修复空串炸裂 bug）
    let clean_html = sanitize_and_fix_urls(&body_html, base_url.as_ref());

    Ok(ExtractedArticle {
        title,
        author,
        publish_date,
        clean_html,
    })
}

/// 提取网页标题
fn extract_title(document: &Html) -> String {
    let wx_title_sel = Selector::parse("#activity-name").ok();
    if let Some(ref sel) = wx_title_sel {
        if let Some(el) = document.select(sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    if let Ok(h1_sel) = Selector::parse("h1") {
        if let Some(el) = document.select(&h1_sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    if let Ok(title_sel) = Selector::parse("title") {
        if let Some(el) = document.select(&title_sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    "未命名网页".to_string()
}

/// 按选择器探测元数据
fn extract_meta_or_selector(document: &Html, selectors: &[&str]) -> Option<String> {
    for s in selectors {
        if let Ok(sel) = Selector::parse(s) {
            if let Some(el) = document.select(&sel).next() {
                if s.starts_with("meta") {
                    if let Some(content) = el.value().attr("content") {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                } else {
                    let text = el.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
    }
    None
}

/// 清洗噪声标签、修复超链接与图片路径（加入非空与非根边界防护）
fn sanitize_and_fix_urls(html_snippet: &str, base_url: Option<&Url>) -> String {
    let mut clean_html = html_snippet.to_string();

    // 1. 过滤噪声节点
    for noise in NOISE_TAGS {
        if let Ok(sel) = Selector::parse(noise) {
            let doc_temp = Html::parse_fragment(&clean_html);
            let mut noise_texts = Vec::new();
            for el in doc_temp.select(&sel) {
                noise_texts.push(el.html());
            }
            for noise_block in noise_texts {
                if !noise_block.is_empty() {
                    clean_html = clean_html.replace(&noise_block, "");
                }
            }
        }
    }

    // 2. 将 data-src 归一化为通用 src
    clean_html = clean_html
        .replace("data-src=\"", "src=\"")
        .replace("data-src='", "src='");

    // 3. 补全相对路径（严格防御空字符串与单纯锚点）
    if let Some(base) = base_url {
        let doc_temp = Html::parse_fragment(&clean_html);
        
        // 修复 img 标签的相对路径
        if let Ok(img_sel) = Selector::parse("img") {
            for el in doc_temp.select(&img_sel) {
                if let Some(src) = el.value().attr("src") {
                    let trimmed = src.trim();
                    // 核心防御：严禁对空串、斜杠、javascript伪协议、锚点执行 replace
                    if !trimmed.is_empty()
                        && trimmed != "/"
                        && !trimmed.starts_with("http://")
                        && !trimmed.starts_with("https://")
                        && !trimmed.starts_with("data:")
                        && !trimmed.starts_with('#')
                        && !trimmed.starts_with("javascript:")
                    {
                        if let Ok(abs_url) = base.join(trimmed) {
                            clean_html = clean_html.replace(trimmed, abs_url.as_str());
                        }
                    }
                }
            }
        }

        // 修复 a 标签的相对路径
        if let Ok(a_sel) = Selector::parse("a") {
            for el in doc_temp.select(&a_sel) {
                if let Some(href) = el.value().attr("href") {
                    let trimmed = href.trim();
                    // 核心防御：严禁对空串、单斜杠、纯锚点执行 replace
                    if !trimmed.is_empty()
                        && trimmed != "/"
                        && !trimmed.starts_with("http://")
                        && !trimmed.starts_with("https://")
                        && !trimmed.starts_with("javascript:")
                        && !trimmed.starts_with('#')
                        && !trimmed.starts_with("mailto:")
                        && !trimmed.starts_with("tel:")
                    {
                        if let Ok(abs_url) = base.join(trimmed) {
                            clean_html = clean_html.replace(trimmed, abs_url.as_str());
                        }
                    }
                }
            }
        }
    }

    clean_html
}