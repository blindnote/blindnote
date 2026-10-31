//! Opening, re-opening, and refusing to open.

mod common;

use blindnote_store::{FreeSpaceFloor, Store, StoreError, UnixTime};
use common::{CIPHERTEXT, Fixture, note};
use rusqlite::Connection;

/// Re-opening must not re-run migrations or disturb what is there.
#[tokio::test]
async fn opening_twice_is_idempotent() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let again = f.reopen().await;

    let id = again.create(note(now)).await.expect("stored");
    assert_eq!(again.burn(id, now).await.expect("burned"), CIPHERTEXT);
}

#[tokio::test]
async fn the_data_directory_is_created_if_missing() {
    let parent = tempfile::tempdir().expect("a temporary directory");
    let nested = parent.path().join("does").join("not").join("exist");

    let store = Store::open(&nested, FreeSpaceFloor::NONE)
        .await
        .expect("the store creates it");

    let now = UnixTime::now();
    store.create(note(now)).await.expect("stored");
    assert!(nested.join("notes.db").exists());
    assert!(nested.join("keys.db").exists());
}

/// Both files carry a schema version, and both are at version 1.
#[tokio::test]
async fn both_files_record_their_schema_version() {
    let f = Fixture::new().await;

    // notes.db is at its third migration (counters, then the daily stats and
    // the unique-visitor sketch tables); keys.db is still at its first.
    for (name, expected) in [("notes.db", 3), ("keys.db", 1)] {
        let conn = Connection::open(f.path(name)).expect("the file opens");
        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("a version");

        assert_eq!(
            version, expected,
            "{name} should be at schema version {expected}"
        );
    }
}

/// An old binary must refuse a database a newer one wrote.
///
/// Carrying on would mean writing rows the newer schema does not describe,
/// into a file holding users' notes. Refusing to start is the lesser failure,
/// and it is loud.
#[tokio::test]
async fn a_newer_schema_is_refused_rather_than_used() {
    let f = Fixture::new().await;
    let conn = Connection::open(f.path("notes.db")).expect("the file opens");
    conn.execute_batch("PRAGMA user_version = 99;")
        .expect("bumped");
    drop(conn);

    let error = Store::open(f.directory(), FreeSpaceFloor::NONE)
        .await
        .expect_err("refused");

    let StoreError::SchemaTooNew {
        database,
        found,
        known,
    } = error
    else {
        panic!("expected SchemaTooNew, got {error:?}");
    };
    assert_eq!(database, "notes");
    assert_eq!(found, 99);
    assert_eq!(known, 3);
}

/// The settings the project's guarantees rest on are read back, not assumed.
#[tokio::test]
async fn the_required_pragmas_are_in_force_on_both_files() {
    let f = Fixture::new().await;

    for name in ["notes.db", "keys.db"] {
        let conn = Connection::open(f.path(name)).expect("the file opens");

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("a journal mode");

        assert_eq!(
            journal_mode.to_ascii_lowercase(),
            "wal",
            "{name} should be in WAL mode, which is persistent in the file"
        );
    }
}

/// The key file truncates its own freelist; the note file does not.
///
/// `auto_vacuum` cannot be changed after a table exists, so if this ever
/// regresses it will regress permanently for every existing deployment. That
/// makes it worth a test rather than a comment.
#[tokio::test]
async fn only_the_key_file_auto_vacuums() {
    let f = Fixture::new().await;

    let expectations = [("keys.db", 1), ("notes.db", 0)];
    for (name, wanted) in expectations {
        let conn = Connection::open(f.path(name)).expect("the file opens");
        let mode: i64 = conn
            .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
            .expect("a mode");

        assert_eq!(mode, wanted, "{name} has the wrong auto_vacuum mode");
    }
}
