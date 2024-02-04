//! See [`PlainErrorResponse`].

use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};
use thiserror::Error;

/// An error which implements `Into<Response<_>>` by generating a `text/plain` response containing
/// the associated status code and its canonical reason text (e.g. `404 Not Found`).
#[derive(Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[error("{status}")]
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

impl From<PlainErrorResponse> for Response<Full<Bytes>> {
    fn from(val: PlainErrorResponse) -> Self {
        Response::builder()
            .status(val.status())
            .header("Content-Type", "text/plain")
            .body(Full::from(val.status().to_string()))
            .expect("response should be valid")
    }
}
