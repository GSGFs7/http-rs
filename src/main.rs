/// this file is a simple example of a web server using the http crate
///
/// it's so hard to use, i think...
use std::sync::Arc;

use http::{
    body::HttpBody, method::HttpMethod, request::HttpRequest, response::HttpResponse,
    router::HttpRouter, server::HttpServer,
};
use tokio::fs::{self, File};

// define a test handler
async fn test(_req: http::request::HttpRequest) -> HttpResponse {
    let test = fs::read("./src/www/html/test.html").await.unwrap();
    HttpResponse::new(200, "OK").with_body(HttpBody::from(&test))
}

// large file handler
async fn stream_large_file_handler(_req: HttpRequest) -> HttpResponse {
    match File::open("./src/www/test_file.bin").await {
        Ok(file) => {
            let mut response = HttpResponse::new(200, "OK").with_streaming_body(file, 8192);
            response.insert_header("Transfer-Encoding", "chunked");
            response
        }
        Err(e) => {
            eprintln!("{e}");
            HttpResponse::new(404, "Not Found").with_body("File not found".into())
        }
    }
}

#[tokio::main]
async fn main() {
    let router = HttpRouter::new()
        .add(
            HttpMethod::Get,
            "/",
            Arc::new(|_| {
                Box::pin(async move {
                    let home = fs::read("./src/www/html/home.html").await.unwrap();
                    HttpResponse::new(200, "OK").with_body(HttpBody::from(&home))
                })
            }),
        )
        .await
        // a simple way to add a handler
        .get("/test", test)
        .await
        .get("/test_file.bin", stream_large_file_handler)
        .await;

    let mut server = HttpServer::new();
    server.set_router(&router);
    let _ = server.run().await;
}
