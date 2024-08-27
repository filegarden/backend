//! File Garden's backend web server.

mod api;
mod content;
mod percent_encoding;
mod response;
mod router;
mod website;

use std::sync::{LazyLock, OnceLock};

use axum::handler::HandlerWithoutStateExt;
use sqlx::postgres::Postgres;
use sqlx::Pool;
use tokio::net::TcpListener;

/// The URI origin for user-uploaded content.
pub static CONTENT_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    dotenvy::var("CONTENT_ORIGIN").expect("environment variable `CONTENT_ORIGIN` should be set")
});

/// The URI origin for the website.
pub static WEBSITE_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    dotenvy::var("WEBSITE_ORIGIN").expect("environment variable `WEBSITE_ORIGIN` should be set")
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
    let address = dotenvy::var("ADDRESS")?;
    let db_url = dotenvy::var("DATABASE_URL")?;

    println!("Connecting to database...");

    DB_POOL
        .set(Pool::<Postgres>::connect(&db_url).await?)
        .expect("`DB_POOL` shouldn't already be set");

    println!("Migrating database...");

    sqlx::migrate!().run(db_pool()).await?;

    println!("Listening to {address}...");

    let listener = TcpListener::bind(address).await?;

    println!("Ready!");

    axum::serve(listener, router::handle.into_make_service()).await?;

    Ok(())
}
