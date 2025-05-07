/// this file is a simple example of a web server using the http crate
///
/// it's so hard to use, i think...
use std::sync::Arc;

use http::{
    body::HttpBody, method::HttpMethod, response::HttpResponse, router::HttpRouter,
    server::HttpServer,
};
use tokio::fs;

#[tokio::main]
async fn main() {
    let home = fs::read("./src/www/html/home.html").await.unwrap();
    let test = fs::read("./src/www/html/test.html").await.unwrap();

    let router = HttpRouter::new()
        .add(
            HttpMethod::Get,
            "/",
            Arc::new(move |_| HttpResponse::new(200, "OK").with_body(HttpBody::from(&home))),
        )
        .await
        .add(
            HttpMethod::Get,
            "/test",
            Arc::new(move |_| HttpResponse::new(200, "OK").with_body(HttpBody::from(&test))),
        )
        .await;

    let mut server = HttpServer::new();
    server.set_router(&router);
    let _ = server.run().await;
}
