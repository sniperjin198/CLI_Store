use flate2::write::ZlibEncoder;
use flate2::Compression;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use rusttype::{Font, Scale};
use serde::Serialize;
use std::io::Write;

use crate::error::PdfCliError;
use crate::inspect::obj_to_f64;

#[derive(Debug, Serialize)]
pub struct WatermarkResponse {
    pub status: String,
    pub action: String,
    pub output_file: String,
    pub watermarked_pages: usize,
    pub text: String,
}

fn load_system_font() -> Option<Font<'static>> {
    let font_candidates = [
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simkai.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    for path in &font_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            if let Some(font) = Font::try_from_vec(bytes) {
                return Some(font);
            }
        }
    }
    None
}

fn compress_bytes(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(data);
    encoder.finish().unwrap_or_else(|_| data.to_vec())
}

/// 将中文字符渲染为高精度透明通道印章
fn create_watermark_image(
    text: &str,
    font_size_pt: f32,
) -> Result<(u32, u32, Vec<u8>, Vec<u8>), PdfCliError> {
    let font = load_system_font().ok_or_else(|| {
        PdfCliError::not_found("未找到 Windows 系统可用中文字体 (simhei.ttf/msyh.ttc)")
    })?;

    // 采用 2.5 倍超采样保证边缘平滑锐利
    let render_scale = font_size_pt * 2.5;
    let scale = Scale::uniform(render_scale);

    let char_count = text.chars().count() as f32;
    let buf_w = ((char_count * render_scale * 1.15) as u32).max(120) + 80;
    let buf_h = ((render_scale * 1.8) as u32).max(80) + 60;

    let mut img = ImageBuffer::<Rgba<u8>, _>::from_pixel(buf_w, buf_h, Rgba([0, 0, 0, 0]));

    // 深灰印章色
    let text_color = Rgba([90, 90, 90, 255]);
    draw_text_mut(&mut img, text_color, 25, 20, scale, &font, text);

    // 紧凑裁剪文字有效范围
    let mut min_x = buf_w;
    let mut max_x = 0;
    let mut min_y = buf_h;
    let mut max_y = 0;

    for y in 0..buf_h {
        for x in 0..buf_w {
            if img.get_pixel(x, y)[3] > 0 {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        min_x = 0;
        max_x = buf_w - 1;
        min_y = 0;
        max_y = buf_h - 1;
    }

    let crop_w = (max_x - min_x + 1).max(1);
    let crop_h = (max_y - min_y + 1).max(1);

    let mut rgb_bytes = Vec::with_capacity((crop_w * crop_h * 3) as usize);
    let mut alpha_bytes = Vec::with_capacity((crop_w * crop_h) as usize);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = img.get_pixel(x, y);
            rgb_bytes.push(p[0]);
            rgb_bytes.push(p[1]);
            rgb_bytes.push(p[2]);
            alpha_bytes.push(p[3]);
        }
    }

    Ok((crop_w, crop_h, rgb_bytes, alpha_bytes))
}

pub fn add_watermark(
    file_path: &str,
    text: &str,
    opacity: f64,
    font_size: f64,
    rotation: f64,
    output_file: Option<&str>,
) -> Result<WatermarkResponse, PdfCliError> {
    if text.trim().is_empty() {
        return Err(PdfCliError::invalid_parameter("水印文本内容不能为空"));
    }

    let clamped_opacity = opacity.clamp(0.05, 1.0);
    let effective_size = if font_size <= 0.0 { 40.0 } else { font_size };

    let mut doc = Document::load(file_path).map_err(|e| {
        PdfCliError::corrupted_document(format!("无法加载 PDF 文档 '{}': {}", file_path, e))
    })?;

    // 1. 生成带 Alpha 遮罩的水印图像
    let (crop_w, crop_h, rgb_raw, alpha_raw) = create_watermark_image(text, effective_size as f32)?;
    let comp_rgb = compress_bytes(&rgb_raw);
    let comp_alpha = compress_bytes(&alpha_raw);

    // 2. 构造 Soft Mask (透明通道)
    let mut smask_dict = Dictionary::new();
    smask_dict.set("Type", "XObject");
    smask_dict.set("Subtype", "Image");
    smask_dict.set("Width", crop_w);
    smask_dict.set("Height", crop_h);
    smask_dict.set("ColorSpace", "DeviceGray");
    smask_dict.set("BitsPerComponent", 8);
    smask_dict.set("Filter", "FlateDecode");
    let smask_id = doc.add_object(Object::Stream(Stream::new(smask_dict, comp_alpha)));

    // 3. 构造 RGB 主图像并挂载 SMask
    let mut img_dict = Dictionary::new();
    img_dict.set("Type", "XObject");
    img_dict.set("Subtype", "Image");
    img_dict.set("Width", crop_w);
    img_dict.set("Height", crop_h);
    img_dict.set("ColorSpace", "DeviceRGB");
    img_dict.set("BitsPerComponent", 8);
    img_dict.set("Filter", "FlateDecode");
    img_dict.set("SMask", Object::Reference(smask_id));
    let img_id = doc.add_object(Object::Stream(Stream::new(img_dict, comp_rgb)));

    // 4. 构造透明度图形状态 ExtGState
    let mut gs_dict = Dictionary::new();
    gs_dict.set("Type", "ExtGState");
    gs_dict.set("CA", clamped_opacity);
    gs_dict.set("ca", clamped_opacity);
    let gs_id = doc.add_object(Object::Dictionary(gs_dict));

    // 计算在 PDF 坐标系下的物理点尺寸
    let w_pt = crop_w as f64 / 2.5;
    let h_pt = crop_h as f64 / 2.5;

    let rad = rotation.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();

    let a = w_pt * cos;
    let b = w_pt * sin;
    let c = -h_pt * sin;
    let d = h_pt * cos;

    let pages = doc.get_pages();
    let mut modified_count = 0;

    for (&_page_num, &page_id) in &pages {
        let (width, height) = get_page_dimensions(&doc, page_id);
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        // 居中映射变换
        let e = center_x - 0.5 * (a + c);
        let f = center_y - 0.5 * (b + d);

        let watermark_ops = vec![
            Operation::new("q", vec![]),
            Operation::new("gs", vec![Object::Name(b"GS_WM".to_vec())]),
            Operation::new("cm", vec![
                a.into(),
                b.into(),
                c.into(),
                d.into(),
                e.into(),
                f.into(),
            ]),
            Operation::new("Do", vec![Object::Name(b"Im_WM".to_vec())]),
            Operation::new("Q", vec![]),
        ];

        let content = Content { operations: watermark_ops };
        let encoded_stream = Stream::new(Dictionary::new(), content.encode().unwrap_or_default());
        let stream_id = doc.add_object(Object::Stream(encoded_stream));

        // 安全挂载资源并追加内容流
        if attach_resources_safe(&mut doc, page_id, gs_id, img_id).is_ok() {
            append_content_stream(&mut doc, page_id, stream_id);
            modified_count += 1;
        }
    }

    let save_target = output_file.unwrap_or(file_path);
    doc.save(save_target).map_err(|e| {
        PdfCliError::io(format!("保存水印 PDF 失败 '{}': {}", save_target, e))
    })?;

    Ok(WatermarkResponse {
        status: "success".to_string(),
        action: "watermark".to_string(),
        output_file: save_target.replace('\\', "/"),
        watermarked_pages: modified_count,
        text: text.to_string(),
    })
}

