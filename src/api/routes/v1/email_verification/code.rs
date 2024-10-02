//! The verification code of a new user's email verification request.

use axum::http::StatusCode;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use crate::{
    api::{self, Json, Query, Response},
    crypto::{generate_short_code, hash_with_salt, hash_without_salt},
    db,
    id::Token,
};

/// A `POST` request query for this API route.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostQuery {
    /// The email verification token.
    pub token: Token,
}

/// A `POST` request body for this API route.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostRequest {}

/// Generates a new email verification code for a new user.
///
/// # Errors
///
/// See [`crate::api::Error`].
#[debug_handler]
pub async fn post(
    Query(query): Query<PostQuery>,
    Json(_): Json<PostRequest>,
) -> Response<PostResponse> {
    let token_hash = hash_without_salt(&query.token);

    let code = generate_short_code();
    let code_hash = hash_with_salt(&code)?;

    let Some(unverified_email) = sqlx::query!(
        "UPDATE unverified_emails
            SET code_hash = $1
            WHERE token_hash = $2 AND user_id IS NULL
            RETURNING email",
        code_hash,
        token_hash.as_ref(),
    )
    .fetch_optional(db::pool())
    .await?
    else {
        return Err(api::Error::ResourceNotFound);
    };

    // To prevent user enumeration, send this same successful response even if the email is taken.
    Ok((
        StatusCode::OK,
        Json(PostResponse {
            email: unverified_email.email,
            code,
        }),
    ))
}

/// A `POST` response body for this API route.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    /// The user's email address.
    pub email: String,

    /// The user's email verification code.
    pub code: String,
}