//! Route handlers for routes to files.

use std::borrow::Cow;

use axum::{
    extract::Request,
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// Route handler for `GET` on routes to files.
#[debug_handler]
pub(crate) async fn get(req: Request) -> impl IntoResponse {
    let initial_uri = req.uri();
    let initial_path = initial_uri.path();
    let initial_query = initial_uri.query();

    let mut path = initial_path.trim_end_matches('/');
    let query = initial_query;

    if path.is_empty() {
        path = "/";
    }

    // TODO: Also normalize the queried file ID to be the correct file ID with correct URI encoding.
    // (And note that normalizing other query parameters is unnecessary.) Don't normalize the
    // presence of a file ID query, so that an extra redirect and database query isn't required when
    // it's omitted.

    if (initial_path, initial_query) != (path, query) {
        // Redirect to a normalized location to reduce how many URLs must be purged from the CDN's
        // cache when a file is changed. It's impossible to purge every possible variation of a URL.

        let normalized_uri = concat_path_and_query(path, query);

        return Redirect::permanent(&normalized_uri).into_response();
    }

    let (user_identifier, file_path) = parse_file_route_path(path);

    let file_id = get_queried_file_id(query);

    format!(
        "{user_identifier} - {file_path} - {}",
        file_id.unwrap_or("None")
    )
    .into_response()
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

    let user_identifier_end = path.find('/').unwrap_or(path.len());
    let (user_identifier, mut file_path) = path.split_at(user_identifier_end);

    if file_path.is_empty() {
        file_path = "/";
    }

    (user_identifier, file_path)
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
