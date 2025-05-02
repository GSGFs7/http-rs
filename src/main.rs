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

    let mut binding = HttpRouter::new();
    let router = binding
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
    server.set_router(router);
    let _ = server.run().await;
}
