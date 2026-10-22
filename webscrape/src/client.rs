use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, USER_AGENT};
use std::time::Duration;

use crate::error::WebScrapeError;

/// 构建支持高兼容性 TLS、政企防封锁的通用 HTTP 客户端
pub fn build_http_client(
    timeout_secs: Option<u64>,
    proxy_url: Option<&str>,
    cookie: Option<&str>,
) -> Result<Client, WebScrapeError> {
    let mut headers = HeaderMap::new();

    // 伪装完整浏览器请求头
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

    if let Some(cookie_str) = cookie {
        if let Ok(val) = HeaderValue::from_str(cookie_str) {
            headers.insert(reqwest::header::COOKIE, val);
        }
    }

    let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(30));

    let mut builder = Client::builder()
        .default_headers(headers)
        .timeout(timeout_duration)
        // 关键配置 1：开启最低 TLS 1.2 兼容，解决老旧政企设备握手断开 (EOF)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        // 关键配置 2：允许宽容重定向
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(10));

    // 代理支持
    if let Some(p) = proxy_url {
        let proxy = reqwest::Proxy::all(p).map_err(|e| {
            WebScrapeError::network_error(format!("无效的代理地址 '{}': {}", p, e))
        })?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|e| {
        WebScrapeError::network_error(format!("构建 HTTP 客户端失败: {}", e))
    })
}

/// 发起网络请求获取原始 HTML 文本
pub fn fetch_html(client: &Client, url: &str) -> Result<String, WebScrapeError> {
    let response = client.get(url).send().map_err(|e| {
        WebScrapeError::network_error(format!("网络请求失败 (URL: {}): {}", url, e))
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(WebScrapeError::http_status(
            status.as_u16(),
            format!("请求失败，目标服务器返回状态码 {}", status),
        ));
    }

    let text = response.text().map_err(|e| {
        WebScrapeError::io(format!("解码响应文本失败: {}", e))
    })?;

    Ok(text)
}

/// 发起网络请求获取原始二进制数据（用于图片、PDF、Excel 等文件下载）
pub fn fetch_bytes(client: &Client, url: &str) -> Result<(Vec<u8>, Option<String>), WebScrapeError> {
    let response = client.get(url).send().map_err(|e| {
        WebScrapeError::network_error(format!("二进制资源下载失败 (URL: {}): {}", url, e))
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(WebScrapeError::http_status(
            status.as_u16(),
            format!("下载资源失败，状态码 {}", status),
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let bytes = response.bytes().map_err(|e| {
        WebScrapeError::network_error(format!("读取二进制流失败: {}", e))
    })?;

    Ok((bytes.to_vec(), content_type))
}