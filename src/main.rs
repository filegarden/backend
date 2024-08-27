//! File Garden's backend web server.

mod api;
mod content;
mod db;
mod percent_encoding;
mod response;
mod router;
mod website;

use std::sync::LazyLock;

use axum::handler::HandlerWithoutStateExt;
use tokio::net::TcpListener;

/// The URI origin for user-uploaded content.
pub static CONTENT_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    dotenvy::var("CONTENT_ORIGIN").expect("environment variable `CONTENT_ORIGIN` should be set")
});

/// The URI origin for the website.
pub static WEBSITE_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    dotenvy::var("WEBSITE_ORIGIN").expect("environment variable `WEBSITE_ORIGIN` should be set")
});

/// # Errors
///
/// See implementation.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let address = dotenvy::var("ADDRESS")?;
    let db_url = dotenvy::var("DATABASE_URL")?;

    println!("Initializing database pool...");

    db::initialize_pool(&db_url);

    println!("Migrating database...");

    sqlx::migrate!().run(db::pool()).await?;

    println!("Listening to {address}...");

    let listener = TcpListener::bind(address).await?;

    println!("Ready!");

    axum::serve(listener, router::handle.into_make_service()).await?;

    Ok(())
}
