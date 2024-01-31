//! Route handlers for routes to files.

use axum::response::IntoResponse;
use axum_macros::debug_handler;

/// Route handler for `GET` on routes to files.
#[debug_handler]
pub(crate) async fn get() -> impl IntoResponse {
    "TODO"
}
