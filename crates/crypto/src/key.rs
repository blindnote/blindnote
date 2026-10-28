//! The per-note key and the at-rest nonce.

use std::fmt;

use zeroize::Zeroize as _;

use crate::CryptoError;

/// Length of a per-note key in bytes: 256 bits, for AES-256-GCM.
pub const KEY_BYTES: usize = 32;

/// Length of an at-rest nonce in bytes: 96 bits, the AES-GCM standard size.
pub const NONCE_BYTES: usize = 12;

/// The 32 bytes whose deletion burns a note.
///
/// # Why the bytes live on the heap
///
/// Moving a `[u8; 32]` held inline copies the bytes to the new location and
/// leaves the old copy behind untouched, because Rust does not run a
/// destructor on a moved-from value. Zeroizing on drop would then scrub one
/// copy and miss every intermediate. Holding the bytes behind a `Box` means a
/// move copies the pointer and nothing else, so there is exactly one copy to
/// scrub, and the constructors write random bytes straight into the allocation
/// without ever materialising them on the stack.
///
/// This is not a complete defence, and it should not be described as one.
/// [`from_slice`](Self::from_slice) copies out of a buffer somebody else owns,
/// the kernel may hold the page in a cache, and a core dump would contain
/// everything -- which is why core dumps and swap are both off on the host.
/// The guarantee is narrow and worth stating narrowly: the key is not left
/// lying around in this process's own memory.
pub struct NoteKey(Box<[u8; KEY_BYTES]>);

impl NoteKey {
    /// Draws a new key from the operating system's CSPRNG.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Random`] if the operating system refuses. There
    /// is no fallback: a note sealed with a key of unknown quality is a worse
    /// outcome than a refused request.
    pub fn random() -> Result<Self, CryptoError> {
        let mut bytes = Box::new([0u8; KEY_BYTES]);
        getrandom::fill(bytes.as_mut_slice()).map_err(CryptoError::Random)?;
        Ok(Self(bytes))
    }

    /// Reads a key from exactly [`KEY_BYTES`] bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Length`] for any other length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != KEY_BYTES {
            return Err(CryptoError::Length {
                expected: KEY_BYTES,
                actual: bytes.len(),
            });
        }

        let mut boxed = Box::new([0u8; KEY_BYTES]);
        boxed.copy_from_slice(bytes);
        Ok(Self(boxed))
    }

    /// Borrows the key material.
    ///
    /// Named for what it does. Every call site is a place where 32 secret
    /// bytes become visible, and those places should be greppable.
    #[must_use]
    pub fn expose_secret(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }
}

impl Drop for NoteKey {
    /// Overwrites the key material.
    ///
    /// Hand-written rather than derived, so that a reader auditing the central
    /// claim of this project can see the scrub without expanding a macro.
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for NoteKey {
    /// Prints a placeholder, never the key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NoteKey(redacted)")
    }
}

/// A 96-bit nonce for the at-rest AEAD.
///
/// Random per note rather than a counter. A counter would need durable state
/// that survives restarts and replication, and getting that wrong reuses a
/// nonce, which is the one mistake AES-GCM does not tolerate. At 96 bits of
/// randomness per note, with each nonce used under a different key, reuse is
/// not a practical concern.
#[derive(Clone, Copy)]
pub struct Nonce([u8; NONCE_BYTES]);

impl Nonce {
    /// Draws a new nonce from the operating system's CSPRNG.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Random`] if the operating system refuses.
    pub fn random() -> Result<Self, CryptoError> {
        let mut bytes = [0u8; NONCE_BYTES];
        getrandom::fill(&mut bytes).map_err(CryptoError::Random)?;
        Ok(Self(bytes))
    }

    /// Reads a nonce from exactly [`NONCE_BYTES`] bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Length`] for any other length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; NONCE_BYTES] = bytes.try_into().map_err(|_| CryptoError::Length {
            expected: NONCE_BYTES,
            actual: bytes.len(),
        })?;
        Ok(Self(bytes))
    }

    /// Borrows the raw bytes, for binding into a SQL statement.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; NONCE_BYTES] {
        &self.0
    }
}

impl fmt::Debug for Nonce {
    /// Prints a placeholder.
    ///
    /// A nonce is not secret, but it is per-note, so a logged nonce is a
    /// correlation handle for a note nobody is supposed to be able to point at.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Nonce(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use super::{KEY_BYTES, NONCE_BYTES, Nonce, NoteKey};
    use crate::CryptoError;

    #[test]
    fn random_keys_differ() {
        let one = NoteKey::random().unwrap();
        let two = NoteKey::random().unwrap();

        assert_ne!(
            one.expose_secret(),
            two.expose_secret(),
            "two draws collided"
        );
    }

    #[test]
    fn a_key_round_trips_through_bytes() {
        let key = NoteKey::random().unwrap();

        let copy = NoteKey::from_slice(key.expose_secret()).unwrap();

        assert_eq!(copy.expose_secret(), key.expose_secret());
    }

    #[test]
    fn from_slice_rejects_the_wrong_key_length() {
        let error = NoteKey::from_slice(&[0u8; KEY_BYTES + 1]).expect_err("rejected");

        assert!(matches!(
            error,
            CryptoError::Length {
                expected: KEY_BYTES,
                actual: 33
            }
        ));
    }

    #[test]
    fn debug_never_prints_key_material() {
        let key = NoteKey::random().unwrap();

        assert_eq!(format!("{key:?}"), "NoteKey(redacted)");
    }

    #[test]
    fn random_nonces_differ() {
        let one = Nonce::random().unwrap();
        let two = Nonce::random().unwrap();

        assert_ne!(one.as_bytes(), two.as_bytes(), "two draws collided");
    }

    #[test]
    fn from_slice_rejects_the_wrong_nonce_length() {
        let error = Nonce::from_slice(&[0u8; NONCE_BYTES - 1]).expect_err("rejected");

        assert!(matches!(
            error,
            CryptoError::Length {
                expected: NONCE_BYTES,
                actual: 11
            }
        ));
    }

    #[test]
    fn debug_never_prints_the_nonce() {
        let nonce = Nonce::random().unwrap();

        assert_eq!(format!("{nonce:?}"), "Nonce(redacted)");
    }
}
