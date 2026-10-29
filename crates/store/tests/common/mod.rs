#![allow(dead_code, unreachable_pub)]

//! Shared fixture for the store's integration tests.
//!
//! The tests only ever touch the public API, plus the raw database files when
//! the point of the test is what an observer with the disk would see.

use std::path::PathBuf;

use blindnote_store::{FreeSpaceFloor, NewNote, Store, UnixTime};
use tempfile::TempDir;

/// The default time-to-live, matching the product's default of 24 hours.
pub const TTL_SECONDS: u32 = 86_400;

/// Stands in for whatever the browser encrypted. The store cannot read it.
pub const CIPHERTEXT: &[u8] = b"ciphertext the server has no key for";

/// A store on a throwaway directory, which is removed when this is dropped.
pub struct Fixture {
    /// The store under test.
    pub store: Store,
    directory: TempDir,
}

impl Fixture {
    /// Opens a fresh store in a new temporary directory.
    pub async fn new() -> Self {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let store = Store::open(directory.path(), FreeSpaceFloor::NONE)
            .await
            .expect("the store opens");

        Self { store, directory }
    }

    /// Re-opens the same directory with a new store, as a restart would.
    pub async fn reopen(&self) -> Store {
        Store::open(self.directory.path(), FreeSpaceFloor::NONE)
            .await
            .expect("the store re-opens")
    }

    /// Path of the directory holding both database files.
    pub fn directory(&self) -> &std::path::Path {
        self.directory.path()
    }

    /// Path of a file inside the data directory.
    pub fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    /// Every file SQLite has created for the key database.
    ///
    /// The main file, its write-ahead log and its shared-memory index. All
    /// three are listed because the question these tests ask is about bytes on
    /// a disk, not about rows in a table.
    pub fn key_files(&self) -> Vec<PathBuf> {
        ["keys.db", "keys.db-wal", "keys.db-shm"]
            .iter()
            .map(|name| self.path(name))
            .filter(|path| path.exists())
            .collect()
    }
}

/// A note with the default TTL, expiring relative to `now`.
pub fn note(now: UnixTime) -> NewNote {
    NewNote {
        client_ciphertext: CIPHERTEXT.to_vec(),
        expires_at: now.plus_seconds(TTL_SECONDS),
        views_left: 1,
    }
}

/// A note that may be read `views` times.
pub fn note_with_views(now: UnixTime, views: u32) -> NewNote {
    NewNote {
        views_left: views,
        ..note(now)
    }
}

/// A note that expired a minute ago.
pub fn expired_note(now: UnixTime) -> NewNote {
    NewNote {
        expires_at: UnixTime::from_seconds(now.as_seconds() - 60),
        ..note(now)
    }
}
