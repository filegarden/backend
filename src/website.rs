//! A web server for the website. File Garden exposes this via `https://filegarden.com/`.

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};

/// The service function to handle incoming requests for the HTTP API.
pub(super) fn handle(request: Request) -> Response {
    let (request, _body) = request.into_parts();

    if request.method != Method::GET {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    "TODO".into_response()
}
