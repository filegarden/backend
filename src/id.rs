//! See [`Id`].

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use derive_more::derive::{AsMut, AsRef, Deref, DerefMut};
use rand::RngCore;
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// An ID that can be deserialized from and serialized to `base64url` (without padding).
#[derive(
    Deref,
    DerefMut,
    AsRef,
    AsMut,
    DeserializeFromStr,
    SerializeDisplay,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Id<T>(T);

impl<const N: usize> Id<[u8; N]> {
    /// Generates a cryptographically secure pseudorandom ID.
    ///
    /// # Errors
    ///
    /// Fails if the CSPRNG fails to obtain random bytes.
    pub fn generate() -> Result<Self, rand::Error> {
        let mut id = Self([0; N]);
        id.reroll()?;
        Ok(id)
    }
}

impl<T: AsMut<[u8]>> Id<T> {
    /// Overwrites this ID with a new cryptographically secure pseudorandom ID, reusing the existing
    /// memory.
    ///
    /// # Errors
    ///
    /// Fails if the CSPRNG fails to obtain random bytes.
    pub fn reroll(&mut self) -> Result<(), rand::Error> {
        rand::thread_rng().try_fill_bytes(self.as_mut())?;
        Ok(())
    }
}

impl FromStr for Id<Vec<u8>> {
    type Err = base64::DecodeError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let bytes = URL_SAFE_NO_PAD.decode(str)?;
        Ok(Self(bytes))
    }
}

impl<T: AsRef<[u8]>> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", URL_SAFE_NO_PAD.encode(self))
    }
}
