use crate::{
    body::HttpBody, headers::HttpHeaders, method::HttpMethod, uri::HttpUri, version::HttpVersion,
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
}
