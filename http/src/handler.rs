use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub type HandlerFn = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
