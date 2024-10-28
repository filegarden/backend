//! General database handling.

use std::{error::Error, sync::OnceLock};

use castaway::cast;
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

/// The error result of a database transaction.
///
/// Doesn't implement [`Error`] to prevent an impl conflict.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum TxError<E> {
    /// Aborts the transaction and returns the wrapped error.
    Abort(E),

    /// Aborts the transaction and runs the `transaction!` callback again.
    Retry,
}

/// The SQLSTATE code for serialization failures.
const SERIALIZATION_FAILURE: &str = "40001";

impl<S, E> From<S> for TxError<E>
where
    // This `Error` bound may be overly restrictive but prevents an impl conflict.
    S: Error + 'static,
    E: From<S>,
{
    fn from(source: S) -> Self {
        match cast!(&source, &sqlx::Error) {
            Ok(sqlx::Error::Database(source))
                if source
                    .code()
                    .is_some_and(|code| code == SERIALIZATION_FAILURE) =>
            {
                Self::Retry
            }
            _ => Self::Abort(source.into()),
        }
    }
}

/// The result of a database transaction.
pub(crate) type TxResult<T, E> = Result<T, TxError<E>>;

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
            #[expect(clippy::allow_attributes, reason = "`unused_mut` isn't always expected")]
            #[allow(unused_mut, reason = "some callers need this to be `mut`")]
            let mut callback = $callback;

            #[expect(clippy::allow_attributes, reason = "`unused_mut` isn't always expected")]
            #[allow(unused_mut, reason = "some callers need this to be `mut`")]
            let mut callback = async || -> $crate::db::TxResult<_, _> {
                let mut tx = $crate::db::pool().begin().await?;

                let return_value = match callback(&mut tx).await {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                tx.commit().await?;
                Ok(return_value)
            };

            loop {
                match callback().await {
                    Ok(value) => break Ok(value),
                    Err($crate::db::TxError::Abort(error)) => break Err(error),
                    Err($crate::db::TxError::Retry) => {}
                }
            }
        }
    };
}

pub(crate) use transaction;
