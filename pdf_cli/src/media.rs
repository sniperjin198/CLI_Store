use flate2::read::ZlibDecoder;
use image::{ImageBuffer, Rgb};
use lopdf::{Document, Object, ObjectId, Stream};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::error::PdfCliError;

#[derive(Debug, Serialize, Clone)]
pub struct ExtractedImageInfo {
    pub page: usize,
    pub image_index: usize,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub output_path: String,
}

#[derive(Debug, Serialize)]
pub struct ExtractImagesResponse {
    pub status: String,
    pub file: String,
    pub output_dir: String,
    pub total_images: usize,
    pub images: Vec<ExtractedImageInfo>,
}

#[derive(Debug, Serialize)]
pub struct RenderPageResponse {
    pub status: String,
    pub file: String,
    pub page: usize,
    pub dpi: u32,
    pub width: u32,
    pub height: u32,
    pub output_file: String,
}

fn safe_decompress_stream(stream: &Stream) -> Option<Vec<u8>> {
    let has_flate = if let Ok(f) = stream.dict.get(b"Filter") {
        match f {
            Object::Name(bytes) => bytes == b"FlateDecode" || bytes == b"Fl",
            Object::Array(arr) => arr.iter().any(|item| {
                if let Object::Name(b) = item {
                    b == b"FlateDecode" || b == b"Fl"
                } else {
                    false
                }
            }),
            _ => false,
        }
    } else {
        false
    };

    if !has_flate {
        return Some(stream.content.clone());
    }

    if let Ok(decomp) = stream.decompressed_content() {
        return Some(decomp);
    }

    let mut decoder = ZlibDecoder::new(&stream.content[..]);
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Some(decompressed);
    }

    None
}

/// 提取 PDF 中内嵌的所有位图图像
pub fn extract_images(
    file_path: &str,
    page_target: Option<usize>,
    output_dir: &str,
) -> Result<ExtractImagesResponse, PdfCliError> {
    let out_path = Path::new(output_dir);
    if !out_path.exists() {
        fs::create_dir_all(out_path).map_err(|e| {
            PdfCliError::io(format!("无法创建输出图片目录 '{}': {}", output_dir, e))
        })?;
    }

    let doc = Document::load(file_path).map_err(|e| {
        PdfCliError::corrupted_document(format!("无法加载 PDF 文档: {}", e))
    })?;

    let pages = doc.get_pages();
    let total_pages = pages.len();

    if let Some(p) = page_target {
        if p == 0 || p > total_pages {
            return Err(PdfCliError::invalid_parameter(format!(
                "指定页码 {} 越界，文档总页数: {}",
                p, total_pages
            )));
        }
    }

    let mut extracted_images = Vec::new();
    let mut exported_streams: HashSet<ObjectId> = HashSet::new();
    let mut global_idx = 0;

    for (&p_num, &page_id) in &pages {
        let current_page = p_num as usize;
        if let Some(target) = page_target {
            if current_page != target {
                continue;
            }
        }

        let img_streams = collect_page_image_streams(&doc, page_id);
        for (stream_id, stream) in img_streams {
            if exported_streams.insert(stream_id) {
                if let Some(info) = dump_single_image(&doc, stream, current_page, global_idx, out_path) {
                    extracted_images.push(info);
                    global_idx += 1;
                }
            }
        }
    }

    if page_target.is_none() {
        for (&id, obj) in &doc.objects {
            if let Object::Stream(stream) = obj {
                if is_image_stream(&doc, stream) && exported_streams.insert(id) {
                    if let Some(info) = dump_single_image(&doc, stream, 1, global_idx, out_path) {
                        extracted_images.push(info);
                        global_idx += 1;
                    }
                }
            }
        }
    }

    Ok(ExtractImagesResponse {
        status: "success".to_string(),
        file: file_path.to_string(),
        output_dir: output_dir.to_string(),
        total_images: extracted_images.len(),
        images: extracted_images,
    })
}

