use std::collections::HashMap;

/// CMap 映射表：负责字符编码到 Unicode 字符串的映射
#[derive(Debug, Default, Clone)]
pub struct CMapTable {
    map: HashMap<u32, String>,
}

impl CMapTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// 从 PDF 的 ToUnicode 原始流文本中解析映射关系
    pub fn parse_to_unicode(stream_content: &str) -> Self {
        let mut table = Self::new();
        let lines: Vec<&str> = stream_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // 解析 beginbfchar 区块 (单字映射)
            if line.ends_with("beginbfchar") {
                i += 1;
                while i < lines.len() {
                    let sub_line = lines[i].trim();
                    if sub_line.ends_with("endbfchar") {
                        break;
                    }
                    table.parse_bfchar_line(sub_line);
                    i += 1;
                }
            }
            // 解析 beginbfrange 区块 (区间映射)
            else if line.ends_with("beginbfrange") {
                i += 1;
                while i < lines.len() {
                    let sub_line = lines[i].trim();
                    if sub_line.ends_with("endbfrange") {
                        break;
                    }
                    table.parse_bfrange_line(sub_line);
                    i += 1;
                }
            }
            i += 1;
        }

        table
    }

    /// 解析形如 `<0001> <0041>` 的单字映射行
    fn parse_bfchar_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Some(code), Some(unicode)) = (parse_hex(parts[0]), parse_unicode_hex(parts[1])) {
                self.map.insert(code, unicode);
            }
        }
    }

    /// 解析区间映射行，支持两种标准格式：
    /// 1. `<0001> <0003> <0041>` (递增序列)
    /// 2. `<0001> <0002> [<0041> <0042>]` (显式数组)
    fn parse_bfrange_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Some(start), Some(end)) = (parse_hex(parts[0]), parse_hex(parts[1])) {
                if parts[2].starts_with('[') {
                    // 数组格式
                    let array_content = line.split('[').nth(1).and_then(|s| s.split(']').next()).unwrap_or("");
                    let targets: Vec<String> = array_content
                        .split_whitespace()
                        .filter_map(parse_unicode_hex)
                        .collect();
                    for (offset, code) in (start..=end).enumerate() {
                        if let Some(target) = targets.get(offset) {
                            self.map.insert(code, target.clone());
                        }
                    }
                } else if let Some(dest_start) = parse_hex(parts[2]) {
                    // 递增序列格式
                    for (offset, code) in (start..=end).enumerate() {
                        let target_code = dest_start + offset as u32;
                        if let Some(c) = char::from_u32(target_code) {
                            self.map.insert(code, c.to_string());
                        }
                    }
                }
            }
        }
    }

    /// 将字节流解码为 UTF-8 字符串，内建两级回退机制：
    /// 1. CMap 优先匹配；
    /// 2. 启发式 UTF-16BE 解码；
    /// 3. UTF-8 / ASCII 无损清洗。
    pub fn decode_bytes(&self, bytes: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;

        // 策略 A: 若包含双字节 CMap，优先按 2 字节滑动窗口查表
        if !self.map.is_empty() {
            while i < bytes.len() {
                if i + 1 < bytes.len() {
                    let code2 = ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32);
                    if let Some(s) = self.map.get(&code2) {
                        result.push_str(s);
                        i += 2;
                        continue;
                    }
                }

                let code1 = bytes[i] as u32;
                if let Some(s) = self.map.get(&code1) {
                    result.push_str(s);
                    i += 1;
                    continue;
                }

                // 未匹配到，按单字节 fallback
                if bytes[i].is_ascii() && !bytes[i].is_ascii_control() {
                    result.push(bytes[i] as char);
                } else {
                    result.push(char::REPLACEMENT_CHARACTER);
                }
                i += 1;
            }
            return result;
        }

        // 策略 B: CMap 缺失时的通用回退（常见于直接内嵌 UTF-16BE 的文档）
        if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
            // 带 BOM 的 UTF-16BE
            let u16_vec: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
                .collect();
            return String::from_utf16_lossy(&u16_vec);
        }

        // 策略 C: 标准 UTF-8 或无损 ASCII
        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => s,
            Err(_) => {
                // 启发式：如果偶数长度且高字节频繁为 0，按无 BOM 的 UTF-16BE 尝试
                if bytes.len() % 2 == 0 && bytes.iter().step_by(2).filter(|&&b| b == 0).count() > bytes.len() / 4 {
                    let u16_vec: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
                        .collect();
                    String::from_utf16_lossy(&u16_vec)
                } else {
                    // 纯 Latin1 / ASCII 回退
                    bytes.iter().map(|&b| if b < 128 { b as char } else { '?' }).collect()
                }
            }
        }
    }
}

/// 解析十六进制字符串 `<0041>` 为 u32
fn parse_hex(s: &str) -> Option<u32> {
    let clean = s.trim().trim_start_matches('<').trim_end_matches('>');
    u32::from_str_radix(clean, 16).ok()
}

/// 解析目标十六进制为 Unicode 字符串（支持多字符代理项）
fn parse_unicode_hex(s: &str) -> Option<String> {
    let clean = s.trim().trim_start_matches('<').trim_end_matches('>');
    if clean.len() % 4 == 0 {
        let mut u16_vec = Vec::new();
        for i in (0..clean.len()).step_by(4) {
            if let Ok(val) = u16::from_str_radix(&clean[i..i + 4], 16) {
                u16_vec.push(val);
            }
        }
        if !u16_vec.is_empty() {
            return Some(String::from_utf16_lossy(&u16_vec));
        }
    } else if let Ok(val) = u32::from_str_radix(clean, 16) {
        if let Some(c) = char::from_u32(val) {
            return Some(c.to_string());
        }
    }
    None
}