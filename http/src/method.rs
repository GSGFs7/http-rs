#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    NoSupport,
}

impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => HttpMethod::NoSupport,
        }
    }
}

impl From<String> for HttpMethod {
    fn from(value: String) -> Self {
        HttpMethod::from(value.as_str())
    }
}

impl HttpMethod {
    /// 'method' can be anything which can into `HttpMethod`
    pub fn is_support<T: Into<HttpMethod>>(method: T) -> bool {
        let m: HttpMethod = method.into();
        m != HttpMethod::NoSupport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into() {
        assert_eq!(HttpMethod::from("GET"), HttpMethod::Get);
        assert_eq!(HttpMethod::from("POST"), HttpMethod::Post);
        assert_eq!(HttpMethod::from("some str"), HttpMethod::NoSupport);

        assert_eq!(HttpMethod::from("GET".to_string()), HttpMethod::Get);
    }

    #[test]
    fn test_method_is_support() {
        assert!(HttpMethod::is_support("GET"));
        assert!(HttpMethod::is_support("POST"));
        assert!(!HttpMethod::is_support("some str"));
        assert!(HttpMethod::is_support(HttpMethod::Get));
        assert!(HttpMethod::is_support(HttpMethod::Post));
        assert!(!HttpMethod::is_support(HttpMethod::NoSupport));
    }
}
