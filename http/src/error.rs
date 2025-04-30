#[derive(Debug)]
pub enum ServerError {
    IOError(std::io::Error),
    ParseError(String),
    ProtocolError(String),
    InternalError(String),
    ConfigError(String),
    TimeoutError(String),
}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        ServerError::IOError(value)
    }
}
