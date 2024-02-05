//! See [`handler`].

use std::borrow::Cow;

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_macros::debug_handler;
use percent_encoding::{percent_decode_str, utf8_percent_encode};

use crate::{percent_encoding::COMPONENT_IGNORING_SLASH, WEBSITE_URI};

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// The service function to handle incoming requests.
#[allow(clippy::unused_async)] // Axum route handlers must be async.
#[debug_handler]
pub(super) async fn handler(req: Request) -> Response {
    let mut response = Response::builder();

    let method = req.method();

    if !(method == Method::GET || method == Method::HEAD) {
        let status = if method == Method::OPTIONS {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::METHOD_NOT_ALLOWED
        };

        return response
            .status(status)
            .header("Allow", "GET, HEAD, OPTIONS")
            .body(Body::empty())
            .expect("response should be valid");
    }

    let uri = req.uri();
    let initial_path = uri.path();

    if initial_path == "/" {
        return Redirect::permanent(WEBSITE_URI).into_response();
    }

    let Ok(path) = percent_decode_str(initial_path).decode_utf8() else {
        return plain_error_response(StatusCode::BAD_REQUEST);
    };

    // The above can decode `%00` into a null byte, so disallow null bytes as a defensive measure.
    if path.contains('\x00') {
        return plain_error_response(StatusCode::BAD_REQUEST);
    }

    let normalized_encoded_path: Cow<str> =
        utf8_percent_encode(&path, COMPONENT_IGNORING_SLASH).into();

    let query = uri.query();

    if initial_path != normalized_encoded_path {
        // Redirect to the same URI with normalized path encoding. This reduces how many URLs must
        // be purged from the CDN's cache when a file changes. It's impossible to purge every
        // possible variation of encoding for a URL.

        let normalized_uri = concat_path_and_query(&normalized_encoded_path, query);
        return Redirect::permanent(&normalized_uri).into_response();
    }

    let (user_identifier, file_path) = parse_file_route_path(&path);
    let file_id = get_queried_file_id(query);

    // TODO: Send the correct `Content-Length`.
    response = response.header("Content-Length", 0);

    if method == Method::HEAD {
        return response
            .body(Body::empty())
            .expect("response should be valid");
    }

    format!(
        "{user_identifier} - {file_path} - {}",
        file_id.unwrap_or("None")
    )
    .into_response()
}

/// Generates a `text/plain` response containing the specified status code and its canonical reason
/// text (e.g. `404 Not Found`).
fn plain_error_response(status: StatusCode) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .body(Body::from(status.to_string()))
        .expect("response should be valid")
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
    let Some(query) = query else {
        return None;
    };

    query
        .split('&')
        .find_map(|param| param.strip_prefix(FILE_ID_QUERY_PREFIX))
}
