//! All routes for the HTTP API.

use std::sync::LazyLock;

use axum::{routing::post, Router};

use crate::api;

pub mod v1 {
    //! The routes for version 1 of the HTTP API.

    pub mod users;
}

/// The API router.
pub(super) static ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/api/v1/users", post(v1::users::post))
        .fallback(|| async { api::Error::RouteNotFound })
});
