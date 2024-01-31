//! Route handlers for `/`.

use axum::response::{IntoResponse, Redirect};

use crate::WEBSITE_URI;

/// Route handler for `GET /`.
pub(crate) async fn get() -> impl IntoResponse {
    Redirect::permanent(WEBSITE_URI)
}
