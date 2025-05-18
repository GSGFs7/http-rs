#[derive(Debug, PartialEq, Eq, Default)]
/// HTTP version
pub enum HttpVersion {
    V1_0,
    #[default]
    V1_1,
    NoSupport,
}

impl HttpVersion {
    pub fn as_string(&self) -> String {
        match self {
            HttpVersion::V1_0 => "HTTP/1.0".to_string(),
            HttpVersion::V1_1 => "HTTP/1.1".to_string(),
            HttpVersion::NoSupport => "NoSupport".to_string(),
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, HttpVersion::NoSupport)
    }
}

impl From<&str> for HttpVersion {
    fn from(value: &str) -> Self {
        match value {
            "HTTP/1.0" => HttpVersion::V1_0,
            "HTTP/1.1" => HttpVersion::V1_1,
            _ => HttpVersion::NoSupport,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_http_version() {
        assert_eq!(HttpVersion::from("HTTP/1145"), HttpVersion::NoSupport);
        assert_eq!(HttpVersion::from("HTTP/1.0"), HttpVersion::V1_0);
        assert_eq!(HttpVersion::from("HTTP/1.1"), HttpVersion::V1_1);
        assert_eq!(HttpVersion::from("HTTP/1.1").as_string(), "HTTP/1.1");
    }
}
