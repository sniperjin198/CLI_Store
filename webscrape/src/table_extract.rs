use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use url::Url;

use crate::error::WebScrapeError;

#[derive(Debug, Serialize, Clone)]
pub struct CardItem {
    pub index: usize,
    pub title: String,
    pub link: Option<String>,
    pub image_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub status: String,
    pub source_url: String,
    pub extract_type: String,
    pub total_items: usize,
    pub data: Value,
}

/// 执行结构化数据抽取：优先根据 CSS 选择器提取卡片条目，无选择器时自动解析网页中的表格
pub fn extract_structured_data(
    raw_html: &str,
    base_url_str: &str,
    selector_str: Option<&str>,
    limit: Option<usize>,
    output_file: Option<&str>,
) -> Result<ExtractResponse, WebScrapeError> {
    let base_url = Url::parse(base_url_str).map_err(|e| {
        WebScrapeError::invalid_parameter(format!("无效的基础 URL '{}': {}", base_url_str, e))
    })?;

    let document = Html::parse_document(raw_html);

    let (extract_type, data_val, total_count) = if let Some(sel_query) = selector_str {
        // 模式 1: 按自定义选择器抽取重复卡片项（如模型列表）
        let cards = extract_custom_cards(&document, sel_query, &base_url, limit)?;
        let count = cards.len();
        (
            format!("selector: {}", sel_query),
            serde_json::to_value(cards).unwrap_or(Value::Null),
            count,
        )
    } else {
        // 模式 2: 自动寻找并提取网页表格 <table>
        let tables = extract_html_tables(&document, limit)?;
        let count = tables.len();
        (
            "table".to_string(),
            serde_json::to_value(tables).unwrap_or(Value::Null),
            count,
        )
    };

    let response = ExtractResponse {
        status: "success".to_string(),
        source_url: base_url_str.to_string(),
        extract_type,
        total_items: total_count,
        data: data_val,
    };

    if let Some(out_path) = output_file {
        let json_str = serde_json::to_string_pretty(&response).map_err(|e| {
            WebScrapeError::unsupported(format!("序列化 JSON 失败: {}", e))
        })?;
        let mut file = File::create(out_path).map_err(|e| {
            WebScrapeError::io(format!("无法写入文件 '{}': {}", out_path, e))
        })?;
        file.write_all(json_str.as_bytes()).map_err(|e| {
            WebScrapeError::io(format!("写入提取数据失败: {}", e))
        })?;
    }

    Ok(response)
}

/// 基于 CSS 选择器提取列表项卡片（专攻模型网商品卡片、文章卡片）
fn extract_custom_cards(
    document: &Html,
    selector_str: &str,
    base_url: &Url,
    limit: Option<usize>,
) -> Result<Vec<CardItem>, WebScrapeError> {
    let selector = Selector::parse(selector_str).map_err(|e| {
        WebScrapeError::parse_error(format!("无效的 CSS 选择器 '{}': {:?}", selector_str, e))
    })?;

    let a_sel = Selector::parse("a").ok();
    let img_sel = Selector::parse("img").ok();

    let mut items = Vec::new();

    for (idx, card_el) in document.select(&selector).enumerate() {
        if let Some(max_limit) = limit {
            if items.len() >= max_limit {
                break;
            }
        }

        // 1. 抽取详情链接
        let mut link = None;
        if let Some(ref asel) = a_sel {
            if let Some(a_el) = card_el.select(asel).next() {
                if let Some(href) = a_el.value().attr("href") {
                    if let Ok(abs_url) = base_url.join(href.trim()) {
                        link = Some(abs_url.to_string());
                    }
                }
            }
        }

        // 2. 抽取封面预览图
        let mut image_url = None;
        if let Some(ref isel) = img_sel {
            if let Some(img_el) = card_el.select(isel).next() {
                let raw_src = img_el
                    .value()
                    .attr("src")
                    .or_else(|| img_el.value().attr("data-src"));
                if let Some(src) = raw_src {
                    if let Ok(abs_img) = base_url.join(src.trim()) {
                        image_url = Some(abs_img.to_string());
                    }
                }
            }
        }

        // 3. 抽取文本与关键属性
        let all_text = card_el.text().collect::<Vec<_>>();
        let title = all_text
            .first()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("Item_{}", idx + 1));

        let mut metadata = HashMap::new();
        let full_text_content = all_text.join(" ");
        metadata.insert("raw_text".to_string(), full_text_content.trim().to_string());

        items.push(CardItem {
            index: idx + 1,
            title,
            link,
            image_url,
            metadata,
        });
    }

    Ok(items)
}

/// 自动扫描并提取页面中的标准 <table> 标签
fn extract_html_tables(
    document: &Html,
    limit: Option<usize>,
) -> Result<Vec<Vec<HashMap<String, String>>>, WebScrapeError> {
    let table_sel = Selector::parse("table").map_err(|e| {
        WebScrapeError::parse_error(format!("解析表格选择器失败: {:?}", e))
    })?;
    let tr_sel = Selector::parse("tr").map_err(|e| {
        WebScrapeError::parse_error(format!("解析行选择器失败: {:?}", e))
    })?;
    let th_sel = Selector::parse("th").ok();
    let td_sel = Selector::parse("td").ok();

    let mut tables_data = Vec::new();

    for table in document.select(&table_sel) {
        if let Some(max_limit) = limit {
            if tables_data.len() >= max_limit {
                break;
            }
        }

        let mut headers = Vec::new();
        let mut rows_data = Vec::new();

        // 探测表头
        if let Some(ref ths) = th_sel {
            for th in table.select(ths) {
                headers.push(th.text().collect::<String>().trim().to_string());
            }
        }

        // 逐行解析
        for tr in table.select(&tr_sel) {
            let mut row_map = HashMap::new();
            if let Some(ref tds) = td_sel {
                let cell_texts: Vec<String> = tr
                    .select(tds)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect();

                if !cell_texts.is_empty() {
                    for (i, cell_val) in cell_texts.into_iter().enumerate() {
                        let header_key = headers
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("col_{}", i + 1));
                        row_map.insert(header_key, cell_val);
                    }
                    rows_data.push(row_map);
                }
            }
        }

        if !rows_data.is_empty() {
            tables_data.push(rows_data);
        }
    }

    Ok(tables_data)
}