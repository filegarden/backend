//! A web server for the HTTP API. File Garden exposes this via `https://filegarden.com/api/`.

use std::sync::LazyLock;

use axum::{extract::Request, response::IntoResponse, Router};
use tower::ServiceExt;

/// The API router.
static ROUTER: LazyLock<Router> = LazyLock::new(Router::new);

/// Routes a request to an API endpoint.
#[allow(clippy::missing_errors_doc)] // The error here is `Infallible`.
pub(super) async fn handle(request: Request) -> impl IntoResponse {
    // Calling the router needs a mutable reference to it (even though it shouldn't), so the router
    // must either have restricted access via a mutex or be cloned on each request. The former would
    // allow only one request at a time, so the latter is faster.
    ROUTER.clone().oneshot(request).await
}
