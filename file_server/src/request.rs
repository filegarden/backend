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

use crate::{percent_encoding::COMPONENT_IGNORING_SLASH, PlainErrorResponse, WEBSITE_URI};

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// The route handler for all routes.
#[allow(clippy::unused_async)] // Axum route handlers must be async.
#[debug_handler]
pub(crate) async fn handler(req: Request) -> Result<Response, PlainErrorResponse> {
    let method = req.method();

    if method == Method::OPTIONS {
        let response = Response::builder()
            .status(StatusCode::NO_CONTENT)
            // TODO: Always include this header (especially since it's required for status 405).
            .header("Allow", "OPTIONS, GET, HEAD")
            .body(Body::empty())
            .expect("request should be valid");

        return Ok(response);
    }

    if !(method == Method::GET || method == Method::HEAD) {
        return Err(StatusCode::METHOD_NOT_ALLOWED.into());
    }

    let uri = req.uri();
    let initial_path = uri.path();

    if initial_path == "/" {
        return Ok(Redirect::permanent(WEBSITE_URI).into_response());
    }

    let path = percent_decode_str(initial_path)
        .decode_utf8()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // The above can decode `%00` into a null byte, so disallow null bytes as a defensive measure.
    if path.contains('\x00') {
        return Err(StatusCode::BAD_REQUEST.into());
    }

    let normalized_encoded_path: Cow<str> =
        utf8_percent_encode(&path, COMPONENT_IGNORING_SLASH).into();

    let query = uri.query();

    if initial_path != normalized_encoded_path {
        // Redirect to the same URI with normalized path encoding. This reduces how many URLs must
        // be purged from the CDN's cache when a file changes. It's impossible to purge every
        // possible variation of encoding for a URL.

        let normalized_uri = concat_path_and_query(&normalized_encoded_path, query);
        return Ok(Redirect::permanent(&normalized_uri).into_response());
    }

    let (user_identifier, file_path) = parse_file_route_path(&path);
    let file_id = get_queried_file_id(query);

    let response = Response::builder()
        // TODO: Send the correct `Content-Length`.
        .header("Content-Length", 0);

    if method == Method::HEAD {
        let response = response
            .body(Body::empty())
            .expect("request should be valid");

        return Ok(response);
    }

    Ok(format!(
        "{user_identifier} - {file_path} - {}",
        file_id.unwrap_or("None")
    )
    .into_response())
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
