//! The at-rest AEAD layer: `AES-256-GCM(k_note, client_ciphertext)`.
//!
//! The payload this module seals is *already* ciphertext, produced in the
//! browser with a key this server has never seen. Sealing it again under
//! `k_note` is what makes burning a note cheap: the whole note becomes
//! unrecoverable once 32 bytes are deleted from a small file, with no need to
//! overwrite a 64 KB blob on an SSD.

use aes_gcm::aead::{Aead as _, KeyInit as _, Payload};
use aes_gcm::{Aes256Gcm, Nonce as GcmNonce};

use crate::{CryptoError, Id, Nonce, NoteKey};

/// Version byte prefixed to the associated data.
///
/// Changing the sealing scheme means changing this byte, which makes every
/// existing blob fail to authenticate rather than be misinterpreted under new
/// rules. Notes live at most 24 hours, so a scheme change needs no migration:
/// it needs one day of both versions being accepted, or a day of refusals.
const AAD_VERSION: u8 = 1;

/// Length of the associated data: a version byte plus two identifiers.
const AAD_BYTES: usize = 1 + 2 * crate::ID_BYTES;

/// A sealed blob together with the nonce it was sealed under.
#[derive(Debug)]
pub struct Sealed {
    /// `AES-256-GCM(k_note, nonce, client_ciphertext)`, tag appended.
    pub blob: Vec<u8>,
    /// The nonce, which the row stores alongside the blob.
    pub nonce: Nonce,
}

/// Builds the associated data that binds a blob to its row.
///
/// Both identifiers go in, so a blob cannot be moved to another note's row and
/// cannot be paired with another row's key: either substitution changes the
/// associated data and the tag stops verifying. Without this, a database with
/// write access would allow swapping rows around undetectably.
fn associated_data(note_id: &Id, key_id: &Id) -> [u8; AAD_BYTES] {
    let mut aad = [0u8; AAD_BYTES];
    aad[0] = AAD_VERSION;
    aad[1..=crate::ID_BYTES].copy_from_slice(note_id.as_bytes());
    aad[1 + crate::ID_BYTES..].copy_from_slice(key_id.as_bytes());
    aad
}

/// Seals a client ciphertext under a per-note key, drawing a fresh nonce.
///
/// # Errors
///
/// Returns [`CryptoError::Random`] if a nonce cannot be drawn, and
/// [`CryptoError::Seal`] if AES-GCM refuses the payload -- in practice only
/// when it is longer than the cipher permits, which the HTTP layer rejects
/// long before this point.
pub fn seal(
    key: &NoteKey,
    note_id: &Id,
    key_id: &Id,
    client_ciphertext: &[u8],
) -> Result<Sealed, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.expose_secret()).map_err(|_| CryptoError::Seal)?;
    let nonce = Nonce::random()?;
    let aad = associated_data(note_id, key_id);
    let gcm_nonce = GcmNonce::from(*nonce.as_bytes());

    let blob = cipher
        .encrypt(
            &gcm_nonce,
            Payload {
                msg: client_ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Seal)?;

    Ok(Sealed { blob, nonce })
}

/// Opens a sealed blob, recovering the client ciphertext.
///
/// # Errors
///
/// Returns [`CryptoError::Open`] if the blob does not authenticate under this
/// key, nonce and row. The error says nothing about which part was wrong.
pub fn open(
    key: &NoteKey,
    note_id: &Id,
    key_id: &Id,
    nonce: &Nonce,
    blob: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.expose_secret()).map_err(|_| CryptoError::Open)?;
    let aad = associated_data(note_id, key_id);
    let gcm_nonce = GcmNonce::from(*nonce.as_bytes());

    cipher
        .decrypt(
            &gcm_nonce,
            Payload {
                msg: blob,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Open)
}

#[cfg(test)]
mod tests {
    use super::{open, seal};
    use crate::{CryptoError, Id, Nonce, NoteKey};

    /// Stands in for whatever the browser produced. This layer never looks
    /// inside it, so any bytes will do.
    const CLIENT_CIPHERTEXT: &[u8] = b"opaque bytes the server cannot read";

    struct Fixture {
        key: NoteKey,
        note_id: Id,
        key_id: Id,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                key: NoteKey::random().unwrap(),
                note_id: Id::random().unwrap(),
                key_id: Id::random().unwrap(),
            }
        }
    }

    #[test]
    fn a_sealed_blob_opens_again() {
        let f = Fixture::new();

        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();
        let opened = open(&f.key, &f.note_id, &f.key_id, &sealed.nonce, &sealed.blob).unwrap();

        assert_eq!(opened, CLIENT_CIPHERTEXT);
    }

    #[test]
    fn the_blob_is_not_the_payload() {
        let f = Fixture::new();

        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        assert_ne!(sealed.blob, CLIENT_CIPHERTEXT);
        assert_eq!(
            sealed.blob.len(),
            CLIENT_CIPHERTEXT.len() + 16,
            "AES-GCM appends a 16-byte tag and does not pad"
        );
    }

    #[test]
    fn sealing_twice_gives_different_blobs() {
        let f = Fixture::new();

        let one = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();
        let two = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        assert_ne!(
            one.nonce.as_bytes(),
            two.nonce.as_bytes(),
            "the nonce must be fresh"
        );
        assert_ne!(one.blob, two.blob);
    }

    #[test]
    fn a_wrong_key_does_not_open_it() {
        let f = Fixture::new();
        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        let other = NoteKey::random().unwrap();
        let error =
            open(&other, &f.note_id, &f.key_id, &sealed.nonce, &sealed.blob).expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn a_blob_cannot_be_moved_to_another_note_id() {
        let f = Fixture::new();
        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        let other_note = Id::random().unwrap();
        let error = open(&f.key, &other_note, &f.key_id, &sealed.nonce, &sealed.blob)
            .expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn a_blob_cannot_be_paired_with_another_key_id() {
        let f = Fixture::new();
        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        let other_key_id = Id::random().unwrap();
        let error = open(
            &f.key,
            &f.note_id,
            &other_key_id,
            &sealed.nonce,
            &sealed.blob,
        )
        .expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn a_wrong_nonce_does_not_open_it() {
        let f = Fixture::new();
        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        let other_nonce = Nonce::random().unwrap();
        let error =
            open(&f.key, &f.note_id, &f.key_id, &other_nonce, &sealed.blob).expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn a_flipped_bit_is_detected() {
        let f = Fixture::new();
        let mut sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        sealed.blob[0] ^= 0x01;
        let error =
            open(&f.key, &f.note_id, &f.key_id, &sealed.nonce, &sealed.blob).expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn a_truncated_tag_is_detected() {
        let f = Fixture::new();
        let sealed = seal(&f.key, &f.note_id, &f.key_id, CLIENT_CIPHERTEXT).unwrap();

        let truncated = &sealed.blob[..sealed.blob.len() - 1];
        let error =
            open(&f.key, &f.note_id, &f.key_id, &sealed.nonce, truncated).expect_err("rejected");

        assert!(matches!(error, CryptoError::Open));
    }

    #[test]
    fn an_empty_payload_still_round_trips() {
        let f = Fixture::new();

        let sealed = seal(&f.key, &f.note_id, &f.key_id, b"").unwrap();
        let opened = open(&f.key, &f.note_id, &f.key_id, &sealed.nonce, &sealed.blob).unwrap();

        assert_eq!(opened, Vec::<u8>::new());
    }
}
