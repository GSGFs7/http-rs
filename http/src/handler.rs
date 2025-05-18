use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::request::HttpRequest;
use crate::response::HttpResponse;

/// Handler function type
///
/// This type supports both closures and normal functions.
///
/// If using closures, the function must be `async` and return a
/// `Pin<Box<dyn Future<Output = HttpResponse>>>`. For example:
///
/// # Examples
///
/// ```rust
/// use std::future::Future;
/// use std::pin::Pin;
/// use std::sync::Arc;
/// use http::request::HttpRequest;
/// use http::response::HttpResponse;
///
/// type HandlerFn = Arc<
///     dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>
///         + Send
///         + Sync
///         + 'static,
/// >;
///
/// async fn example_handler(req: HttpRequest) -> HttpResponse {
///     // Process the request and return a response
///     HttpResponse::new(200, "OK")
///         .with_body("Hello, world!".into())
/// }
///
/// let handler: HandlerFn = Arc::new(move |req: HttpRequest| {
///     Box::pin(example_handler(req))
/// });
/// ```
pub type HandlerFn = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
