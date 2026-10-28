//! 128-bit random identifiers.

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use subtle::{Choice, ConstantTimeEq};

use crate::CryptoError;

/// Length of an identifier in bytes.
///
/// 128 bits. The note id is one half of the secret in a link -- the other half
/// being the key -- so it has to be unguessable, not merely unique.
pub const ID_BYTES: usize = 16;

/// A 128-bit random identifier: a note id, or a key id.
///
/// Two distinct uses share the type because they have identical requirements.
/// A note id is what the client sends to fetch or burn a note. A key id is the
/// handle that connects a row in `notes` to a row in `note_keys`, and it is
/// deliberately *not* the note id: that way the key file holds no note
/// identifiers at all.
#[derive(Clone, Copy)]
pub struct Id([u8; ID_BYTES]);

impl Id {
    /// Draws a new identifier from the operating system's CSPRNG.
    ///
    /// There is no userspace PRNG in this crate on purpose: a reseeding
    /// generator is state that has to be kept, protected and zeroized, and at
    /// this call volume it buys nothing over asking the kernel.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Random`] if the operating system refuses.
    pub fn random() -> Result<Self, CryptoError> {
        let mut bytes = [0u8; ID_BYTES];
        getrandom::fill(&mut bytes).map_err(CryptoError::Random)?;
        Ok(Self(bytes))
    }

    /// Reads an identifier from exactly [`ID_BYTES`] bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Length`] for any other length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; ID_BYTES] = bytes.try_into().map_err(|_| CryptoError::Length {
            expected: ID_BYTES,
            actual: bytes.len(),
        })?;
        Ok(Self(bytes))
    }

    /// Borrows the raw bytes, for binding into a SQL statement.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; ID_BYTES] {
        &self.0
    }

    /// Renders the identifier as unpadded base64url: 22 characters.
    ///
    /// Unpadded because the `=` of padded base64 is reserved in a query string
    /// and ugly in a URL fragment, and the length is fixed anyway.
    #[must_use]
    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    /// Parses an identifier from unpadded base64url.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Encoding`] if the string is not base64url, and
    /// [`CryptoError::Length`] if it decodes to the wrong number of bytes.
    pub fn from_base64url(encoded: &str) -> Result<Self, CryptoError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| CryptoError::Encoding)?;
        Self::from_slice(&bytes)
    }
}

impl ConstantTimeEq for Id {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl PartialEq for Id {
    /// Compares in constant time.
    ///
    /// A SQLite index lookup on an id is not constant time and cannot be made
    /// so, which is noted honestly in the store's documentation. That is not a
    /// reason for the comparisons we *do* control to leak a prefix length.
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for Id {}

impl fmt::Debug for Id {
    /// Prints a placeholder, never the identifier.
    ///
    /// A note id must never be written to disk, and a log line is disk. This
    /// is the rule made structural: `tracing::debug!(?id)` prints nothing.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Id(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use super::{ID_BYTES, Id};
    use crate::CryptoError;

    #[test]
    fn random_ids_differ() {
        let one = Id::random().unwrap();
        let two = Id::random().unwrap();

        assert_ne!(one, two, "two draws from the CSPRNG collided");
    }

    #[test]
    fn base64url_round_trips() {
        let id = Id::random().unwrap();

        let encoded = id.to_base64url();
        assert_eq!(
            encoded.len(),
            22,
            "16 bytes is 22 unpadded base64url characters"
        );
        assert_eq!(Id::from_base64url(&encoded).unwrap(), id);
    }

    #[test]
    fn base64url_rejects_padding_and_other_alphabets() {
        assert!(matches!(
            Id::from_base64url("AAAAAAAAAAAAAAAAAAAAAA=="),
            Err(CryptoError::Encoding)
        ));
        assert!(matches!(
            Id::from_base64url("not base64!"),
            Err(CryptoError::Encoding)
        ));
    }

    #[test]
    fn from_slice_rejects_the_wrong_length() {
        let error = Id::from_slice(&[0u8; ID_BYTES - 1]).expect_err("rejected");

        assert!(matches!(
            error,
            CryptoError::Length {
                expected: ID_BYTES,
                actual: 15
            }
        ));
    }

    #[test]
    fn debug_never_prints_the_identifier() {
        let id = Id::random().unwrap();

        let rendered = format!("{id:?}");
        assert_eq!(rendered, "Id(redacted)");
        assert!(!rendered.contains(&id.to_base64url()));
    }
}
