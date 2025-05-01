/// find the last position of the header
pub fn find_headers_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
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
