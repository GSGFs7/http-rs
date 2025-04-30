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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into() {
        assert_eq!(HttpMethod::from("GET"), HttpMethod::Get);
        assert_eq!(HttpMethod::from("some str"), HttpMethod::NoSupport);
    }
}
