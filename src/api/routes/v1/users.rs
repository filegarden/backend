//! An HTTP resource representing the set of all user accounts.

use axum::http::StatusCode;
use axum_macros::debug_handler;
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;

use crate::{
    api::{
        auth::hash_password,
        validation::{Birthdate, UserEmail, UserName, UserPassword},
        Json, Response,
    },
    db,
    id::{NewUserId, Token},
};

/// A `POST` request body for this API route.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostRequest {
    /// The user's email address.
    pub email: UserEmail,

    /// The user's name.
    pub name: UserName,

    /// The user's birthdate, from a string in ISO 8601 date format.
    pub birthdate: Birthdate,

    /// The user's password in plain text.
    pub password: UserPassword,
}

/// Creates a new user and sends them an account verification email.
///
/// # Errors
///
/// See [`crate::api::Error`].
#[debug_handler]
pub async fn post(Json(body): Json<PostRequest>) -> Response<PostResponse> {
    let mut user_id = NewUserId::generate()?;

    let password_hash = hash_password(&body.password)?;

    let mut tx = db::pool().begin().await?;

    let email_taken = sqlx::query!(
        "SELECT 1 AS x FROM users
            WHERE email = $1",
        body.email.as_str(),
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some();

    if !email_taken {
        loop {
            // If this loop's query fails from an ID conflict, this savepoint is rolled back to
            // rather than aborting the entire transaction.
            let mut savepoint = tx.begin().await?;

            match sqlx::query!(
                "INSERT INTO users (id, name, birthdate, password_hash)
                    VALUES ($1, $2, $3, $4)",
                user_id.as_slice(),
                *body.name,
                *body.birthdate,
                password_hash,
            )
            .execute(&mut *savepoint)
            .await
            {
                Err(sqlx::Error::Database(error)) if error.constraint() == Some("users_pkey") => {
                    user_id.reroll()?;
                    continue;
                }
                result => result?,
            };

            savepoint.commit().await?;
            break;
        }

        let email_verification_token = Token::generate()?;
        let email_verification_token_hash = digest(&SHA256, email_verification_token.as_slice());

        sqlx::query!(
            "INSERT INTO unverified_emails (user_id, email, token_hash)
                VALUES ($1, $2, $3)",
            user_id.as_slice(),
            body.email.as_str(),
            email_verification_token_hash.as_ref(),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(PostResponse {
            email: body.email,
            name: body.name,
        }),
    ))
}

/// A `POST` response body for this API route.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    /// The user's email address.
    pub email: UserEmail,

    /// The user's name.
    pub name: UserName,
}
