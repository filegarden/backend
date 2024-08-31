//! A web server for the HTTP API. File Garden exposes this via `https://filegarden.com/api/`.

use axum::{extract::Request, http::StatusCode, response::IntoResponse, Json};
use routes::ROUTER;
use serde::Serialize;
use strum_macros::IntoStaticStr;
use thiserror::Error;
use tower::ServiceExt;

pub mod routes;

/// An API error.
#[derive(Debug, Error, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Error {
    /// A CSPRNG operation failed.
    #[error("Couldn't securely invoke the server's random number generator. Please try again.")]
    Csprng(#[from] rand::Error),

    /// A database operation failed.
    #[error("An internal database error occurred. Please try again.")]
    Database(#[from] sqlx::Error),

    /// Validation failed for a value in the request body.
    #[error("The request body contains an invalid value.")]
    Validation,
}

impl Error {
    /// Gets the HTTP response status code corresponding to the API error.
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::Csprng(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation => StatusCode::BAD_REQUEST,
        }
    }

    /// Gets the API error's code in `SCREAMING_SNAKE_CASE`.
    fn code(&self) -> &'static str {
        self.into()
    }
}

/// An API error's response body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    /// The computer-friendly error code in `SCREAMING_SNAKE_CASE`. See [`Error`] for error codes.
    pub code: &'static str,

    /// The human-friendly error message.
    pub message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
        };

        (self.status(), Json(body)).into_response()
    }
}

/// An API response type.
pub(crate) type Response<T> = std::result::Result<Json<T>, Error>;

/// Routes a request to an API endpoint.
pub(super) async fn handle(request: Request) -> axum::response::Response {
    // Calling the router needs a mutable reference to it (even though it shouldn't), so the router
    // must either have restricted access via a mutex or be cloned on each request. The former would
    // allow only one request at a time, so the latter is faster.
    ROUTER.clone().oneshot(request).await.into_response()
}
