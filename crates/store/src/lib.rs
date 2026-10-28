#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

//! Durable storage for notes, in two SQLite files.
//!
//! # The two files
//!
//! `notes.db` holds sealed payloads. `keys.db` holds the 32-byte key that
//! opens each one. They are separate files because burning a note means
//! destroying its key, and destroying 32 bytes in a small file is something
//! SQLite can plausibly finish -- scrub the b-tree page with
//! `PRAGMA secure_delete`, truncate the write-ahead log, let
//! `PRAGMA auto_vacuum=FULL` shorten the file. Overwriting a 64 KB blob on an
//! SSD is not something anyone can prove, so the design does not try.
//!
//! Ciphertext lingering in the note file's log is harmless; without the key it
//! is noise. That asymmetry is why the two files are configured differently.
//!
//! # What is not atomic, and why that is fine
//!
//! A burn touches both files, and SQLite's documentation is explicit that this
//! cannot be done atomically: transactions across attached databases "are
//! atomic for each individual database, but are not atomic across all
//! databases as a set" in WAL mode. So a burn is two atomic steps, ordered so
//! that the window between them is harmless:
//!
//! 1. Take the note row. Atomic within `notes.db`: a second caller arriving at
//!    the same instant finds nothing.
//! 2. Take the key. Atomic within `keys.db`.
//!
//! A crash in between leaves a key with no ciphertext to open -- useless, and
//! destroyed on its own deadline, because each key row carries a copy of its
//! note's expiry. Creation runs the same two writes in the opposite order, for
//! the same reason: a crash must never leave a note whose key never arrived.
//!
//! # One error for three states
//!
//! A note that never existed, a note that has expired and a note that has
//! already been read all produce [`StoreError::Unavailable`], from one branch
//! of one function. Not three errors that happen to be formatted alike today:
//! the caller is given no way to tell them apart, so no caller can leak the
//! difference.

mod db;
mod keys;
mod notes;
mod schema;
mod time;

pub use crate::time::UnixTime;
// Re-exported because it appears in this crate's public API, and a caller
// should not have to name the crypto crate to hold a note id.
pub use blindnote_crypto::Id;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blindnote_crypto::{CryptoError, NoteKey};
use rusqlite::Connection;

use crate::db::WalPolicy;

/// A note to be stored.
#[derive(Debug)]
pub struct NewNote {
    /// What the browser produced. This crate cannot read it and never tries.
    pub client_ciphertext: Vec<u8>,
    /// When the note stops being served and becomes sweepable.
    pub expires_at: UnixTime,
    /// How many times it may be read. One, in this version.
    pub views_left: u32,
}

/// What one sweep removed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SweepReport {
    /// Note rows deleted.
    pub notes: usize,
    /// Key rows destroyed.
    pub keys: usize,
}

/// Why a storage operation failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The note does not exist, has expired, or has already been read.
    ///
    /// Deliberately one variant. See the crate documentation.
    #[error("the note is not available")]
    Unavailable,

    /// The data directory could not be created or secured.
    #[error("the data directory {path} could not be prepared: {source}")]
    DataDir {
        /// The directory that was being prepared.
        path: PathBuf,
        /// What the operating system said.
        #[source]
        source: io::Error,
    },

    /// A database file was written by a newer build than this one.
    #[error(
        "the {database} database is at schema version {found}, newer than this binary knows ({known})"
    )]
    SchemaTooNew {
        /// Which of the two files.
        database: &'static str,
        /// Version found in the file.
        found: i32,
        /// Highest version this binary can produce.
        known: i32,
    },

    /// A setting this project's guarantees rely on did not take effect.
    #[error("PRAGMA {pragma} is {found}, expected {wanted}")]
    Pragma {
        /// Which pragma.
        pragma: &'static str,
        /// The value the code asked for.
        wanted: String,
        /// The value the database reported.
        found: String,
    },

    /// A note was offered with no views, which nobody could ever read.
    ///
    /// Refused at the door rather than stored: a note that cannot be opened
    /// is a caller's bug, and silently accepting it would surface later as a
    /// user's note mysteriously not being there.
    #[error("a note must allow at least one view")]
    NoViews,

    /// A freshly generated identifier was already in use.
    ///
    /// With 128 random bits this means the random number generator is broken,
    /// not that a retry would help.
    #[error("a generated identifier collided with an existing one")]
    IdCollision,

    /// SQLite refused.
    #[error("sqlite failed: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A cryptographic operation failed, or a stored value had a bad length.
    #[error("crypto failed: {0}")]
    Crypto(#[from] CryptoError),

    /// A blocking database task panicked.
    #[error("a database task panicked")]
    Panicked,
}

