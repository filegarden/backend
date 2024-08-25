//! All services and their request handlers.

use axum::{
    extract::Request,
    http::{header::HOST, StatusCode},
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use once_cell::sync::Lazy;

use crate::{api, content, website, CONTENT_ORIGIN, WEBSITE_ORIGIN};

/// The URI host for user-uploaded content.
static CONTENT_HOST: Lazy<&str> = Lazy::new(|| host_from_origin(&CONTENT_ORIGIN));

/// The URI host for the website.
static WEBSITE_HOST: Lazy<&str> = Lazy::new(|| host_from_origin(&WEBSITE_ORIGIN));

/// Handles all incoming requests and routes them to other services based on the request URI.
#[debug_handler]
pub(super) async fn handle(request: Request) -> Response {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|host| host.to_str().ok());

    if host == Some(*CONTENT_HOST) {
        return content::handle(request).into_response();
    }

    if host == Some(*WEBSITE_HOST) {
        if request.uri().path().starts_with("/api/") {
            return api::handle(request).await.into_response();
        }

        return website::handle(request).into_response();
    }

    StatusCode::MISDIRECTED_REQUEST.into_response()
}

/// Returns the host from an origin URI string.
///
/// # Panics
///
/// Panics if the origin string doesn't contain "//".
fn host_from_origin(origin: &str) -> &str {
    let host_index = origin.find("//").expect("origin should contain \"//\"") + 2;

    origin
        .get(host_index..)
        .expect("origin should be sliceable at the index after \"//\"")
}
