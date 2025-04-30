use crate::{request::HttpRequest, response::HttpResponse};

// This support closures, but it complex for writing
// pub type HandlerFn = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>;

pub type HandlerFn = fn(request: HttpRequest) -> HttpResponse;
