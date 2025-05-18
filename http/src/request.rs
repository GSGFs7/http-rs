use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    body::HttpBody, error::ServerError, headers::HttpHeaders, method::HttpMethod, uri::HttpUri,
    utils, version::HttpVersion,
};

#[derive(Debug)]
pub struct HttpRequest {
    /// HTTP method
    pub method: HttpMethod,
    /// HTTP headers
    pub headers: HttpHeaders,
    /// HTTP body
    pub body: Option<HttpBody>,
    /// HTTP URI
    pub uri: HttpUri,
    /// HTTP version
    pub version: HttpVersion,
}

impl From<String> for HttpRequest {
    fn from(value: String) -> Self {
        HttpRequest::from(value.as_str())
    }
}

/// Parse HTTP request from a string
impl From<&str> for HttpRequest {
    fn from(value: &str) -> Self {
        let mut parsed_method = HttpMethod::NoSupport;
        let mut parsed_uri = HttpUri::new();
        let mut parsed_version = HttpVersion::V1_1;
        let mut parsed_headers = HttpHeaders::new();

        let lines: Vec<&str> = value.lines().collect();
        let len = lines.len();

        // request line
        if len >= 1 {
            let parts: Vec<&str> = lines[0].split_whitespace().collect();
            if parts.len() == 3 {
                parsed_method = HttpMethod::from(parts[0]);
                parsed_uri = HttpUri::from(parts[1]);
                parsed_version = HttpVersion::from(parts[2]);
            }
        }

        // headers
        let mut header_lines = 0;
        if len >= 2 {
            for line in lines[1..].iter() {
                if line.is_empty() {
                    break;
                }

                header_lines += 1;
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    parsed_headers.insert(parts[0].trim(), parts[1].trim());
                }
            }
        }

        // body
        let mut data = Vec::new();
        if lines.len() > 1 + header_lines + 1 {
            for line in lines[(1 + header_lines + 1)..].iter() {
                data.extend_from_slice(line.as_bytes());
            }
        }
        let parsed_body = HttpBody::from_data(data);

        HttpRequest {
            method: parsed_method,
            headers: parsed_headers,
            body: Some(parsed_body),
            uri: parsed_uri,
            version: parsed_version,
        }
    }
}

