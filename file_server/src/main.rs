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

use std::sync::OnceLock;

use axum::handler::HandlerWithoutStateExt;
use once_cell::sync::Lazy;
use sqlx::postgres::Postgres;
use sqlx::Pool;
use tokio::net::TcpListener;

/// The URL to the website.
pub static WEBSITE_URI: Lazy<String> = Lazy::new(|| {
    dotenvy::var("WEBSITE_URI").expect("environment variable `WEBSITE_URL` should be set")
});

/// The SQLx database pool.
static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

/// Gets the SQLx database pool.
///
/// # Panics
///
/// Panics if called before the database pool is initialized by [`main`].
pub fn db_pool() -> &'static Pool<Postgres> {
    DB_POOL
        .get()
        .expect("database pool should be initialized before use")
}

/// # Errors
///
/// See implementation.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = dotenvy::var("FILE_SERVER_ADDR")?;
    let db_url = dotenvy::var("DATABASE_URL")?;

    println!("Connecting to database...");

    DB_POOL
        .set(Pool::<Postgres>::connect(&db_url).await?)
        .expect("`DB_POOL` shouldn't already be set");

    println!("Migrating database...");

    sqlx::migrate!("../migrations").run(db_pool()).await?;

    println!("Listening to {addr}...");

    let listener = TcpListener::bind(addr).await?;

    println!("Ready!");

    axum::serve(listener, service::handler.into_make_service()).await?;

    Ok(())
}
