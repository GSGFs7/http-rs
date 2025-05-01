use std::sync::Arc;

use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    time::{Duration, timeout},
};

use crate::{
    body::HttpBody, error::ServerError, request::HttpRequest, response::HttpResponse,
    router::HttpRouter, utils::find_headers_end,
};

pub struct HttpConnection {
    /// TCP stream
    stream: TcpStream,
    router: Arc<HttpRouter>,
    timeout: Duration,
    buffer_size: usize,
    keep_alive: bool,
}

impl HttpConnection {
    pub fn new(stream: TcpStream, router: HttpRouter, timeout_secs: u64) -> Self {
        HttpConnection {
            stream,
            router: Arc::new(router),
            timeout: Duration::from_secs(timeout_secs),
            buffer_size: 8192,
            keep_alive: false,
        }
    }

    pub fn keep_alive(&mut self, keep_alive: bool) -> &mut Self {
        self.keep_alive = keep_alive;
        self
    }

    pub fn buffer_size(&mut self, size: usize) -> &mut Self {
        self.buffer_size = size;
        self
    }

    pub async fn process(&mut self) -> Result<(), ServerError> {
        let mut buffer = vec![0; self.buffer_size];
        let mut read_bytes = 0;

        let _headers_end = loop {
            match timeout(self.timeout, self.stream.read(&mut buffer[read_bytes..])).await {
                Ok(Ok(0)) => return Ok(()),
                Ok(Ok(n)) => {
                    read_bytes += n;

                    if let Some(pos) = find_headers_end(&buffer[..read_bytes]) {
                        break pos;
                    }

                    if read_bytes >= buffer.len() {
                        return Err(ServerError::ProtocolError(
                            "request headers was too big".to_string(),
                        ));
                    }
                }
                Ok(Err(e)) => return Err(ServerError::IOError(e)),
                Err(_) => return Err(ServerError::TimeoutError("request timeout".to_string())),
            }
        };

        let request_str = String::from_utf8_lossy(&buffer[..read_bytes]).to_string();
        let request = HttpRequest::from(request_str);

        let keep_alive = matches!(request.headers.get("Connection"), Some(c) if c.to_lowercase() == "keep-alive");

        let handler = match self
            .router
            .find_handler(&request.uri.path, request.method)
            .await
        {
            Some(h) => h,
            None => {
                // Not Found
                let mut response = HttpResponse::new(404, "Not Found");
                response.headers_mut().insert("Content-Type", "text/plain");
                response.add_body(HttpBody::from("Not Found"));

                if keep_alive {
                    response.headers_mut().insert("Connection", "keep-alive");
                }

                response
                    .send(&mut self.stream)
                    .await
                    .map_err(ServerError::IOError)?;

                return Ok(());
            }
        };

        let mut response = handler(request);

        if keep_alive {
            response.headers_mut().insert("Connection", "keep-alive");
        }

        response
            .send(&mut self.stream)
            .await
            .map_err(ServerError::IOError)?;

        Ok(())
    }
}

