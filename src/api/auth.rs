//! Various API utilities to help with authentication and authorization.

use argon2::{
    password_hash::{Salt, SaltString},
    Argon2, PasswordHasher,
};
use rand::RngCore;

/// Hashes and salts a password using Argon2.
///
/// # Errors
///
/// Fails if the CSPRNG fails when generating salt.
pub(super) fn hash_password(password: &str) -> Result<String, rand::Error> {
    let mut salt = [0; Salt::RECOMMENDED_LENGTH];
    rand::thread_rng().try_fill_bytes(&mut salt)?;

    let salt_string = SaltString::encode_b64(&salt).expect("salt should be valid");

    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt_string)
        .expect("password hashing should be infallible")
        .to_string())
}
