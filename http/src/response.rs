use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{body::HttpBody, headers::HttpHeaders, version::HttpVersion};

#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP status code
    status_code: u16,
    /// HTTP status text
    status_text: String,
    /// HTTP headers
    headers: HttpHeaders,
    /// HTTP body
    body: HttpBody,
    /// HTTP version
    version: HttpVersion,
    /// Whether the response uses chunked encoding
    chunked_encoding: bool,
}

impl HttpResponse {
    pub fn new(status_code: u16, status_text: &str) -> Self {
        HttpResponse {
            status_code,
            status_text: status_text.to_string(),
            headers: HttpHeaders::new(),
            body: HttpBody::new(),
            version: HttpVersion::V1_1,
            chunked_encoding: false,
        }
    }

    pub fn with_body(mut self, body: HttpBody) -> Self {
        self.body = body;
        self
    }

    pub fn add_body(&mut self, body: HttpBody) -> &mut Self {
        self.body = body;
        self
    }

    pub fn handlers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    pub fn body(&self) -> &HttpBody {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut HttpBody {
        &mut self.body
    }

    pub fn with_streaming_body<R>(mut self, reader: R, buffer_size: usize) -> Self
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        self.body = HttpBody::from_reader(reader, buffer_size);

        self.chunked_encoding = true;
        self.headers.insert("Transfer-Encoding", "chunked");

        self
    }

    async fn write_headers<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let header = format!(
            "{} {} {}\r\n",
            self.version.as_string(),
            self.status_code,
            self.status_text
        );
        writer.write_all(header.as_bytes()).await?;

        // if not chunked, add content length
        if !self.chunked_encoding {
            if let Some(length) = self.body.content_length() {
                if !self.headers.contains_key("Content-Length") {
                    let header = format!("Content-Length: {length}\r\n");
                    writer.write_all(header.as_bytes()).await?;
                }
            }
        }

        for (key, value) in self.headers.iter() {
            let header_line = format!("{key}: {value}\r\n");
            writer.write_all(header_line.as_bytes()).await?;
        }

        // end of headers
        writer.write_all(b"\r\n").await?;

        Ok(())
    }

    /// send response
    pub async fn send<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.write_headers(writer).await?;

        match self.chunked_encoding {
            true => self.send_chunked(writer).await?,
            false => self.send_normal(writer).await?,
        }

        Ok(())
    }

    async fn send_normal<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        while let Some(chunk) = self.body.read_next().await? {
            writer.write_all(&chunk).await?;
        }
        writer.flush().await?;

        Ok(())
    }

    async fn send_chunked<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        while let Some(chunk) = self.body.read_next().await? {
            if !chunk.is_empty() {
                let content = format!("{:X}\r\n", chunk.len());
                writer.write_all(content.as_bytes()).await?;
                writer.write_all(&chunk).await?;
                writer.write_all(b"\r\n").await?;
            }
        }

        writer.write_all(b"0\r\n").await?;
        writer.write_all(b"\r\n").await?;
        writer.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::task::Poll;

    use super::*;
    use tokio::test;

    #[test]
    async fn test_basic_response() {
        let mut response = HttpResponse::new(200, "OK").with_body(HttpBody::from("Hello, World!"));
        response.headers.insert("Content-Type", "text/plain");

        let mut buffer = Vec::new();
        response.send(&mut buffer).await.unwrap();

        let response_str = String::from_utf8_lossy(&buffer);
        assert!(response_str.contains("HTTP/1.1 200 OK"));
        assert!(response_str.contains("Content-Type: text/plain"));
        assert!(response_str.contains("Content-Length: 13"));
        assert!(response_str.contains("Hello, World!"));

        assert!(!response_str.contains("Transfer-Encoding"))
    }

    #[test]
    async fn test_empty_request() {
        let mut response = HttpResponse::new(204, "No Content");

        let mut buffer = Vec::new();
        response.send(&mut buffer).await.unwrap();

        let response_str = String::from_utf8_lossy(&buffer);
        assert!(response_str.contains("HTTP/1.1 204 No Content"));
        assert!(!response_str.contains("Content-Length"));
    }

    #[test]
    async fn test_chunked_encoding() {
        struct TestReader {
            chunk: Vec<Vec<u8>>,
            current: usize,
        }

        impl AsyncRead for TestReader {
            fn poll_read(
                mut self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
                buf: &mut io::ReadBuf<'_>,
            ) -> Poll<io::Result<()>> {
                if self.current >= self.chunk.len() {
                    return Poll::Ready(Ok(()));
                }

                let chunk = &self.chunk[self.current];
                buf.put_slice(chunk);
                self.current += 1;

                Poll::Ready(Ok(()))
            }
        }

        impl Unpin for TestReader {}

        let chunks = vec![
            b"First chunk".to_vec(),
            b"Second chunk".to_vec(),
            b"Other chunk".to_vec(),
        ];

        let reader = TestReader {
            chunk: chunks.clone(),
            current: 0,
        };

        let mut response = HttpResponse::new(200, "OK").with_streaming_body(reader, 1024);
        response.headers.insert("Content-Type", "test/plain");

        let mut buffer = Vec::new();
        response.send(&mut buffer).await.unwrap();

        let response_str = String::from_utf8_lossy(&buffer);
        assert!(response_str.contains("Transfer-Encoding: chunked"));

        print!("{response_str}");

        for chunk in &chunks {
            let chunk_size = format!("{:X}", chunk.len());
            assert!(response_str.contains(&chunk_size));
            assert!(response_str.contains(std::str::from_utf8(chunk).unwrap()));
        }

        assert!(response_str.contains("0\r\n\r\n"));
    }
}