/// Handle on both database files. Cheap to clone.
#[derive(Clone, Debug)]
pub struct Store {
    inner: Arc<Inner>,
}

/// The two writer connections.
///
/// One writer per file, with no reader pool. SQLite is synchronous, so every
/// call goes through [`tokio::task::spawn_blocking`]; the serialisation that
/// costs is the `fsync` under `synchronous = FULL`, not the mutex. A reader
/// pool is what WAL mode is for and will be worth adding the moment there is a
/// read-only code path -- at present there is none, because creating, reading
/// and reporting a note are all writes.
#[derive(Debug)]
struct Inner {
    notes: Mutex<Connection>,
    keys: Mutex<Connection>,
}

impl Store {
    /// Opens, configures and migrates both files under `data_dir`.
    ///
    /// The directory is created if missing and restricted to the owner. The
    /// `-wal` and `-shm` files SQLite creates alongside each database inherit
    /// the process umask, which is why the directory mode is the protection
    /// that matters rather than the file modes.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::DataDir`] if the directory cannot be prepared,
    /// [`StoreError::Pragma`] if a required setting did not take effect,
    /// [`StoreError::SchemaTooNew`] if a file came from a newer build, and
    /// [`StoreError::Sqlite`] for anything else SQLite refuses.
    pub async fn open(data_dir: &Path) -> Result<Self, StoreError> {
        let data_dir = data_dir.to_path_buf();

        tokio::task::spawn_blocking(move || Self::open_blocking(&data_dir))
            .await
            .map_err(|_| StoreError::Panicked)?
    }

    /// The file work behind [`Store::open`].
    fn open_blocking(data_dir: &Path) -> Result<Self, StoreError> {
        prepare_directory(data_dir)?;

        let notes_path = data_dir.join("notes.db");
        let keys_path = data_dir.join("keys.db");
        let keys_are_new = !keys_path.exists();

        let mut notes = db::open(&notes_path, WalPolicy::Relaxed, false)?;
        let mut keys = db::open(&keys_path, WalPolicy::Minimal, keys_are_new)?;

        schema::migrate_notes(&mut notes)?;
        schema::migrate_keys(&mut keys)?;

        Ok(Self {
            inner: Arc::new(Inner {
                notes: Mutex::new(notes),
                keys: Mutex::new(keys),
            }),
        })
    }

    /// Seals a note, stores it, and returns the id the link has to carry.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::NoViews`] if the note allows no reads,
    /// [`StoreError::Crypto`] if randomness or sealing fails, and
    /// [`StoreError::Sqlite`] if either write fails. A failed note write is
    /// compensated by deleting the key that was just stored, so a rejected
    /// create leaves nothing behind.
    pub async fn create(&self, note: NewNote) -> Result<Id, StoreError> {
        if note.views_left == 0 {
            return Err(StoreError::NoViews);
        }

        let note_id = Id::random()?;
        let key_id = Id::random()?;
        let key = NoteKey::random()?;
        let sealed = blindnote_crypto::seal(&key, &note_id, &key_id, &note.client_ciphertext)?;

        let expires_at = note.expires_at;
        let views_left = note.views_left;

        // The key lands first. A crash between the two writes then leaves a
        // key with nothing to open, which is useless and is destroyed on its
        // own deadline. The other order would leave a note whose key never
        // arrived: permanently unreadable, for a user who was told it was
        // created.
        self.with_keys(move |conn| keys::insert(conn, &key_id, &key, expires_at))
            .await?;

        let stored = self
            .with_notes(move |conn| {
                notes::insert(
                    conn,
                    &note_id,
                    &key_id,
                    &sealed.blob,
                    &sealed.nonce,
                    expires_at,
                    views_left,
                )
            })
            .await;

        if stored.is_err() {
            // Compensate now rather than leave it to the sweeper. If this
            // fails too, the deadline still applies.
            if let Err(error) = self
                .with_keys(move |conn| keys::delete(conn, &key_id))
                .await
            {
                tracing::error!(%error, "could not roll back the key of a rejected note");
            }
        }
        stored?;

        Ok(note_id)
    }

