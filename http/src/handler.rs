use std::sync::Arc;

use crate::{request::HttpRequest, response::HttpResponse};

// This supports closures, but it complex for writing
pub type HandlerFn = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>;

// This not support closures
// pub type HandlerFn = fn(request: HttpRequest) -> HttpResponse;
