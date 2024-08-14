//! A web server for the HTTP API. File Garden exposes this through `https://filegarden.com/api/`.

use axum::extract::Request;
use axum_macros::debug_handler;

use crate::response::Response;

/// The service function to handle incoming requests for the HTTP API.
#[allow(clippy::unused_async)] // Axum route handlers must be async.
#[debug_handler]
pub(super) async fn handler(request: Request) -> Response {
    _ = request;

    Response::new().body("TODO")
}
