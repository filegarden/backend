//! See [`Id`].

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use derive_more::derive::{AsRef, Deref};
use rand::RngCore;
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// An ID that can be deserialized from and serialized to `base64url` (without padding).
#[derive(Debug, Deref, AsRef, DeserializeFromStr, SerializeDisplay, Clone)]
#[as_ref(forward)]
pub struct Id<T>(T);

impl<T> Id<T>
where
    T: AsMut<[u8]> + Default,
{
    /// Generates a cryptographically secure pseudorandom ID.
    ///
    /// # Errors
    ///
    /// Fails if the CSPRNG fails to obtain random bytes.
    pub(crate) fn generate() -> Result<Self, rand::Error> {
        let mut bytes = T::default();
        rand::thread_rng().try_fill_bytes(bytes.as_mut())?;
        Ok(Self(bytes))
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
