//! Route handlers for routes to files.

use axum::{
    extract::Request,
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;

/// Route handler for `GET` on routes to files.
#[debug_handler]
pub(crate) async fn get(req: Request) -> impl IntoResponse {
    let uri = req.uri();

    let path = uri.path();
    let query_str = match uri.query() {
        Some(query_str) => format!("?{query_str}"),
        None => "".to_string(),
    };

    let normalized_path = normalize_path(path);

    // TODO: Also normalize URI encoding and the file ID query.

    if path != normalized_path {
        return Redirect::permanent(&format!("{normalized_path}{query_str}")).into_response();
    }

    let (user_identifier, file_path) = parse_file_route_path(path);

    format!("{user_identifier} - {file_path}").into_response()
}

/// Fixes all the quirks in the syntax of the specified URI path.
fn normalize_path(path: &str) -> String {
    let mut normalized_path = path.to_string();

    strip_repeat_slashes(&mut normalized_path);

    if normalized_path != "/" {
        normalized_path = normalized_path
            .strip_suffix('/')
            .unwrap_or(&normalized_path)
            .to_string();
    }

    normalized_path
}

/// Extracts a tuple of the user identifier and file path from the path of a file route URI.
fn parse_file_route_path(path: &str) -> (&str, &str) {
    let path = path.strip_prefix('/').expect("path should start with `/`");

    let user_identifier_end = path.find('/').unwrap_or(path.len());
    let (user_identifier, mut file_path) = path.split_at(user_identifier_end);

    if file_path.is_empty() {
        file_path = "/";
    }

    (user_identifier, file_path)
}

/// Replaces each instance of multiple consecutive slashes with a single slash.
fn strip_repeat_slashes(string: &mut String) {
    let mut prev_char = char::default();

    string.retain(|char| {
        let is_repeat = char == '/' && prev_char == '/';
        prev_char = char;

        !is_repeat
    });
}
