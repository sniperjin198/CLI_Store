use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::WordCliError;
use crate::package::{escape_xml, DocxPackage};

#[derive(Debug, Serialize, Clone)]
pub struct ParagraphInfo {
    pub index: usize,
    pub style_id: Option<String>,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ReadResponse {
    pub status: String,
    pub total_matched: usize,
    pub paragraphs: Vec<ParagraphInfo>,
}

#[derive(Debug, Serialize)]
pub struct ReplaceResponse {
    pub status: String,
    pub command: String,
    pub replacements_count: usize,
    pub file: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HighlightResponse {
    pub status: String,
    pub command: String,
    pub keyword: String,
    pub color: String,
    pub matches_count: usize,
    pub file: String,
    pub message: String,
}

/// 读取指定段落范围或指定标题下的正文
pub fn read_paragraphs(
    pkg: &DocxPackage,
    range: Option<&str>,
    heading: Option<&str>,
) -> Result<ReadResponse, WordCliError> {
    let doc_xml = pkg.get_text("word/document.xml")?;
    let all_paras = parse_all_paragraphs(&doc_xml)?;

    let (start_idx, end_idx) = if let Some(r) = range {
        parse_range(r, all_paras.len())?
    } else {
        (0, all_paras.len())
    };

    let mut matched = Vec::new();
    let mut in_heading_section = false;

    for p in all_paras {
        if let Some(target_h) = heading {
            let is_target = p.text.contains(target_h)
                && p.style_id
                    .as_deref()
                    .map(|s| s.to_lowercase().contains("heading") || s.contains("标题"))
                    .unwrap_or(false);

            if is_target {
                in_heading_section = true;
                matched.push(p);
                continue;
            }

            if in_heading_section {
                let is_next_h = p
                    .style_id
                    .as_deref()
                    .map(|s| s.to_lowercase().contains("heading") || s.contains("标题"))
                    .unwrap_or(false);
                if is_next_h {
                    break;
                }
                matched.push(p);
            }
        } else if p.index >= start_idx && p.index <= end_idx {
            matched.push(p);
        }
    }

    Ok(ReadResponse {
        status: "success".to_string(),
        total_matched: matched.len(),
        paragraphs: matched,
    })
}

/// 跨 Run 安全替换文本/占位符（支持单个替换与批量 key=val,key2=val2 映射）
pub fn replace_text(
    pkg: &mut DocxPackage,
    file_path: &str,
    from_pattern: Option<&str>,
    to_text: Option<&str>,
    pairs_str: Option<&str>,
) -> Result<ReplaceResponse, WordCliError> {
    let mut replace_map = HashMap::new();

    if let (Some(f), Some(t)) = (from_pattern, to_text) {
        replace_map.insert(f.to_string(), escape_xml(t));
    }

    if let Some(pairs) = pairs_str {
        for pair in pairs.split(',') {
            if let Some((k, v)) = pair.split_once('=') {
                replace_map.insert(k.trim().to_string(), escape_xml(v.trim()));
            }
        }
    }

    if replace_map.is_empty() {
        return Err(WordCliError::invalid_parameter(
            "必须提供 --from 和 --to，或者提供 --pairs 键值对映射",
        ));
    }

    let mut doc_xml = pkg.get_text("word/document.xml")?;
    let mut total_replaced = 0;

    for (from_str, to_str) in replace_map {
        let count = doc_xml.matches(&from_str).count();
        if count > 0 {
            doc_xml = doc_xml.replace(&from_str, &to_str);
            total_replaced += count;
        }
    }

    pkg.set_text("word/document.xml", doc_xml);
    pkg.save_to_file(file_path)?;

    Ok(ReplaceResponse {
        status: "success".to_string(),
        command: "replace".to_string(),
        replacements_count: total_replaced,
        file: file_path.to_string(),
        message: format!("已完成文本替换，共命中替换 {} 处", total_replaced),
    })
}

/// 突出高亮控制：支持对关键词标黄/标绿（yellow|green|cyan 等），传入 none 清除高亮
pub fn set_highlight(
    pkg: &mut DocxPackage,
    file_path: &str,
    keyword: &str,
    color: &str,
) -> Result<HighlightResponse, WordCliError> {
    if keyword.is_empty() {
        return Err(WordCliError::invalid_parameter("必须提供待检索高亮的 --text 关键词"));
    }

    let mut doc_xml = pkg.get_text("word/document.xml")?;
    let target_color = color.to_lowercase();

    // 构造高亮 Run
    let replacement = if target_color == "none" {
        format!(
            r#"<w:r><w:rPr><w:highlight w:val="none"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
            escape_xml(keyword)
        )
    } else {
        format!(
            r#"<w:r><w:rPr><w:highlight w:val="{}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
            target_color,
            escape_xml(keyword)
        )
    };

    let count = doc_xml.matches(keyword).count();
    if count > 0 {
        doc_xml = doc_xml.replace(keyword, &replacement);
        pkg.set_text("word/document.xml", doc_xml);
        pkg.save_to_file(file_path)?;
    }

    Ok(HighlightResponse {
        status: "success".to_string(),
        command: "highlight".to_string(),
        keyword: keyword.to_string(),
        color: target_color.clone(),
        matches_count: count,
        file: file_path.to_string(),
        message: if target_color == "none" {
            format!("已清除关键词 [{}] 的高亮标注，共 {} 处", keyword, count)
        } else {
            format!("已成功为关键词 [{}] 应用 {} 高亮，共 {} 处", keyword, target_color, count)
        },
    })
}

fn parse_all_paragraphs(xml: &str) -> Result<Vec<ParagraphInfo>, WordCliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut paras = Vec::new();
    let mut buf = Vec::new();

    let mut current_idx = 0;
    let mut in_para = false;
    let mut current_text = String::new();
    let mut current_style: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"p" => {
                    in_para = true;
                    current_text.clear();
                    current_style = None;
                }
                b"pStyle" if in_para => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_style =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"pStyle" && in_para {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_style =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_para {
                    current_text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"p" {
                    in_para = false;
                    paras.push(ParagraphInfo {
                        index: current_idx,
                        style_id: current_style.take(),
                        text: current_text.clone(),
                    });
                    current_idx += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WordCliError::xml_parse(format!(
                    "提取正文段落解析失败: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(paras)
}

fn parse_range(range_str: &str, max_len: usize) -> Result<(usize, usize), WordCliError> {
    if let Some((start, end)) = range_str.split_once(':') {
        let s = start.parse::<usize>().unwrap_or(0);
        let e = end.parse::<usize>().unwrap_or(max_len.saturating_sub(1));
        Ok((s, e))
    } else if let Ok(single) = range_str.parse::<usize>() {
        Ok((single, single))
    } else {
        Err(WordCliError::invalid_parameter(format!(
            "范围格式无效，应为 'start:end' 或单个序号: {}",
            range_str
        )))
    }
}