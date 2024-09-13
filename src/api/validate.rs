//! Utilities to help with API request validation.

use std::str::FromStr;

use derive_more::derive::{AsRef, Deref, Display};
use idna::uts46::{self, Uts46};
use lettre::Address;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
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

/// A user-inputted email address. Ensures the address uses a domain name with a TLD, and normalizes
/// the domain name (for non-ASCII characters).
#[derive(
    Deref,
    AsRef,
    Display,
    DeserializeFromStr,
    Serialize,
    Clone,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Hash,
    Debug,
)]
#[as_ref(forward)]
pub struct UserEmail(Address);

/// An error constructing a [`UserEmail`].
#[derive(Error, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum UserEmailError {
    /// The email address was invalid.
    #[error("invalid email address")]
    Invalid,

    /// The domain part was an IP address rather than a domain name. There's no reason to let users
    /// use IP addresses in emails; strict mail agents don't even allow it.
    #[error("IP addresses not allowed in email address")]
    IpAddr,

    /// The domain name has no TLD. This is likely a typo or a user trying to exploit `localhost`.
    #[error("domain in email address is missing a TLD")]
    NoTld,
}

impl FromStr for UserEmail {
    type Err = UserEmailError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let Some((user, domain)) = str.rsplit_once('@') else {
            return Err(UserEmailError::Invalid);
        };

        if domain.starts_with('[') {
            return Err(UserEmailError::IpAddr);
        }

        if !domain.contains('.') {
            return Err(UserEmailError::NoTld);
        }

        let (domain, domain_result) = Uts46::new().to_user_interface(
            // These are the recommended arguments for this function.
            domain.as_bytes(),
            uts46::AsciiDenyList::URL,
            uts46::Hyphens::Allow,
            |_, _, _| true,
        );

        if domain_result.is_err() {
            return Err(UserEmailError::Invalid);
        }

        let Ok(address) = Address::new(user, domain.to_lowercase()) else {
            return Err(UserEmailError::Invalid);
        };

        Ok(Self(address))
    }
}

#[cfg(test)]
#[expect(clippy::missing_errors_doc, reason = "see rust-lang/rust-clippy#13391")]
mod tests {
    use super::*;

    #[test]
    fn user_email_validation() {
        let invalid_emails = [
            "invalid",
            "user@example-.com",
            "user@[127.0.0.1]",
            "user@[::1]",
            "user@examplecom",
        ];

        for email in invalid_emails {
            email
                .parse::<UserEmail>()
                .expect_err("user email should be invalid");
        }
    }

    #[test]
    fn user_email_normalization() -> anyhow::Result<()> {
        // The user portion isn't all lowercase or all uppercase when normalized because RFC 5321
        // (section 2.3.11) lets mail servers treat the user portion case-sensitively.
        let normalized_email = "uSeR@examplé.com";

        let equivalent_emails = [
            "uSeR@examplé.com",
            "uSeR@example\u{0301}.com",
            "uSeR@EXAMPLÉ.com",
            "uSeR@EXAMPLE\u{0301}.com",
            "uSeR@xn--exampl-gva.com",
            "uSeR@xN--eXaMpL-gVa.CoM",
        ];

        for email in equivalent_emails {
            assert_eq!(
                normalized_email,
                email.parse::<UserEmail>()?.as_ref() as &str
            );
        }

        Ok(())
    }
}
