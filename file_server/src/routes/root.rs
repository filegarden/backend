//! Route handlers for `/`.

use axum::response::{IntoResponse, Redirect};
use axum_macros::debug_handler;

use crate::WEBSITE_URI;

/// Route handler for `GET /`.
#[debug_handler]
pub(crate) async fn get() -> impl IntoResponse {
    Redirect::permanent(WEBSITE_URI)
}
