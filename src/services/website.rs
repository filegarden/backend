//! A web server to proxy the website. File Garden exposes this through `https://filegarden.com/`.

use axum::extract::Request;
use axum_macros::debug_handler;

use crate::response::Response;

/// The service function to handle incoming requests for the website.
#[allow(clippy::unused_async)] // Axum route handlers must be async.
#[debug_handler]
pub(super) async fn handler(request: Request) -> Response {
    _ = request;

    Response::new().body("TODO")
}
