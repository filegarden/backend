//! A web server to proxy the files stored by users. File Garden exposes this through
//! `https://file.garden/...`.
//!
//! For security, this server must be exposed on a separate origin from the website. Otherwise, a
//! user could upload an HTML file containing an XSS attack (for example, containing a script which
//! sends a request to the website's API authenticated by the client's cookies, allowing a page to
//! act on behalf of a user without their knowledge).

pub(crate) mod percent_encoding;
pub(crate) mod response;
mod service;

use std::io;

use axum::handler::HandlerWithoutStateExt;
use tokio::net::TcpListener;

/// The URL to the website.
pub const WEBSITE_URI: &str = "https://filegarden.com/";

/// The address the server should listen on.
const LISTENER_ADDR: &str = "0.0.0.0:3001";

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(LISTENER_ADDR).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{LISTENER_ADDR}");
    }

    axum::serve(listener, service::handler.into_make_service()).await?;

    Ok(())
}
