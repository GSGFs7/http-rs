use std::path::Path;

/// find the last position of the header
///
/// if found, return the position of the end of the header  
/// if not found, return None
pub fn find_headers_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4) // use windows to find the last position of the header
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

pub fn get_content_type(path: &str) -> &'static str {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::test;

    #[test]
    async fn test_find_headers_end() {
        let headers = b"POST / HTTP/1.1\r\nHost: gsgfs.moe\r\n\r\nsome body data";
        assert_eq!(find_headers_end(headers), Some(36));
    }
}
