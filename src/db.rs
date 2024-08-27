//! General database handling.

use std::sync::OnceLock;

use sqlx::PgPool;

/// The SQLx database pool.
static DB_POOL: OnceLock<PgPool> = OnceLock::new();

/// Initializes the SQLx database pool.
///
/// # Panics
///
/// Panics if the database is already initialized.
pub(super) async fn initialize_pool(db_url: &str) -> sqlx::Result<(), sqlx::Error> {
    DB_POOL
        .set(PgPool::connect(db_url).await?)
        .expect("database pool shouldn't already be initialized");

    Ok(())
}

/// Gets the SQLx database pool.
///
/// # Panics
///
/// Panics if called before the database pool is initialized.
pub fn pool() -> &'static PgPool {
    DB_POOL
        .get()
        .expect("database pool should be initialized before use")
}
