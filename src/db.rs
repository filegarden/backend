//! General database handling.

use std::sync::OnceLock;

use sqlx::{postgres::PgPoolOptions, Executor, PgPool};

/// The SQLx database pool.
static DB_POOL: OnceLock<PgPool> = OnceLock::new();

/// Initializes the SQLx database pool and runs pending database migrations.
///
/// # Errors
///
/// Returns an error if the initial database connection or its migrations fail.
///
/// # Panics
///
/// Panics if the database is already initialized.
pub(super) async fn initialize() -> sqlx::Result<(), sqlx::Error> {
    let db_url = dotenvy::var("DATABASE_URL")
        .expect("environment variable `DATABASE_URL` should be a valid string");

    let pool = PgPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("SET default_transaction_isolation TO 'serializable';")
                    .await?;

                Ok(())
            })
        })
        .connect(&db_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    DB_POOL
        .set(pool)
        .expect("database pool shouldn't already be initialized");

    Ok(())
}

/// Gets the SQLx database pool.
///
/// # Panics
///
/// Panics if called before the database pool is initialized.
pub(crate) fn pool() -> &'static PgPool {
    DB_POOL
        .get()
        .expect("database pool should be initialized before use")
}

/// Begins a database transaction with the maximum isolation level (`SERIALIZABLE`), retrying if the
/// database detects a race condition (serialization failure).
///
/// Maximum isolation is used to minimize the possibility of data races. This generally greatly
/// simplifies database operations and reduces the mental overhead of working with them.
macro_rules! transaction {
    ($($ident:ident)* |$tx:ident| $(-> $Return:ty)? $block:block) => {
        $crate::db::transaction!(
            $($ident)* |$tx: &mut ::sqlx::Transaction<'static, ::sqlx::Postgres>| $(-> $Return)? {
                $block
            }
        )
    };

    ($callback:expr) => {
        async {
            #[expect(clippy::allow_attributes, reason = "`unused_mut` isn't always expected.")]
            #[allow(unused_mut, reason = "Some callers need this to be `mut`.")]
            let mut callback = $callback;

            #[expect(clippy::allow_attributes, reason = "`unused_mut` isn't always expected.")]
            #[allow(unused_mut, reason = "Some callers need this to be `mut`.")]
            let mut callback = async || {
                let mut tx = $crate::db::pool().begin().await?;

                let return_value = match callback(&mut tx).await {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                tx.commit().await?;
                Ok(return_value)
            };

            loop {
                // TODO: Handle serialization anomaly.
                match callback().await {
                    result => break result,
                };
            }
        }
    };
}

pub(crate) use transaction;
