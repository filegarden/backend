//! A web server for the HTTP API. File Garden exposes this via `https://filegarden.com/api/`.

use std::error::Error as _;

use axum::{
    extract::{rejection::JsonRejection, Request},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::FromRequest;
use routes::ROUTER;
use serde::Serialize;
use strum_macros::IntoStaticStr;
use thiserror::Error;
use tower::ServiceExt;

pub mod routes;
pub mod validate;

/// An API error.
#[derive(Debug, Error, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Error {
    /// The request body is too large.
    #[error("The request body is too large.")]
    ContentTooLarge,

    /// A CSPRNG operation failed.
    #[error("Couldn't securely invoke the server's random number generator. Please try again.")]
    Csprng,

    /// A database operation failed.
    #[error("An internal database error occurred. Please try again.")]
    Database,

    /// The `Content-Type` header isn't set to `application/json`.
    #[error("Header `Content-Type: application/json` must be set.")]
    JsonContentType,

    /// The JSON syntax is incorrect.
    #[error("Invalid JSON syntax: {0}")]
    JsonSyntax(String),

    /// The requested API route doesn't exist.
    #[error("The requested API route doesn't exist.")]
    RouteNotFound,

    /// An error occurred which is unknown or expected never to happen.
    #[error("An unexpected internal server error occurred: {0}")]
    Unknown(String),

    /// The request body doesn't match the target type and its validation conditions.
    #[error("Invalid request data: {0}")]
    Validation(String),
}

impl Error {
    /// Gets the HTTP response status code corresponding to the API error.
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::ContentTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Csprng => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::JsonSyntax(_) => StatusCode::BAD_REQUEST,
            Self::RouteNotFound => StatusCode::NOT_FOUND,
            Self::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Gets the API error's code in `SCREAMING_SNAKE_CASE`.
    fn code(&self) -> &'static str {
        self.into()
    }
}

impl From<rand::Error> for Error {
    fn from(_: rand::Error) -> Self {
        Self::Csprng
    }
}

impl From<sqlx::Error> for Error {
    fn from(_: sqlx::Error) -> Self {
        Self::Database
    }
}

impl From<JsonRejection> for Error {
    fn from(error: JsonRejection) -> Self {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            return Self::ContentTooLarge;
        }

        match error {
            JsonRejection::JsonDataError(error) => Self::Validation(match error.source() {
                Some(source) => source.to_string(),
                None => error.body_text(),
            }),
            JsonRejection::JsonSyntaxError(error) => Self::JsonSyntax(match error.source() {
                Some(source) => source.to_string(),
                None => error.body_text(),
            }),
            JsonRejection::MissingJsonContentType(_) => Self::JsonContentType,
            error => Self::Unknown(error.body_text()),
        }
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

/// Equivalent to [`axum::Json`], but fails with an [`Error`] JSON response instead of a plain text
/// response.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

/// An API response type.
pub type Response<T> = std::result::Result<(StatusCode, Json<T>), Error>;

/// Routes a request to an API endpoint.
pub(super) async fn handle(request: Request) -> axum::response::Response {
    // Calling the router needs a mutable reference to it (even though it shouldn't), so the router
    // must either have restricted access via a mutex or be cloned on each request. The former would
    // allow only one request at a time, so the latter is faster.
    ROUTER.clone().oneshot(request).await.into_response()
}
