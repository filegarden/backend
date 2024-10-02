//! The set of email verification requests for new users.

use axum::http::StatusCode;
use axum_macros::debug_handler;
use lettre::{message::Mailbox, AsyncTransport};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;

use crate::{
    api::{validation::UserEmail, Json, Response},
    crypto::hash_without_salt,
    db,
    email::{EmailTakenMessage, MessageTemplate, VerificationMessage, MAILER},
    id::Token,
    WEBSITE_ORIGIN,
};

pub mod code;

/// A `POST` request body for this API route.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostRequest {
    /// The email address to verify.
    pub email: UserEmail,
}

/// Sends a verification email for a new user if the email isn't already taken by an existing user.
///
/// # Errors
///
/// See [`crate::api::Error`].
#[debug_handler]
pub async fn post(Json(body): Json<PostRequest>) -> Response<PostResponse> {
    let mut tx = db::pool().begin().await?;

    let existing_user = sqlx::query!(
        "SELECT name FROM users
            WHERE email = $1",
        body.email.as_str(),
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(user) = existing_user {
        let email = EmailTakenMessage {
            email: body.email.as_str(),
        }
        .to(Mailbox::new(Some(user.name), (*body.email).clone()));

        tokio::spawn(MAILER.send(email));
    } else {
        sqlx::query!(
            "DELETE FROM unverified_emails
                WHERE user_id IS NULL AND email = $1",
            body.email.as_str(),
        )
        .execute(&mut *tx)
        .await?;

        let mut token = Token::generate()?;

        loop {
            // If this loop's query fails from a token conflict, this savepoint is rolled back to
            // rather than aborting the entire transaction.
            let mut savepoint = tx.begin().await?;

            let token_hash = hash_without_salt(&token);

            match sqlx::query!(
                "INSERT INTO unverified_emails (token_hash, email)
                    VALUES ($1, $2)",
                token_hash.as_ref(),
                body.email.as_str(),
            )
            .execute(&mut *savepoint)
            .await
            {
                Err(sqlx::Error::Database(error))
                    if error.constraint() == Some("unverified_emails_pkey") =>
                {
                    token.reroll()?;
                    continue;
                }
                result => result?,
            };

            savepoint.commit().await?;
            break;
        }

        let email = VerificationMessage {
            email: body.email.as_str(),
            verification_url: &format!("{}/sign-up?token={}", *WEBSITE_ORIGIN, token),
        }
        .to(Mailbox::new(None, (*body.email).clone()));

        tokio::spawn(MAILER.send(email));
    }

    tx.commit().await?;

    // To prevent user enumeration, send this same successful response even if the email is taken.
    Ok((StatusCode::OK, Json(PostResponse { email: body.email })))
}

/// A `POST` response body for this API route.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    /// The normalized email address to verify.
    pub email: UserEmail,
}
