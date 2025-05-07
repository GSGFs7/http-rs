#[derive(Debug)]
pub enum ServerError {
    /// IO error
    IOError(std::io::Error),
    /// Parse error
    ParseError(String),
    /// Protocol error
    ProtocolError(String),
    /// Connection error
    InternalError(String),
    /// Configuration error
    ConfigError(String),
    /// Timeout error
    TimeoutError(String),
}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        ServerError::IOError(value)
    }
}
