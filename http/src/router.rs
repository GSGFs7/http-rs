use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{handler::HandlerFn, method::HttpMethod};

type ParamRoute = Option<(String, Box<RouteNode>)>;
type StaticRoutes = HashMap<String, Arc<RouteNode>>;
type Handlers = HashMap<HttpMethod, Arc<HandlerFn>>;
type Middlewares = Vec<HandlerFn>;

/// Similar with Trie tree
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct RouteNode {
    /// name of current node
    name: String,
    /// The processing method mapping of the current node
    handlers: Arc<RwLock<Handlers>>,
    /// Static subpaths, such as /users, /posts
    static_routes: Arc<RwLock<StaticRoutes>>,
    /// Parameter subpath, such as /:id, /:username
    param_route: Arc<RwLock<ParamRoute>>,
    /// Wildcard handlers, such as /* or /files/*
    wildcard_handler: Arc<RwLock<Option<HandlerFn>>>,
    /// Node-level middleware
    middlewares: Arc<RwLock<Middlewares>>,
}

// tokio::sync::RwLock not have `Clone` impl
// impl Clone for RouteNode {
//     fn clone(&self) -> Self {
//         let handlers_clone = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(async { self.handlers.read().await.clone() });

//         let static_routes_clone = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(async { self.static_routes.read().await.clone() });

//         let param_route_clone = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(async { self.param_route.read().await.clone() });

//         let wildcard_handler_clone = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(async { self.wildcard_handler.read().await.clone() });

//         let middlewares_clone = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(async { self.middlewares.read().await.clone() });

//         RouteNode {
//             name: self.name.clone(),
//             handlers: RwLock::new(handlers_clone),
//             static_routes: RwLock::new(static_routes_clone),
//             param_route: RwLock::new(param_route_clone),
//             wildcard_handler: RwLock::new(wildcard_handler_clone),
//             middlewares: RwLock::new(middlewares_clone),
//         }
//     }
// }

impl RouteNode {
    pub fn new() -> Self {
        RouteNode {
            name: "/".to_string(), // root router
            handlers: Arc::new(RwLock::new(HashMap::new())),
            static_routes: Arc::new(RwLock::new(HashMap::new())),
            param_route: Arc::new(RwLock::new(None)),
            wildcard_handler: Arc::new(RwLock::new(None)),
            middlewares: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_name(name: &str) -> Self {
        RouteNode {
            name: name.to_string(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            static_routes: Arc::new(RwLock::new(HashMap::new())),
            param_route: Arc::new(RwLock::new(None)),
            wildcard_handler: Arc::new(RwLock::new(None)),
            middlewares: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[must_use]
#[derive(Debug, Clone)]
pub struct HttpRouter {
    root: Arc<RouteNode>,
    global_middlewares: Vec<HandlerFn>,
}

impl Default for HttpRouter {
    fn default() -> Self {
        HttpRouter {
            root: Arc::new(RouteNode::new()),
            global_middlewares: Vec::new(),
        }
    }
}

impl HttpRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// add a router
    pub async fn add(&mut self, method: HttpMethod, path: &str, handler: HandlerFn) -> &mut Self {
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut current = Arc::clone(&self.root);

        // Move on the tree to find the node
        for segment in segments {
            if segment.is_empty() {
                continue;
            }

            let node_ref = Arc::clone(&current);

            if segment.starts_with(':') {
                todo!();
            } else if segment == "*" {
                todo!();
            } else {
                // if not found the node, create it
                if !node_ref.static_routes.read().await.contains_key(segment) {
                    let new_node = Arc::new(RouteNode::with_name(segment));
                    node_ref
                        .static_routes
                        .write()
                        .await
                        .insert(segment.to_string(), new_node);
                }
                current = Arc::clone(node_ref.static_routes.read().await.get(segment).unwrap());
            }
        }

        // add handler
        Arc::clone(&current)
            .handlers
            .write()
            .await
            .insert(method, Arc::new(handler));

        self
    }

    pub async fn find_handler(&self, path: &str, method: HttpMethod) -> Option<HandlerFn> {
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut current = Arc::clone(&self.root);

        for segment in segments {
            if segment.is_empty() {
                continue;
            }

            let node_ref = Arc::clone(&current);
            if !node_ref.static_routes.read().await.contains_key(segment) {
                let new_node = Arc::new(RouteNode::with_name(segment));
                node_ref
                    .static_routes
                    .write()
                    .await
                    .insert(segment.to_string(), new_node);
            }
            let route = node_ref.static_routes.read().await;
            let route = match route.get(segment) {
                Some(route) => route,
                None => return None,
            };
            current = Arc::clone(route);
        }

        current
            .handlers
            .read()
            .await
            .get(&method)
            .map(|handler| **handler)
    }

    pub fn add_global_middleware(&mut self, handler: HandlerFn) -> &mut Self {
        self.global_middlewares.push(handler);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{request::HttpRequest, response::HttpResponse};
    use tokio::test;

    #[test]
    async fn test_add_router_to_root() {
        let mut router = HttpRouter::new();

        router
            .add(HttpMethod::Get, "/", |_req| {
                HttpResponse::new(200, "OK").with_body(crate::body::HttpBody::InMemory {
                    data: b"Hello world".to_vec(),
                })
            })
            .await;

        println!("{:#?}", router);

        let root_handlers = router.root.handlers.read().await;
        assert!(root_handlers.contains_key(&HttpMethod::Get));

        let f = root_handlers.get(&HttpMethod::Get).unwrap();
        let mut response = f(HttpRequest::from("GET / HTTP/1.1".to_string()));
        let body = response.body.read_next().await.unwrap().unwrap();
        assert_eq!(body, b"Hello world".to_vec());
    }

    #[test]
    async fn test_long_path_router() {}
}
