use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, ReadHalf, WriteHalf, split},
    net::TcpStream,
    time::{Duration, timeout},
};

use crate::{
    body::HttpBody, error::ServerError, request::HttpRequest, response::HttpResponse,
    router::HttpRouter, utils::find_headers_end, version::HttpVersion,
};

pub struct HttpConnection {
    /// Reader half of the TCP stream
    reader: ReadHalf<TcpStream>,
    /// Writer half of the TCP stream
    writer: WriteHalf<TcpStream>,
    /// Router
    router: Arc<HttpRouter>,
    /// Timeout for each connection
    timeout: Duration,
    /// Buffer size for reading
    buffer_size: usize,
    /// Whether to keep the connection alive
    keep_alive: bool,
}

impl HttpConnection {
    pub fn new(stream: TcpStream, router: HttpRouter, timeout_secs: u64) -> Self {
        // split the stream into reader and writer
        let (reader, writer) = split(stream);

        HttpConnection {
            reader,
            writer,
            router: Arc::new(router),
            timeout: Duration::from_secs(timeout_secs),
            buffer_size: 8192,
            keep_alive: true,
        }
    }

    pub fn keep_alive(&mut self, keep_alive: bool) {
        self.keep_alive = keep_alive;
    }

    pub fn buffer_size(&mut self, size: usize) {
        self.buffer_size = size;
    }

    /// Process the connection
    pub async fn process(&mut self) -> Result<(), ServerError> {
        // keep-alive loop, process multiple requests
        loop {
            let mut buffer = vec![0; self.buffer_size];
            let mut read_bytes_for_headers = 0;

            // read the request headers
            loop {
                match timeout(
                    self.timeout,
                    self.reader.read(&mut buffer[read_bytes_for_headers..]),
                )
                .await
                {
                    // connect closed by peer
                    Ok(Ok(0)) => return Ok(()),
                    Ok(Ok(n)) => {
                        read_bytes_for_headers += n;

                        // if find the headers it is complete
                        if let Some(_pos) = find_headers_end(&buffer[..read_bytes_for_headers]) {
                            break;
                        }

                        if read_bytes_for_headers >= buffer.len() {
                            return Err(ServerError::ProtocolError(
                                "request header was too big".to_string(),
                            ));
                        }
                    }
                    Ok(Err(e)) => return Err(ServerError::IOError(e)),
                    Err(_) => return Err(ServerError::TimeoutError("request timeout".to_string())),
                }
            }

            // process the headers
            let request_str =
                String::from_utf8_lossy(&buffer[..read_bytes_for_headers]).to_string();
            let request = HttpRequest::from(request_str);

            // check if the request is need keep-alive
            let mut connection_keep_alive;
            if request.version == HttpVersion::V1_1 {
                connection_keep_alive = !request
                    .headers
                    .get("Connection")
                    .is_some_and(|h| h.eq_ignore_ascii_case("close"));
            } else {
                connection_keep_alive = request
                    .headers
                    .get("Connection")
                    .is_some_and(|h| h.eq_ignore_ascii_case("keep-alive"));
            }
            if !self.keep_alive {
                connection_keep_alive = false;
            }

            // find the handler
            let handler = self
                .router
                .find_handler(&request.uri.path, request.method)
                .await;
            let mut response = match handler {
                Some(h) => h(request).await,
                None => {
                    // if the handler is not found, return 404
                    let mut response = HttpResponse::new(404, "Not Found");
                    response.headers_mut().insert("Content-Type", "text/plain");
                    response.add_body(HttpBody::from("Not Found"));

                    response
                        .send(&mut self.writer)
                        .await
                        .map_err(ServerError::IOError)?;

                    return Ok(());
                }
            };

            if connection_keep_alive && self.keep_alive {
                response.headers_mut().insert("Connection", "keep-alive");
            } else {
                response.headers_mut().insert("Connection", "close");
            }

            response
                .send(&mut self.writer)
                .await
                .map_err(ServerError::IOError)?;

            if !connection_keep_alive {
                break;
            }
        }

        Ok(())
    }
}
