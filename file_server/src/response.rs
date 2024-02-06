//! See [`Response`].

use axum::{
    body::Body,
    http::{self, HeaderName, HeaderValue, StatusCode},
};

/// A wrapper for [`axum::response::Response`] with a simpler API.
pub(crate) struct Response {
    /// The [`axum::response::Response`] value being wrapped.
    inner: axum::response::Response,
}

impl Response {
    /// Constructs a new [`Response`].
    pub(crate) fn new() -> Self {
        Self {
            inner: axum::response::Response::new(Body::empty()),
        }
    }

    /// Sets a [`StatusCode`] on the response.
    pub(crate) fn status(&mut self, status: StatusCode) -> &mut Self {
        *self.inner.status_mut() = status;

        self
    }

    /// Sets a header on the response.
    pub(crate) fn header(&mut self, name: HeaderName, value: HeaderValue) -> &mut Self {
        self.inner.headers_mut().insert(name, value);

        self
    }

    /// Sets a header on the response, panicking if the header is invalid.
    ///
    /// # Panics
    ///
    /// Panics if the header name or value isn't valid. For example, passing a string panics if it
    /// contains a character that isn't visible ASCII (32-127).
    pub(crate) fn header_valid<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key)
            .map_err(Into::into)
            .expect("header name should be valid");
        let value = <HeaderValue as TryFrom<V>>::try_from(value)
            .map_err(Into::into)
            .expect("header value should be valid");

        self.header(name, value)
    }

    /// Sets a [`Body`] on the response.
    pub(crate) fn body<T: Into<Body>>(mut self, body: T) -> Self {
        *self.inner.body_mut() = body.into();

        self
    }

    /// Sets a [`StatusCode`], and sets it along with its canonical reason text (e.g. `404 Not
    /// Found`) as a `text/plain` body on the response.
    pub(crate) fn plain_error(mut self, status: StatusCode) -> Self {
        self.status(status)
            .header_valid("Content-Type", "text/plain");

        self.body(status.to_string())
    }
}

impl axum::response::IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        self.inner
    }
}
