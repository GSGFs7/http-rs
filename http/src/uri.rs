#[derive(Debug, Default)]
pub struct HttpUri {
    /// HTTP URI path
    pub path: String,
}

impl HttpUri {
    pub fn new() -> Self {
        HttpUri {
            path: String::new(),
        }
    }
}

impl From<&str> for HttpUri {
    fn from(value: &str) -> Self {
        HttpUri {
            path: value.to_string(),
        }
    }
}
