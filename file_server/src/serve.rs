//! See [`serve`].

use std::{
    borrow::{Borrow, Cow},
    convert::Infallible,
};

use http_body_util::Full;
use hyper::{
    body::{Body, Buf, Incoming},
    Method, Request, Response, StatusCode,
};
use percent_encoding::{percent_decode_str, utf8_percent_encode};

use crate::{percent_encoding::COMPONENT_IGNORING_SLASH, PlainErrorResponse, WEBSITE_URI};

/// The start of a file ID query parameter.
const FILE_ID_QUERY_PREFIX: &str = "_id=";

/// The service function to handle all requests.
pub(crate) async fn serve(
    req: Request<Incoming>,
) -> Result<Response<impl Body<Data = impl Buf, Error = Infallible>>, Infallible> {
    Ok(handle(&req))
}

/// The function to handle all requests.
fn handle(req: &Request<Incoming>) -> Response<impl Body<Data = impl Buf, Error = Infallible>> {
    let method = req.method();

    if method == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            // TODO: Always include this header (especially since it's required for status 405).
            .header("Allow", "OPTIONS, GET, HEAD")
            .body(Full::from(""))
            .expect("request should be valid");
    }

    if !(method == Method::GET || method == Method::HEAD) {
        return PlainErrorResponse::from(StatusCode::METHOD_NOT_ALLOWED).into();
    }

    let uri = req.uri();
    let initial_path = uri.path();

    if initial_path == "/" {
        return Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", WEBSITE_URI)
            .body(Full::from(""))
            .expect("response should be valid");
    }

    let Ok(path) = percent_decode_str(initial_path).decode_utf8() else {
        return PlainErrorResponse::from(StatusCode::BAD_REQUEST).into();
    };

    // The above can decode `%00` into a null byte, so disallow null bytes as a defensive measure.
    if path.contains('\x00') {
        return PlainErrorResponse::from(StatusCode::BAD_REQUEST).into();
    }

    let normalized_encoded_path: Cow<str> =
        utf8_percent_encode(&path, COMPONENT_IGNORING_SLASH).into();

    let query = uri.query();

    if initial_path != normalized_encoded_path {
        // Redirect to the same URI with normalized path encoding. This reduces how many URLs must
        // be purged from the CDN's cache when a file changes. It's impossible to purge every
        // possible variation of encoding for a URL.

        let normalized_uri = concat_path_and_query(&normalized_encoded_path, query);

        return Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", normalized_uri.borrow() as &str)
            .body(Full::from(""))
            .expect("response should be valid");
    }

    let (user_identifier, file_path) = parse_file_route_path(&path);
    let file_id = get_queried_file_id(query);

    let response = Response::builder();
    // TODO: Send the correct `Content-Length`.
    // .header("Content-Length", 0);

    if method == Method::HEAD {
        return response
            .body(Full::from(""))
            .expect("request should be valid");
    }

    response
        .body(Full::from(format!(
            "{user_identifier} - {file_path} - {}",
            file_id.unwrap_or("None")
        )))
        .expect("request should be valid")
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
