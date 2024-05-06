//! See [`handler`].

use std::borrow::Cow;

use axum::{
    extract::Request,
    http::{
        header::{ACCESS_CONTROL_ALLOW_ORIGIN, ALLOW, CONTENT_SECURITY_POLICY},
        Method, StatusCode,
    },
};
use axum_macros::debug_handler;
use percent_encoding::{percent_decode_str, utf8_percent_encode};

use crate::{percent_encoding::COMPONENT_IGNORING_SLASH, response::Response, WEBSITE_URI};

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// The [`Content-Security-Policy`](https://developer.mozilla.org/docs/Web/HTTP/CSP) header's
/// value for all requests.
const CSP: &str =
    "default-src file.garden linkh.at data: mediastream: blob: 'unsafe-inline' 'unsafe-eval'";

/// The service function to handle incoming requests.
#[allow(clippy::unused_async)] // Axum route handlers must be async.
#[debug_handler]
pub(super) async fn handler(request: Request) -> Response {
    let mut response = Response::new();

    response
        .header_valid(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header_valid(CONTENT_SECURITY_POLICY, CSP);

    let method = request.method();

    if !(method == Method::GET || method == Method::HEAD) {
        let status = if method == Method::OPTIONS {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::METHOD_NOT_ALLOWED
        };

        response
            .status(status)
            .header_valid(ALLOW, "GET, HEAD, OPTIONS");

        return response;
    }

    let uri = request.uri();
    let initial_path = uri.path();

    if initial_path == "/" {
        return response.permanent_redirect(WEBSITE_URI);
    }

    let Ok(path) = percent_decode_str(initial_path).decode_utf8() else {
        return response.plain_error(StatusCode::BAD_REQUEST);
    };

    // The above can decode `%00` into a null byte, so disallow null bytes as a defensive measure.
    if path.contains('\x00') {
        return response.plain_error(StatusCode::BAD_REQUEST);
    }

    let normalized_encoded_path: Cow<str> =
        utf8_percent_encode(&path, COMPONENT_IGNORING_SLASH).into();

    let query = uri.query();

    if initial_path != normalized_encoded_path {
        // Redirect to the same URI with normalized path encoding. This reduces how many URLs must
        // be purged from the CDN's cache when a file changes. It's impossible to purge every
        // possible variation of encoding for a URL.

        let normalized_uri = concat_path_and_query(&normalized_encoded_path, query);

        return response.permanent_redirect(&normalized_uri);
    }

    let (user_identifier, file_path) = parse_file_route_path(&path);
    let file_id = get_queried_file_id(query);

    // response
    //     .header_valid(CONTENT_LENGTH, 0)
    //     .header_valid(CONTENT_TYPE, "")
    //     .header_valid(LAST_MODIFIED, "");

    if method == Method::HEAD {
        return response;
    }

    response.body(format!(
        "{user_identifier} - {file_path} - {}",
        file_id.unwrap_or("None")
    ))
}

/// Joins a path and a query into one string, separated by a `?` if there exists a query.
fn concat_path_and_query<'a>(path: &'a str, query: Option<&'a str>) -> Cow<'a, str> {
    let mut path_and_query = Cow::from(path);

    if let Some(query) = query {
        path_and_query.to_mut().push('?');
        path_and_query.to_mut().push_str(query);
    }

    path_and_query
}

/// Extracts a tuple of the user identifier and file path from the path of a file route URI.
fn parse_file_route_path(path: &str) -> (&str, &str) {
    let path = path.strip_prefix('/').expect("path should start with `/`");

    match path.find('/') {
        Some(slash_index) => path.split_at(slash_index),
        None => (path, "/"),
    }
}

/// Extracts the value of the file ID query parameter, if it exists in the specified URI query.
fn get_queried_file_id(query: Option<&str>) -> Option<&str> {
    query?
        .split('&')
        .find_map(|param| param.strip_prefix(FILE_ID_QUERY_PREFIX))
}
