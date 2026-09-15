use serde::Serialize;

use crate::error::PdfCliError;
use crate::extractor::extract_text;
use crate::inspect::{inspect_pdf, OutlineNode};

#[derive(Debug, Serialize, Clone)]
pub struct ChunkItem {
    pub chunk_id: usize,
    pub heading_path: String,
    pub page_range: [usize; 2],
    pub text: String,
    pub char_count: usize,
    pub token_estimate: usize,
}

#[derive(Debug, Serialize)]
pub struct ChunkResponse {
    pub status: String,
    pub file: String,
    pub strategy: String,
    pub total_chunks: usize,
    pub chunks: Vec<ChunkItem>,
}

/// 执行 RAG 语料切片主函数
pub fn chunk_document(
    file_path: &str,
    strategy: &str,
    chunk_size: usize,
    overlap: usize,
    clean_noise: bool,
) -> Result<ChunkResponse, PdfCliError> {
    let effective_chunk_size = if chunk_size == 0 { 500 } else { chunk_size };
    let effective_overlap = if overlap >= effective_chunk_size {
        effective_chunk_size / 5
    } else {
        overlap
    };

    // 1. 获取文档宏观自省信息（包含总页数与书签树）
    let inspect_res = inspect_pdf(file_path, Some("outline"))?;
    let total_pages = inspect_res.total_pages;

    // 2. 提取整篇文档清理后的文本流（按页组织）
    let text_res = extract_text(file_path, None, Some("all"), None, clean_noise, false, "text")?;
    let page_text_map: std::collections::BTreeMap<usize, String> = text_res
        .pages
        .into_iter()
        .map(|p| (p.page, p.text))
        .collect();

    let mut chunks = Vec::new();

    if strategy.eq_ignore_ascii_case("heading") {
        if let Some(outlines) = inspect_res.outlines {
            if !outlines.is_empty() {
                // 展平大纲树并关联起止页码
                let mut flat_sections = Vec::new();
                flatten_outlines(&outlines, "", &mut flat_sections);

                // 补充每个章节的结束页码
                for i in 0..flat_sections.len() {
                    let next_page = if i + 1 < flat_sections.len() {
                        flat_sections[i + 1].start_page
                    } else {
                        total_pages + 1
                    };
                    flat_sections[i].end_page = if next_page > flat_sections[i].start_page {
                        next_page - 1
                    } else {
                        flat_sections[i].start_page
                    };
                }

                let mut chunk_id = 0;
                for sec in flat_sections {
                    let mut section_text = String::new();
                    for p in sec.start_page..=sec.end_page {
                        if let Some(txt) = page_text_map.get(&p) {
                            if !section_text.is_empty() {
                                section_text.push_str("\n\n");
                            }
                            section_text.push_str(txt);
                        }
                    }

                    let trimmed = section_text.trim();
                    if !trimmed.is_empty() {
                        // 若单个章节超出窗口上限，在章节内再做滑动窗口切分
                        if trimmed.chars().count() > effective_chunk_size {
                            let sub_chunks = split_by_sliding_window(
                                trimmed,
                                effective_chunk_size,
                                effective_overlap,
                            );
                            for sub in sub_chunks {
                                let char_cnt = sub.chars().count();
                                chunks.push(ChunkItem {
                                    chunk_id,
                                    heading_path: sec.path.clone(),
                                    page_range: [sec.start_page, sec.end_page],
                                    text: sub,
                                    char_count: char_cnt,
                                    token_estimate: estimate_tokens(char_cnt),
                                });
                                chunk_id += 1;
                            }
                        } else {
                            let char_cnt = trimmed.chars().count();
                            chunks.push(ChunkItem {
                                chunk_id,
                                heading_path: sec.path.clone(),
                                page_range: [sec.start_page, sec.end_page],
                                text: trimmed.to_string(),
                                char_count: char_cnt,
                                token_estimate: estimate_tokens(char_cnt),
                            });
                            chunk_id += 1;
                        }
                    }
                }
            }
        }
    }

    // 若无书签树或书签模式未生成切片，回退执行全局滑动窗口切分
    if chunks.is_empty() {
        let mut full_doc_text = String::new();
        for p in 1..=total_pages {
            if let Some(txt) = page_text_map.get(&p) {
                if !full_doc_text.is_empty() {
                    full_doc_text.push_str("\n\n");
                }
                full_doc_text.push_str(txt);
            }
        }

        let raw_chunks = split_by_sliding_window(
            full_doc_text.trim(),
            effective_chunk_size,
            effective_overlap,
        );

        for (idx, text) in raw_chunks.into_iter().enumerate() {
            let char_cnt = text.chars().count();
            chunks.push(ChunkItem {
                chunk_id: idx,
                heading_path: "Document / SlidingWindow".to_string(),
                page_range: [1, total_pages],
                text,
                char_count: char_cnt,
                token_estimate: estimate_tokens(char_cnt),
            });
        }
    }

    Ok(ChunkResponse {
        status: "success".to_string(),
        file: file_path.to_string(),
        strategy: strategy.to_string(),
        total_chunks: chunks.len(),
        chunks,
    })
}

#[derive(Debug)]
struct FlatSection {
    path: String,
    start_page: usize,
    end_page: usize,
}

fn flatten_outlines(nodes: &[OutlineNode], prefix: &str, acc: &mut Vec<FlatSection>) {
    for node in nodes {
        let current_path = if prefix.is_empty() {
            node.title.clone()
        } else {
            format!("{} / {}", prefix, node.title)
        };

        if let Some(target) = node.target_page {
            acc.push(FlatSection {
                path: current_path.clone(),
                start_page: target,
                end_page: target,
            });
        }

        if !node.children.is_empty() {
            flatten_outlines(&node.children, &current_path, acc);
        }
    }
}

/// 按固定字符数与重叠长度切分文本（滑动窗口）
fn split_by_sliding_window(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();

    if total_len <= chunk_size {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut start = 0;
    let step = chunk_size.saturating_sub(overlap).max(1);

    while start < total_len {
        let end = (start + chunk_size).min(total_len);
        let slice: String = chars[start..end].iter().collect();
        let trimmed = slice.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }

        if end == total_len {
            break;
        }
        start += step;
    }

    result
}

/// 中英混合 Token 数启发式估算（中文字符约 0.65 token，英文单词约 1.3 token）
fn estimate_tokens(char_count: usize) -> usize {
    ((char_count as f64) * 0.75).round() as usize
}