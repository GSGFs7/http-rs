use http::{feature::file_server_handler, server};

#[tokio::main]
async fn main() {
    // TODO: Support wildcards
    let router = http::router::HttpRouter::new()
        .get("/*", file_server_handler)
        .await;

    let mut server = server::HttpServer::new();
    server.set_router(&router);

    server.run().await.unwrap();
}
