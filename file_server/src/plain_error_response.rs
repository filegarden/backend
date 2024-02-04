//! See [`PlainErrorResponse`].

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// An error which implements [`IntoResponse`] by generating a `text/plain` response containing the
/// associated status code and its canonical reason text (e.g. `404 Not Found`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PlainErrorResponse {
    /// The [`StatusCode`] to generate the [`Response`] from.
    status: StatusCode,
}

impl PlainErrorResponse {
    /// Get the [`StatusCode`] associated with this response.
    pub(crate) fn status(self) -> StatusCode {
        self.status
    }
}

impl From<StatusCode> for PlainErrorResponse {
    fn from(status: StatusCode) -> Self {
        Self { status }
    }
}

impl IntoResponse for PlainErrorResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status())
            .header("Content-Type", "text/plain")
            .body(Body::from(self.status().to_string()))
            .expect("response should be valid")
    }
}
