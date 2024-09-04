//! Utilities to help with API request validation.

use std::ops::Deref;

use serde::{de, Deserialize, Deserializer};
use serde_with::SerializeDisplay;
use thiserror::Error;
use time::{format_description::well_known::Iso8601, Date};

/// A user's name.
pub type UserName = BoundedString<1, 64>;

/// A user's password in plain text.
pub type UserPassword = BoundedString<8, 256>;

/// Deserializes a string in ISO 8601 format (with only the date part) to a [`Date`].
///
/// # Errors
///
/// Fails if the input is an invalid [`String`] or invalid ISO 8601 date.
pub fn deserialize_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
    let str = <&str>::deserialize(deserializer)?;

    Date::parse(str, &Iso8601::DATE).map_err(de::Error::custom)
}

/// A [`String`] newtype that guarantees its length is within a certain range.
#[derive(Debug, Deserialize, SerializeDisplay)]
#[serde(try_from = "String")]
pub struct BoundedString<const MIN: usize, const MAX: usize>(String);

/// An error initializing a [`BoundedString`] from a [`String`].
#[derive(Debug, Error)]
pub enum BoundedStringError<const MIN: usize, const MAX: usize> {
    /// The length was less than the [`BoundedString`]'s `MIN`.
    #[error("invalid length {0}, expected at least {MIN}")]
    TooShort(usize),

    /// The length was greater than the [`BoundedString`]'s `MAX`.
    #[error("invalid length {0}, expected at most {MAX}")]
    TooLong(usize),
}

impl<const MIN: usize, const MAX: usize> TryFrom<String> for BoundedString<MIN, MAX> {
    type Error = BoundedStringError<MIN, MAX>;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        if string.len() < MIN {
            Err(BoundedStringError::TooShort(string.len()))
        } else if string.len() > MAX {
            Err(BoundedStringError::TooLong(string.len()))
        } else {
            Ok(Self(string))
        }
    }
}

impl<const MIN: usize, const MAX: usize> Deref for BoundedString<MIN, MAX> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> AsRef<T> for BoundedString<MIN, MAX>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
