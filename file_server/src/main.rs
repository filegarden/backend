//! A web server to proxy the files stored by users. File Garden exposes this through
//! `https://file.garden/...`.
//!
//! For security, this server must be exposed on a separate origin from the website. Otherwise, a
//! user could upload an HTML file containing an XSS attack (for example, containing a script which
//! sends a request to the website's API authenticated by the client's cookies, allowing a page to
//! act on behalf of a user without their knowledge).

pub(crate) mod percent_encoding;
mod plain_error_response;
mod request;

use std::io;

use axum::{
    handler::{Handler, HandlerWithoutStateExt},
    http::{header, HeaderValue},
};
pub(crate) use plain_error_response::PlainErrorResponse;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

/// The URL to the website.
pub const WEBSITE_URI: &str = "https://filegarden.com/";

/// The address the server should listen on.
const LISTENER_ADDR: &str = "127.0.0.1:3001";

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(LISTENER_ADDR).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{LISTENER_ADDR}");
    }

    let service = request::handler
        .layer(CorsLayer::very_permissive())
        .layer(SetResponseHeaderLayer::overriding(
            header::ALLOW,
            HeaderValue::from_static("OPTIONS, GET, HEAD"),
        ))
        .into_make_service();

    axum::serve(listener, service).await?;

    Ok(())
}
