//! Utilities to help with API request validation.

use derive_more::derive::{AsRef, Deref, Display};
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
#[derive(
    Deref,
    AsRef,
    Display,
    Deserialize,
    SerializeDisplay,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
)]
#[as_ref(forward)]
#[serde(try_from = "String")]
pub struct BoundedString<const MIN: usize, const MAX: usize>(String);

/// An error constructing a [`BoundedString`].
#[derive(Error, PartialEq, Eq, Clone, Copy, Debug)]
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