impl HttpRequest {
    pub async fn from_stream<S: AsyncRead + Unpin + Send + Sync + 'static>(
        mut stream: S,
    ) -> Result<Self, ServerError> {
        let mut buffer = Vec::new();
        // read to headers end
        let headers_end = loop {
            let mut buf = [0u8; 1024]; // buffer size
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Err(ServerError::ProtocolError(
                    "Unexpected EOF while reading headers".into(),
                ));
            }
            buffer.extend_from_slice(&buf[..n]);

            if let Some(pos) = utils::find_headers_end(&buffer) {
                break pos;
            }
        };

        // parse headers
        let headers_str = std::str::from_utf8(&buffer[..headers_end]).map_err(|e| {
            ServerError::ProtocolError(format!("Decode headers to UTF-8 error: {e}"))
        })?;
        let (method, uri, version, headers) = Self::parse_headers(headers_str)?;

        // body
        let remaining_stream = Pin::new(Box::new(stream)); // Wrapped as AsyncRead stream
        let body = {
            let pre_read = if buffer.len() > headers_end {
                buffer[headers_end..].to_vec()
            } else {
                Vec::new()
            };

            if !pre_read.is_empty()
                || headers.contains_key("Content-Length")
                || headers.contains_key("Transfer-Encoding")
            {
                HttpBody::Streaming {
                    reader: remaining_stream,
                    read_buf: pre_read,
                    buffer_size: 1024,
                }
            } else {
                HttpBody::Empty
            }
        };

        Ok(HttpRequest {
            method,
            headers,
            body: Some(body),
            uri,
            version,
        })
    }

    fn parse_headers(
        headers_str: &str,
    ) -> Result<(HttpMethod, HttpUri, HttpVersion, HttpHeaders), ServerError> {
        let lines: Vec<&str> = headers_str.lines().collect();
        if lines.is_empty() {
            return Err(ServerError::ProtocolError("Empty request line".into()));
        }

        // first line
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() != 3 {
            return Err(ServerError::ProtocolError(
                "Invalid request line format".into(),
            ));
        }
        let method = HttpMethod::from(request_line_parts[0]);
        let uri = HttpUri::from(request_line_parts[1]);
        let version = HttpVersion::from(request_line_parts[2]);

        // headers
        let mut headers = HttpHeaders::new();
        for line in lines.iter().skip(1) {
            // if find "\r\n\r\n"
            if line.is_empty() {
                break;
            }

            // Split the line into key and value
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                headers.insert(parts[0].trim(), parts[1].trim());
            }
        }

        Ok((method, uri, version, headers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_request_parse_from_string() {
        let request_string = "GET /index.html HTTP/1.1\r\n\
                             Host: example.com\r\n\
                             User-Agent: Mozilla/5.0 Firefox/114\r\n\
                             Accept: text/html\r\n\
                             \r\n";

        let request = HttpRequest::from(request_string.to_string());

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.uri.path, "/index.html");
        assert_eq!(request.version, HttpVersion::V1_1);
        assert_eq!(request.headers.get("Host").unwrap(), "example.com");
        assert_eq!(
            request.headers.get("User-Agent").unwrap(),
            "Mozilla/5.0 Firefox/114"
        );
        assert_eq!(request.headers.get("Accept").unwrap(), "text/html");
        match &request.body.unwrap() {
            HttpBody::Empty => (),
            HttpBody::InMemory { data } => assert!(data.is_empty()),
            HttpBody::Streaming { .. } => panic!("Expected InMemory or Empty body, got Streaming"),
        }
    }

    #[test]
    async fn test_empty_request() {
        let request_string = "GET / HTTP/1.1";

        let request = HttpRequest::from(request_string);
        let mut body = request.body.unwrap();
        assert_eq!(body.content_length(), None);
        assert_eq!(body.read_next().await.unwrap(), None);
    }

    struct ChunkedStream {
        data: Vec<Vec<u8>>,
        pos: usize,
    }

    impl AsyncRead for ChunkedStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            if self.pos >= self.data.len() {
                return std::task::Poll::Ready(Ok(()));
            }

            let chunk = &self.data[self.pos];
            buf.put_slice(chunk);
            self.pos += 1;
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[test]
    async fn test_streaming_request_parse() {
        let request_headers = b"POST /post HTTP/1.1\r\n\
                                Host: gsgfs.moe\r\n\
                                Content-Length: 13\r\n\
                                Transfer-Encoding: chunked\r\n\
                                \r\n";
        let chunk1 = b"5\r\nHello\r\n";
        let chunk2 = b"5\r\nWorld\r\n";
        let chunk_end = b"0\r\n\r\n";

        let stream = ChunkedStream {
            data: vec![
                request_headers.to_vec(),
                chunk1.to_vec(),
                chunk2.to_vec(),
                chunk_end.to_vec(),
            ],
            pos: 0,
        };

        let request = HttpRequest::from_stream(stream).await.unwrap();

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.uri.path, "/post");
        assert_eq!(request.version, HttpVersion::V1_1);
        assert_eq!(request.headers.get("Host").unwrap(), "gsgfs.moe");
        assert_eq!(request.headers.get("Content-Length").unwrap(), "13");
        assert_eq!(request.headers.get("Transfer-Encoding").unwrap(), "chunked");

        if let Some(http_body) = request.body {
            if let HttpBody::Streaming {
                mut read_buf,
                buffer_size,
                mut reader,
            } = http_body
            {
                let mut content = Vec::new();
                content.append(&mut read_buf);

                reader.read_to_end(&mut content).await.unwrap();

                let expected_body_bytes = b"5\r\nHello\r\n5\r\nWorld\r\n0\r\n\r\n";
                assert_eq!(content, expected_body_bytes);
                assert_eq!(buffer_size, 1024);
            } else {
                panic!("Expected Streaming body, got '{http_body:?}'");
            }
        } else {
            panic!("Request body was None");
        }
    }
}
