/// this file is a simple example of a web server using the http crate
///
/// it's so hard to use, i think...
use std::sync::Arc;

use http::{
    body::HttpBody, method::HttpMethod, response::HttpResponse, router::HttpRouter,
    server::HttpServer,
};
use tokio::fs;

// define a test handler
async fn test(_req: http::request::HttpRequest) -> HttpResponse {
    let test = fs::read("./src/www/html/test.html").await.unwrap();
    HttpResponse::new(200, "OK").with_body(HttpBody::from(&test))
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
        .await;

    let mut server = HttpServer::new();
    server.set_router(&router);
    let _ = server.run().await;
}
