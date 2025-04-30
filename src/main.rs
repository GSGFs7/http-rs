use http::{
    body::HttpBody, method::HttpMethod, response::HttpResponse, router::HttpRouter,
    server::HttpServer,
};

#[tokio::main]
async fn main() {
    let mut binding = HttpRouter::new();
    let router = binding
        .add(HttpMethod::Get, "/", |_| {
            HttpResponse::new(200, "OK").with_body(HttpBody::from("Hello world!"))
        })
        .await;

    let mut server = HttpServer::new();
    server.set_router(router);
    let _ = server.run().await;
}
