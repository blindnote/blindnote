//! Does burning a note actually remove the key's bytes from the disk?
//!
//! This is the measurement the project calls for by name: after a burn and a
//! `TRUNCATE` checkpoint, byte-scan the raw files and check that the 32 bytes
//! are gone. It is the closest thing to a direct test of the central claim,
//! and it is written as an outside observer with access to the disk -- it
//! opens the database file itself rather than asking the store anything.
//!
//! What this does *not* prove is that the bytes left the physical medium. On
//! an SSD with wear levelling that is unprovable, and the project's claim is
//! deliberately the cryptographic one: the key is gone from the file.

mod common;

use std::path::{Path, PathBuf};

use blindnote_store::UnixTime;
use common::{Fixture, expired_note, note};
use rusqlite::Connection;

/// Reads the single stored key by opening the key file directly.
///
/// The test has to know the bytes it is looking for, and the store does not
/// hand them out -- which is the right design. Reading them out of the file is
/// exactly what an attacker with the disk would do.
fn read_the_only_key(path: &Path) -> Vec<u8> {
    let conn = Connection::open(path).expect("the key file opens");

    conn.query_row("SELECT k_note FROM note_keys", [], |row| {
        row.get::<_, Vec<u8>>(0)
    })
    .expect("exactly one key is stored")
}

/// Every file in the data directory that contains `needle` anywhere in it.
fn files_containing(directory: &Path, needle: &[u8]) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(directory).expect("the data directory is readable");

    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            let contents = std::fs::read(path).expect("a readable file");
            contents
                .windows(needle.len())
                .any(|window| window == needle)
        })
        .collect()
}

#[tokio::test]
async fn the_key_is_on_disk_before_the_burn() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    f.store.create(note(now)).await.expect("stored");

    let key = read_the_only_key(&f.path("keys.db"));
    let found = files_containing(f.directory(), &key);

    assert!(
        !found.is_empty(),
        "the control failed: a stored key should be findable on disk, \
         so a later test finding nothing would prove nothing"
    );
}

#[tokio::test]
async fn burning_removes_the_key_bytes_from_every_file() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");
    let key = read_the_only_key(&f.path("keys.db"));
    assert!(!files_containing(f.directory(), &key).is_empty(), "control");

    f.store.burn(id, now).await.expect("burned");

    let survivors = files_containing(f.directory(), &key);
    assert!(
        survivors.is_empty(),
        "the key survived the burn in {survivors:?}"
    );
}

#[tokio::test]
async fn sweeping_removes_the_key_bytes_from_every_file() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    f.store.create(expired_note(now)).await.expect("stored");
    let key = read_the_only_key(&f.path("keys.db"));
    assert!(!files_containing(f.directory(), &key).is_empty(), "control");

    let report = f.store.sweep(now).await.expect("swept");
    assert_eq!(report.keys, 1);

    let survivors = files_containing(f.directory(), &key);
    assert!(
        survivors.is_empty(),
        "the key survived the sweep in {survivors:?}"
    );
}

/// A note that is never read must not outlive its deadline on disk either.
///
/// Without this, "self-destructing" would be false for exactly the case users
/// worry about: the note nobody opened.
#[tokio::test]
async fn an_unread_expired_note_is_removed_by_the_sweep() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    f.store.create(expired_note(now)).await.expect("stored");

    let report = f.store.sweep(now).await.expect("swept");

    assert_eq!(report.notes, 1);
    assert_eq!(report.keys, 1);
}

/// The key file must never learn which note a key belongs to.
///
/// Holding the key file alone should tell you how many notes are pending and
/// nothing else. If a note id ever appeared in it, the two files would stop
/// being independent and the separation would buy less than it claims.
#[tokio::test]
async fn the_key_file_contains_no_note_ids() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let id = f.store.create(note(now)).await.expect("stored");

    for path in f.key_files() {
        let contents = std::fs::read(&path).expect("a readable file");
        let found = contents
            .windows(id.as_bytes().len())
            .any(|window| window == id.as_bytes());

        assert!(!found, "{} contains the note id", path.display());
    }
}