fn get_page_dimensions(doc: &Document, page_id: ObjectId) -> (f64, f64) {
    let mut current_id = page_id;
    while let Ok(page_dict) = doc.get_dictionary(current_id) {
        if let Ok(arr) = page_dict.get(b"MediaBox").and_then(Object::as_array) {
            if arr.len() >= 4 {
                let x0 = obj_to_f64(&arr[0]).unwrap_or(0.0);
                let y0 = obj_to_f64(&arr[1]).unwrap_or(0.0);
                let x1 = obj_to_f64(&arr[2]).unwrap_or(595.28);
                let y1 = obj_to_f64(&arr[3]).unwrap_or(841.89);
                return ((x1 - x0).abs(), (y1 - y0).abs());
            }
        }
        if let Ok(parent_ref) = page_dict.get(b"Parent").and_then(Object::as_reference) {
            current_id = parent_ref;
        } else {
            break;
        }
    }
    (595.28, 841.89)
}

fn attach_resources_safe(
    doc: &mut Document,
    page_id: ObjectId,
    gs_id: ObjectId,
    img_id: ObjectId,
) -> Result<(), PdfCliError> {
    let page_dict = doc.get_dictionary_mut(page_id).map_err(|e| {
        PdfCliError::corrupted_document(format!("无法获取页面字典: {}", e))
    })?;

    let res_ref = match page_dict.get(b"Resources") {
        Ok(Object::Reference(id)) => Some(*id),
        _ => None,
    };

    if let Some(r_id) = res_ref {
        let res_dict = doc.get_dictionary_mut(r_id).map_err(|e| {
            PdfCliError::corrupted_document(format!("无法获取引用的资源字典: {}", e))
        })?;
        update_res_dict(res_dict, gs_id, img_id);
    } else {
        let page_dict = doc.get_dictionary_mut(page_id).unwrap();
        if !page_dict.has(b"Resources") {
            page_dict.set("Resources", Dictionary::new());
        }
        if let Ok(Object::Dictionary(ref mut res_dict)) = page_dict.get_mut(b"Resources") {
            update_res_dict(res_dict, gs_id, img_id);
        }
    }

    Ok(())
}

fn update_res_dict(res_dict: &mut Dictionary, gs_id: ObjectId, img_id: ObjectId) {
    match res_dict.get_mut(b"ExtGState") {
        Ok(Object::Dictionary(ref mut d)) => {
            d.set("GS_WM", Object::Reference(gs_id));
        }
        _ => {
            let mut d = Dictionary::new();
            d.set("GS_WM", Object::Reference(gs_id));
            res_dict.set("ExtGState", Object::Dictionary(d));
        }
    }

    match res_dict.get_mut(b"XObject") {
        Ok(Object::Dictionary(ref mut d)) => {
            d.set("Im_WM", Object::Reference(img_id));
        }
        _ => {
            let mut d = Dictionary::new();
            d.set("Im_WM", Object::Reference(img_id));
            res_dict.set("XObject", Object::Dictionary(d));
        }
    }
}

fn append_content_stream(doc: &mut Document, page_id: ObjectId, stream_id: ObjectId) {
    if let Ok(page_dict) = doc.get_dictionary_mut(page_id) {
        match page_dict.get_mut(b"Contents") {
            Ok(Object::Array(ref mut arr)) => {
                arr.push(Object::Reference(stream_id));
            }
            Ok(Object::Reference(old_id)) => {
                let old_ref = *old_id;
                page_dict.set(
                    "Contents",
                    Object::Array(vec![Object::Reference(old_ref), Object::Reference(stream_id)]),
                );
            }
            _ => {
                page_dict.set("Contents", Object::Reference(stream_id));
            }
        }
    }
}