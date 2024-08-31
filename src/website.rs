//! A web server for the website. File Garden exposes this via `https://filegarden.com/`.

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};

/// The service function to handle incoming requests for the HTTP API.
pub(super) fn handle(request: Request) -> Response {
    drop(request);

    "TODO".into_response()
}
