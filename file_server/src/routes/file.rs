//! Route handlers for routes to files.

use std::borrow::Cow;

use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// All ASCII characters in the [component percent-encode
/// set](https://url.spec.whatwg.org/#component-percent-encode-set).
///
/// Using this with [`utf8_percent_encode`] gives identical results to JavaScript's
/// [`encodeURIComponent`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent).
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

/// The set of [`COMPONENT`] ASCII characters, but with `/` excluded.
///
/// Using this with [`utf8_percent_encode`] gives identical results to JavaScript's
/// [`encodeURIComponent`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent),
/// with the exception that `/` characters are left alone rather than percent-encoded.
const COMPONENT_IGNORING_SLASH: &AsciiSet = &COMPONENT.remove(b'/');

/// Route handler for `GET` on routes to files.
#[debug_handler]
pub(crate) async fn get(req: Request) -> Result<impl IntoResponse, StatusCode> {
    let uri = req.uri();

    let initial_path = uri.path();
    let query = uri.query();

    let path = percent_decode_str(initial_path)
        .decode_utf8()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // The above can decode `%00` into a null byte, so disallow null bytes as a defensive measure.
    if path.contains('\x00') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let normalized_encoded_path: Cow<str> =
        utf8_percent_encode(&path, COMPONENT_IGNORING_SLASH).into();

    if initial_path != normalized_encoded_path {
        // Redirect to the same URI with normalized path encoding. This reduces how many URLs must
        // be purged from the CDN's cache when a file changes. It's impossible to purge every
        // possible variation of encoding for a URL.

        let normalized_uri = concat_path_and_query(&normalized_encoded_path, query);
        return Ok(Redirect::permanent(&normalized_uri).into_response());
    }

    let (user_identifier, file_path) = parse_file_route_path(&path);
    let file_id = get_queried_file_id(query);

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
