use std::sync::Arc;

use tokio::{
    net::TcpListener,
    spawn,
    sync::Semaphore,
    time::{self, Duration},
};

use crate::{connect::HttpConnection, error::ServerError, router::HttpRouter};

const MAX_CONNECTIONS: usize = 1000;
const CONNECTION_TIMEOUT: usize = 5;

#[derive(Clone)]
pub struct ServerConfig {
    pub address: String,
    pub router: Arc<HttpRouter>,
    pub timeout: usize,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_string(),
            router: Arc::new(HttpRouter::new()),
            timeout: CONNECTION_TIMEOUT,
            max_connections: MAX_CONNECTIONS,
        }
    }
}

#[derive(Default)]
pub struct HttpServer {
    pub config: ServerConfig,
}

impl HttpServer {
    pub fn new() -> Self {
        HttpServer {
            config: ServerConfig::default(),
        }
    }

    pub fn with_config(config: ServerConfig) -> Self {
        HttpServer { config }
    }

    pub fn set_config(&mut self, config: ServerConfig) -> &mut Self {
        self.config = config;
        self
    }

    pub fn set_router(&mut self, router: &HttpRouter) -> &mut Self {
        self.config.router = Arc::new(router.clone());
        self
    }

    pub fn set_port(&mut self, port: &str) -> &mut Self {
        self.config.address = port.to_string();
        self
    }

    pub async fn run(&self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(&self.config.address).await?;

        println!("Server running http://{}", self.config.address);

        let semaphore = Arc::new(Semaphore::new(self.config.max_connections));

        loop {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    eprintln!("Get permit failed: {}", e);
                    time::sleep(Duration::from_secs(self.config.timeout as u64)).await;
                    continue;
                }
            };

            let (socket, addr) = match listener.accept().await {
                Ok(connection) => connection,
                Err(e) => {
                    eprintln!("Accept connection failed: {}", e);
                    continue;
                }
            };

            println!("New connection from {}", addr);

            let mut connection = HttpConnection::new(
                socket,
                (*self.config.router).clone(),
                self.config.timeout as u64,
            );

            spawn(async move {
                let _permit = permit;

                if let Err(e) = connection.process().await {
                    eprintln!("Connection error from {}: {:?}", addr, e);
                };
            });
        }
    }
}
