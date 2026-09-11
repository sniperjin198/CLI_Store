use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;

use crate::error::WordCliError;
use crate::package::DocxPackage;

#[derive(Debug, Serialize, Clone)]
pub struct StyleItem {
    pub id: String,
    pub name: String,
    pub style_type: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
pub struct StylesResponse {
    pub status: String,
    pub total: usize,
    pub styles: Vec<StyleItem>,
}

pub fn list_styles(pkg: &DocxPackage) -> Result<StylesResponse, WordCliError> {
    let xml_content = pkg.get_text("word/styles.xml").map_err(|_| {
        WordCliError::not_found("文档中未找到 word/styles.xml 样式定义文件")
    })?;

    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);

    let mut styles = Vec::new();
    let mut buf = Vec::new();

    let mut current_id: Option<String> = None;
    let mut current_type: Option<String> = None;
    let mut current_default = false;
    let mut current_name: Option<String> = None;
    let mut in_style = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"style" => {
                    in_style = true;
                    current_id = None;
                    current_type = None;
                    current_default = false;
                    current_name = None;

                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"styleId" => {
                                current_id = Some(
                                    String::from_utf8_lossy(&attr.value).to_string(),
                                );
                            }
                            b"type" => {
                                current_type = Some(
                                    String::from_utf8_lossy(&attr.value).to_string(),
                                );
                            }
                            b"default" => {
                                current_default = attr.value.as_ref() == b"1"
                                    || attr.value.as_ref() == b"true";
                            }
                            _ => {}
                        }
                    }
                }
                b"name" if in_style => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_name = Some(
                                String::from_utf8_lossy(&attr.value).to_string(),
                            );
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"name" if in_style => {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            current_name = Some(
                                String::from_utf8_lossy(&attr.value).to_string(),
                            );
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"style" {
                    in_style = false;
                    if let Some(id) = current_id.take() {
                        let name = current_name.take().unwrap_or_else(|| id.clone());
                        let style_type = current_type.take().unwrap_or_else(|| "paragraph".to_string());
                        styles.push(StyleItem {
                            id,
                            name,
                            style_type,
                            is_default: current_default,
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(WordCliError::xml_parse(format!(
                    "解析 styles.xml 失败: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(StylesResponse {
        status: "success".to_string(),
        total: styles.len(),
        styles,
    })
}