fn is_image_stream(doc: &Document, stream: &Stream) -> bool {
    if let Some(sub_obj) = stream.dict.get(b"Subtype").ok() {
        if let Some(name) = resolve_name(doc, sub_obj) {
            if name.eq_ignore_ascii_case("Image") {
                return true;
            }
        }
    }
    let has_w = stream.dict.get(b"Width").is_ok() || stream.dict.get(b"W").is_ok();
    let has_h = stream.dict.get(b"Height").is_ok() || stream.dict.get(b"H").is_ok();
    has_w && has_h
}

fn resolve_name(doc: &Document, obj: &Object) -> Option<String> {
    match obj {
        Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Reference(id) => {
            if let Ok(target) = doc.get_object(*id) {
                resolve_name(doc, target)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn resolve_int(doc: &Document, obj_opt: Option<&Object>) -> Option<u32> {
    match obj_opt {
        Some(Object::Integer(i)) => Some(*i as u32),
        Some(Object::Real(r)) => Some(*r as u32),
        Some(Object::Reference(id)) => {
            if let Ok(target) = doc.get_object(*id) {
                resolve_int(doc, Some(target))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn collect_page_image_streams<'a>(doc: &'a Document, mut current_page_id: ObjectId) -> Vec<(ObjectId, &'a Stream)> {
    let mut results = Vec::new();
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(current_page_id) {
            break;
        }

        if let Ok(page_dict) = doc.get_dictionary(current_page_id) {
            if let Ok(resources_obj) = page_dict.get(b"Resources") {
                extract_images_from_resources(doc, resources_obj, &mut results, &mut visited);
            }

            if let Ok(parent_ref) = page_dict.get(b"Parent").and_then(Object::as_reference) {
                current_page_id = parent_ref;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    results
}

fn extract_images_from_resources<'a>(
    doc: &'a Document,
    resources_obj: &Object,
    results: &mut Vec<(ObjectId, &'a Stream)>,
    visited: &mut HashSet<ObjectId>,
) {
    let resources = match resources_obj {
        Object::Dictionary(d) => Some(d),
        Object::Reference(r) => doc.get_dictionary(*r).ok(),
        _ => None,
    };

    if let Some(res) = resources {
        if let Ok(xobject_obj) = res.get(b"XObject") {
            let xobjects = match xobject_obj {
                Object::Dictionary(d) => Some(d),
                Object::Reference(r) => doc.get_dictionary(*r).ok(),
                _ => None,
            };

            if let Some(xobj_map) = xobjects {
                for (_name, item_obj) in xobj_map {
                    let stream_id_opt = match item_obj {
                        Object::Reference(id) => Some(*id),
                        _ => None,
                    };
                    if let Some(stream_ref) = stream_id_opt {
                        if !visited.insert(stream_ref) {
                            continue;
                        }
                        if let Ok(stream) = doc.get_object(stream_ref).and_then(Object::as_stream) {
                            if is_image_stream(doc, stream) {
                                results.push((stream_ref, stream));
                            } else if let Ok(sub) = stream.dict.get(b"Subtype").and_then(Object::as_name_str) {
                                if sub.eq_ignore_ascii_case("Form") {
                                    if let Ok(sub_res) = stream.dict.get(b"Resources") {
                                        extract_images_from_resources(doc, sub_res, results, visited);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn resolve_color_table(doc: &Document, stream: &Stream) -> Option<Vec<u8>> {
    let cs_obj = stream.dict.get(b"ColorSpace").ok()?;
    let arr = resolve_to_array(doc, cs_obj)?;

    if arr.len() >= 4 {
        let first_name = match &arr[0] {
            Object::Name(bytes) => String::from_utf8_lossy(bytes).to_string(),
            _ => String::new(),
        };

        if first_name.eq_ignore_ascii_case("Indexed") {
            return extract_raw_palette_bytes(doc, &arr[3]);
        }
    }

    None
}

fn resolve_to_array<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Vec<Object>> {
    match obj {
        Object::Array(arr) => Some(arr),
        Object::Reference(id) => {
            if let Ok(target) = doc.get_object(*id) {
                resolve_to_array(doc, target)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_raw_palette_bytes(doc: &Document, obj: &Object) -> Option<Vec<u8>> {
    match obj {
        Object::String(bytes, _) => Some(bytes.clone()),
        Object::Reference(id) => {
            if let Ok(target) = doc.get_object(*id) {
                extract_raw_palette_bytes(doc, target)
            } else {
                None
            }
        }
        Object::Stream(s) => safe_decompress_stream(s),
        _ => None,
    }
}

fn dump_single_image(
    doc: &Document,
    stream: &Stream,
    page: usize,
    index: usize,
    out_dir: &Path,
) -> Option<ExtractedImageInfo> {
    let width = resolve_int(doc, stream.dict.get(b"Width").ok())
        .or_else(|| resolve_int(doc, stream.dict.get(b"W").ok()))?;
    let height = resolve_int(doc, stream.dict.get(b"Height").ok())
        .or_else(|| resolve_int(doc, stream.dict.get(b"H").ok()))?;

    if width == 0 || height == 0 {
        return None;
    }

    let is_jpeg = stream.content.len() >= 4 && stream.content[0] == 0xFF && stream.content[1] == 0xD8;
    if is_jpeg {
        let file_name = format!("img_p{}_{:03}.jpg", page, index);
        let save_path = out_dir.join(&file_name);
        if fs::write(&save_path, &stream.content).is_ok() {
            return Some(ExtractedImageInfo {
                page,
                image_index: index,
                width,
                height,
                format: "jpg".to_string(),
                output_path: save_path.to_string_lossy().replace('\\', "/"),
            });
        }
    }

    let raw_data = safe_decompress_stream(stream)?;

    if let Some(palette) = resolve_color_table(doc, stream) {
        let mut rgb_buf = ImageBuffer::<Rgb<u8>, _>::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel_idx = (y * width + x) as usize;
                if pixel_idx < raw_data.len() {
                    let color_idx = raw_data[pixel_idx] as usize;
                    let pal_pos = color_idx * 3;
                    if pal_pos + 2 < palette.len() {
                        let r = palette[pal_pos];
                        let g = palette[pal_pos + 1];
                        let b = palette[pal_pos + 2];
                        rgb_buf.put_pixel(x, y, Rgb([r, g, b]));
                    }
                }
            }
        }
        let file_name = format!("img_p{}_{:03}.png", page, index);
        let save_path = out_dir.join(&file_name);
        if rgb_buf.save(&save_path).is_ok() {
            return Some(ExtractedImageInfo {
                page,
                image_index: index,
                width,
                height,
                format: "png".to_string(),
                output_path: save_path.to_string_lossy().replace('\\', "/"),
            });
        }
    }

    let total_pixels = (width * height) as usize;
    if raw_data.len() >= total_pixels * 3 {
        let mut rgb_buf = ImageBuffer::<Rgb<u8>, _>::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                rgb_buf.put_pixel(
                    x,
                    y,
                    Rgb([raw_data[idx], raw_data[idx + 1], raw_data[idx + 2]]),
                );
            }
        }
        let file_name = format!("img_p{}_{:03}.png", page, index);
        let save_path = out_dir.join(&file_name);
        if rgb_buf.save(&save_path).is_ok() {
            return Some(ExtractedImageInfo {
                page,
                image_index: index,
                width,
                height,
                format: "png".to_string(),
                output_path: save_path.to_string_lossy().replace('\\', "/"),
            });
        }
    }

    if raw_data.len() >= total_pixels {
        let mut gray_buf = ImageBuffer::<image::Luma<u8>, _>::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                gray_buf.put_pixel(x, y, image::Luma([raw_data[idx]]));
            }
        }
        let file_name = format!("img_p{}_{:03}.png", page, index);
        let save_path = out_dir.join(&file_name);
        if gray_buf.save(&save_path).is_ok() {
            return Some(ExtractedImageInfo {
                page,
                image_index: index,
                width,
                height,
                format: "png".to_string(),
                output_path: save_path.to_string_lossy().replace('\\', "/"),
            });
        }
    }

    None
}

// -----------------------------------------------------------------------------
// Windows 原生 WinRT PDF 页面渲染
// -----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub fn render_page(
    file_path: &str,
    page_target: usize,
    dpi: u32,
    output_file: &str,
) -> Result<RenderPageResponse, PdfCliError> {

    use std::fs;

    use windows::{
        core::HSTRING,
        Data::Pdf::{
            PdfDocument,
            PdfPageRenderOptions,
        },
        Storage::{
            StorageFile,
        },
        Storage::Streams::{
            DataReader,
            InMemoryRandomAccessStream,
        },
        Win32::System::WinRT::{
            RoInitialize,
            RO_INIT_MULTITHREADED,
        },
    };


    let effective_dpi = if dpi == 0 {
        300
    } else {
        dpi
    };


    unsafe {
        let _ = RoInitialize(
            RO_INIT_MULTITHREADED
        );
    }



    let abs_in =
        fs::canonicalize(file_path)
        .map_err(|e|
            PdfCliError::io(
                format!(
                    "解析PDF路径失败:{}",
                    e
                )
            )
        )?;


    let input_path =
        abs_in
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string();



    let file =
        StorageFile::GetFileFromPathAsync(
            &HSTRING::from(input_path)
        )
        .map_err(|e|
            PdfCliError::io(
                format!(
                    "打开PDF失败:{}",
                    e
                )
            )
        )?
        .get()
        .map_err(|e|
            PdfCliError::io(
                format!(
                    "读取PDF失败:{}",
                    e
                )
            )
        )?;



    let document =
        PdfDocument::LoadFromFileAsync(
            &file
        )
        .map_err(|e|
            PdfCliError::io(
                format!(
                    "加载PDF失败:{}",
                    e
                )
            )
        )?
        .get()
        .map_err(|e|
            PdfCliError::io(
                format!(
                    "解析PDF失败:{}",
                    e
                )
            )
        )?;



    let page_count =
        document.PageCount()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )? as usize;



    if page_target == 0 ||
       page_target > page_count {

        return Err(
            PdfCliError::invalid_parameter(
                format!(
                    "页码 {} 超出范围，总页数 {}",
                    page_target,
                    page_count
                )
            )
        );
    }



    let page =
        document.GetPage(
            (page_target - 1) as u32
        )
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let size =
        page.Size()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let width =
        (
            size.Width *
            effective_dpi as f32 /
            72.0
        )
        .round() as u32;



    let options =
        PdfPageRenderOptions::new()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    options
        .SetDestinationWidth(width)
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let stream =
        InMemoryRandomAccessStream::new()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    page.RenderWithOptionsToStreamAsync(
        &stream,
        &options
    )
    .map_err(|e|
        PdfCliError::io(
            e.to_string()
        )
    )?
    .get()
    .map_err(|e|
        PdfCliError::io(
            format!(
                "页面渲染失败:{}",
                e
            )
        )
    )?;



    let size =
        stream.Size()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let input =
        stream
        .GetInputStreamAt(0)
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let reader =
        DataReader::CreateDataReader(
            &input
        )
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    reader
        .LoadAsync(size as u32)
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?
        .get()
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    let mut buffer =
        vec![0u8; size as usize];



    reader
        .ReadBytes(
            &mut buffer
        )
        .map_err(|e|
            PdfCliError::io(
                e.to_string()
            )
        )?;



    fs::write(
        output_file,
        &buffer
    )
    .map_err(|e|
        PdfCliError::io(
            format!(
                "保存PNG失败:{}",
                e
            )
        )
    )?;



    let height =
        (
            size as f64 /
            width as f64
        )
        .round() as u32;



    Ok(
        RenderPageResponse {

            status:
                "success"
                .to_string(),

            file:
                file_path
                .to_string(),

            page:
                page_target,

            dpi:
                effective_dpi,

            width,

            height,

            output_file:
                output_file
                .replace("\\","/"),
        }
    )
}



#[cfg(not(target_os = "windows"))]
pub fn render_page(
    _file_path: &str,
    _page_target: usize,
    _dpi: u32,
    _output_file: &str,
)
-> Result<RenderPageResponse, PdfCliError>
{
    Err(
        PdfCliError::unsupported_feature(
            "页面渲染仅支持Windows"
        )
    )
}