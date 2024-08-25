//! A web server for the website. File Garden exposes this via `https://filegarden.com/`.

use axum::{extract::Request, response::IntoResponse};

/// The service function to handle incoming requests for the HTTP API.
pub(super) fn handle(request: Request) -> impl IntoResponse {
    drop(request);

    "TODO"
}
