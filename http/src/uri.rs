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

    pub fn as_string(&self) -> String {
        self.path.clone()
    }
}

impl From<&str> for HttpUri {
    fn from(value: &str) -> Self {
        HttpUri {
            path: value.to_string(),
        }
    }
}

impl From<String> for HttpUri {
    fn from(value: String) -> Self {
        HttpUri { path: value }
    }
}
