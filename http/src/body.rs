use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Default)]
pub enum HttpBody {
    /// complete data in memory
    InMemory { data: Vec<u8> },
    /// streaming data
    Streaming {
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
        read_buf: Vec<u8>,
        buffer_size: usize,
    },
    /// empty body
    #[default]
    Empty,
}

impl HttpBody {
    pub fn new() -> Self {
        HttpBody::Empty
    }

    pub fn from_data(data: Vec<u8>) -> Self {
        match data.is_empty() {
            true => HttpBody::Empty,
            false => HttpBody::InMemory { data },
        }
    }

    pub fn from_reader<R>(reader: R, buffer_size: usize) -> Self
    where
        R: AsyncRead + Send + Sync + 'static,
    {
        HttpBody::Streaming {
            reader: Box::pin(reader),
            read_buf: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    pub async fn read_next(&mut self) -> tokio::io::Result<Option<Vec<u8>>> {
        match self {
            HttpBody::InMemory { data } => {
                if data.is_empty() {
                    Ok(None)
                } else {
                    let res = Ok(Some(data.clone()));
                    data.clear();
                    res
                }
            }
            HttpBody::Streaming {
                reader,
                read_buf,
                buffer_size,
            } => {
                read_buf.clear();
                read_buf.resize(*buffer_size, 0);
                match AsyncReadExt::read(reader, read_buf).await {
                    // end of stream
                    Ok(0) => Ok(None),
                    Ok(n) => {
                        read_buf.truncate(n);
                        Ok(Some(read_buf.clone()))
                    }
                    Err(e) => Err(e),
                }
            }
            HttpBody::Empty => Ok(None),
        }
    }

    pub fn is_streaming(&self) -> bool {
        matches!(self, HttpBody::Streaming { .. })
    }

    pub fn content_length(&self) -> Option<usize> {
        match self {
            HttpBody::InMemory { data } => Some(data.len()),
            HttpBody::Streaming { .. } => None,
            HttpBody::Empty => None,
        }
    }
}

impl std::fmt::Debug for HttpBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpBody::InMemory { data } => f.debug_struct("InMemory").field("data", data).finish(),
            HttpBody::Streaming {
                read_buf,
                buffer_size,
                ..
            } => f
                .debug_struct("Streaming")
                .field("read_buf", read_buf)
                .field("buffer_size", buffer_size)
                .field("reader", &"<dyn AsyncRead>")
                .finish(),
            HttpBody::Empty => write!(f, "Empty"),
        }
    }
}

impl From<&str> for HttpBody {
    fn from(value: &str) -> Self {
        HttpBody::InMemory {
            data: value.as_bytes().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_empty_body() {
        let mut body = HttpBody::new();

        assert_eq!(body.read_next().await.unwrap(), None);
        assert_eq!(body.content_length(), None);
        assert!(!body.is_streaming());
    }

    #[test]
    async fn test_in_memory_body() {
        let data = "Hello world!".as_bytes().to_vec();
        let expected_len = data.len();
        let mut body = HttpBody::from_data(data);

        // length
        assert_eq!(body.content_length(), Some(expected_len));

        // read content
        let content = body.read_next().await.unwrap().unwrap();
        assert_eq!(String::from_utf8_lossy(&content), "Hello world!");

        // have read all data
        assert_eq!(body.read_next().await.unwrap(), None);

        // not streaming
        assert!(!body.is_streaming());
    }

    #[test]
    async fn test_streaming_body() {
        use std::io::Cursor;

        let data = "First chunk\nSecond chunk\nOther chunk".as_bytes().to_vec();
        let cursor = Cursor::new(data.clone());

        // some buffer size to ensure read few times
        let mut body = HttpBody::from_reader(cursor, 8);

        // streaming
        assert!(body.is_streaming());
        assert_eq!(body.content_length(), None);

        let mut all_chunks = Vec::new();
        while let Some(chunk) = body.read_next().await.unwrap() {
            all_chunks.extend_from_slice(&chunk);
            // println!("{:?}", chunk);
        }

        assert_eq!(
            String::from_utf8_lossy(&all_chunks),
            String::from_utf8_lossy(&data)
        );
    }
}