    /// Burns a note and returns the ciphertext the browser has to decrypt.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Unavailable`] when the note does not exist, has
    /// expired, or has already been read -- the three are indistinguishable by
    /// construction.
    pub async fn burn(&self, id: Id, now: UnixTime) -> Result<Vec<u8>, StoreError> {
        let taken = self
            .with_notes(move |conn| notes::take(conn, &id, now))
            .await?;
        let key_id = taken.key_id;

        let key = if taken.was_final {
            self.with_keys(move |conn| keys::take(conn, &key_id))
                .await?
        } else {
            self.with_keys(move |conn| keys::read(conn, &key_id))
                .await?
        };

        Ok(blindnote_crypto::open(
            &key,
            &id,
            &key_id,
            &taken.nonce,
            &taken.blob,
        )?)
    }

    /// Destroys a note on an abuse report, without decrypting anything.
    ///
    /// Succeeds whether or not the note existed. The report path must not
    /// become the existence oracle the read path is careful not to be.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Sqlite`] if a statement fails. Never
    /// [`StoreError::Unavailable`].
    pub async fn report(&self, id: Id) -> Result<(), StoreError> {
        let key_id = self
            .with_notes(move |conn| notes::delete(conn, &id))
            .await?;

        if let Some(key_id) = key_id {
            self.with_keys(move |conn| keys::delete(conn, &key_id))
                .await?;
        }

        Ok(())
    }

    /// Deletes everything whose deadline has passed.
    ///
    /// Keys first: that is the deletion that actually destroys a note. If the
    /// process dies in between, what is left is a note nobody can read, which
    /// the next sweep removes.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Sqlite`] if a statement fails.
    pub async fn sweep(&self, now: UnixTime) -> Result<SweepReport, StoreError> {
        let keys = self
            .with_keys(move |conn| keys::delete_expired(conn, now))
            .await?;
        let notes = self
            .with_notes(move |conn| notes::delete_expired(conn, now))
            .await?;

        Ok(SweepReport { notes, keys })
    }

    /// Runs a closure against the note file, off the async runtime.
    async fn with_notes<T, F>(&self, work: F) -> Result<T, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
        T: Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || work(&mut lock(&inner.notes)))
            .await
            .map_err(|_| StoreError::Panicked)?
    }

    /// Runs a closure against the key file, off the async runtime.
    async fn with_keys<T, F>(&self, work: F) -> Result<T, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
        T: Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || work(&mut lock(&inner.keys)))
            .await
            .map_err(|_| StoreError::Panicked)?
    }
}

/// Locks a connection, recovering from a poisoned mutex.
///
/// A poisoned mutex means a previous holder panicked. The connection itself is
/// still usable: any transaction it held was rolled back when the guard was
/// dropped, which is SQLite's behaviour and not something this code has to
/// arrange. Refusing to serve from then on would turn one panic into an
/// outage, and these are users' notes.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Creates the data directory if needed and restricts it to its owner.
fn prepare_directory(data_dir: &Path) -> Result<(), StoreError> {
    let fail = |source: io::Error| StoreError::DataDir {
        path: data_dir.to_path_buf(),
        source,
    };

    std::fs::create_dir_all(data_dir).map_err(fail)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(data_dir, std::fs::Permissions::from_mode(0o700)).map_err(fail)?;
    }

    Ok(())
}
