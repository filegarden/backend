//! Utilities to help with API request validation.

use serde::{de, Deserialize, Deserializer};
use time::{format_description::well_known::Iso8601, Date};

/// Deserializes a string in ISO 8601 format (with only the date part) to a [`Date`].
///
/// # Errors
///
/// Fails if the input is an invalid [`String`] or invalid ISO 8601 date.
pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    Date::parse(&string, &Iso8601::DATE).map_err(de::Error::custom)
}
