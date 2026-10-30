#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, reason = "unwrap is acceptable in test code")
)]

//! Identifiers, per-note keys and the at-rest AEAD layer.
//!
//! # What this crate is not
//!
//! This crate never sees a note's plaintext, and neither does any other crate
//! in this workspace. The browser encrypts the note before it is sent and the
//! key travels in the URL fragment, which is never part of an HTTP request.
//! What arrives here is already ciphertext.
//!
//! # Why there is a second layer of encryption at all
//!
//! Wrapping the client's ciphertext in `AES-256-GCM(k_note, ...)` looks
//! redundant -- `k_note` lives on the same disk, in the other SQLite file. It
//! is not there to keep an attacker with the disk out. It is there to make
//! *deletion cheap*: destroying 32 bytes in a small file is something SQLite's
//! `secure_delete` plus a WAL truncate can plausibly finish, while overwriting
//! a 64 KB blob on an SSD is not something anyone can prove. Burning a note is
//! deleting its key, and this layer is what makes that equivalent to
//! destroying the note.
//!
//! The claim is cryptographic, never forensic: "the key is gone", never "the
//! bytes were physically erased".
//!
//! # Debug output is redacted on purpose
//!
//! [`Id`] and [`NoteKey`] both implement [`Debug`](std::fmt::Debug) by printing
//! a placeholder. A note id is half of the secret in the link and must never
//! reach a log; a key is the whole secret. Redacting at the type level means
//! that a later `tracing::debug!(?note_id)` leaks nothing, instead of relying
//! on every author remembering the rule.

mod at_rest;
mod id;
mod key;

pub use crate::at_rest::{Sealed, open, seal};
pub use crate::id::{ID_BYTES, Id};
pub use crate::key::{KEY_BYTES, NONCE_BYTES, Nonce, NoteKey};

/// Why a cryptographic operation failed.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// The operating system's random number generator failed.
    ///
    /// Fatal rather than retryable: a server that cannot get randomness must
    /// not fall back to anything, and must not pretend it produced a key.
    #[error("the operating system random number generator failed: {0}")]
    Random(#[source] getrandom::Error),

    /// A byte string had the wrong length for the type it was being read into.
    #[error("expected {expected} bytes, got {actual}")]
    Length {
        /// How many bytes the type requires.
        expected: usize,
        /// How many bytes were offered.
        actual: usize,
    },

    /// A string was not valid unpadded base64url.
    #[error("not valid base64url")]
    Encoding,

    /// Sealing failed.
    ///
    /// In practice this means the payload was longer than AES-GCM permits.
    #[error("sealing failed")]
    Seal,

    /// The sealed blob did not authenticate.
    ///
    /// Deliberately carries nothing: not which check failed, not how far it
    /// got, not the length involved. Every authentication failure has to look
    /// the same from the outside.
    #[error("the sealed blob did not authenticate")]
    Open,
}
