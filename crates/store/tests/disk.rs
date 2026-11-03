//! The free-space floor, which exists for one rule: notes are never lost.
//!
//! A full disk is the one ordinary failure that can destroy a note somebody is
//! waiting to read. The floor is set absurdly high in these tests -- higher
//! than any real filesystem has free -- so that the refusal path runs without
//! anybody having to fill a disk.

mod common;

use blindnote_store::{FreeSpaceFloor, NewNote, Store, StoreError, UnixTime};
use common::{CIPHERTEXT, Fixture, TTL_SECONDS, expired_note, note};
use tempfile::TempDir;

/// Higher than the free space of any filesystem these tests will meet.
const IMPOSSIBLE_FLOOR: FreeSpaceFloor = FreeSpaceFloor(u64::MAX);

/// A store that believes it has no room.
async fn cramped(directory: &TempDir) -> Store {
    Store::open(directory.path(), IMPOSSIBLE_FLOOR, UnixTime::now())
        .await
        .expect("the store still opens: a full disk is not a broken one")
}

#[tokio::test]
async fn a_nearly_full_disk_refuses_new_notes() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let store = cramped(&directory).await;

    let error = store
        .create(note(UnixTime::now()))
        .await
        .expect_err("refused for want of space");

    let StoreError::Full {
        free_bytes,
        floor_bytes,
    } = error
    else {
        panic!("expected StoreError::Full, got {error:?}");
    };
    assert_eq!(floor_bytes, u64::MAX);
    assert!(free_bytes < floor_bytes);
}

/// The half that matters: a stored note stays readable.
///
/// Refusing new notes is only acceptable because it protects the ones already
/// there. A floor that also blocked reads would turn a nearly-full disk into
/// data loss, which is the thing it exists to prevent.
#[tokio::test]
async fn a_note_stored_before_the_disk_filled_is_still_readable() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let now = UnixTime::now();

    // Stored while there was room.
    let roomy = Store::open(directory.path(), FreeSpaceFloor::NONE, UnixTime::now())
        .await
        .expect("the store opens");
    let id = roomy.create(note(now)).await.expect("stored");
    drop(roomy);

    // The disk fills, the process restarts, and the note is still there.
    let store = cramped(&directory).await;

    assert_eq!(
        store.burn(id, now).await.expect("still readable"),
        CIPHERTEXT
    );
}

/// Reporting makes room, so it stays open too.
#[tokio::test]
async fn a_nearly_full_disk_still_accepts_a_report() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let now = UnixTime::now();

    let roomy = Store::open(directory.path(), FreeSpaceFloor::NONE, UnixTime::now())
        .await
        .expect("the store opens");
    let id = roomy.create(note(now)).await.expect("stored");
    drop(roomy);

    let store = cramped(&directory).await;

    store.report(id).await.expect("a report is a deletion");
}

/// Sweeping makes room, so it stays open too.
#[tokio::test]
async fn a_nearly_full_disk_still_sweeps() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let now = UnixTime::now();

    let roomy = Store::open(directory.path(), FreeSpaceFloor::NONE, UnixTime::now())
        .await
        .expect("the store opens");
    roomy.create(expired_note(now)).await.expect("stored");
    drop(roomy);

    let store = cramped(&directory).await;
    let report = store.sweep(now).await.expect("the sweeper is not gated");

    assert_eq!(report.notes, 1);
    assert_eq!(report.keys, 1);
}

/// A floor of zero means the check does not run at all.
#[tokio::test]
async fn no_floor_means_no_refusal() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    f.store
        .create(NewNote {
            client_ciphertext: CIPHERTEXT.to_vec(),
            expires_at: now.plus_seconds(TTL_SECONDS),
            views_left: 1,
        })
        .await
        .expect("the fixture sets no floor");
}

/// The refusal is about space, not about the note.
///
/// Worth separating because the two refusals mean opposite things to a caller:
/// one says "send something else", the other says "come back later".
#[tokio::test]
async fn a_full_disk_and_an_unreadable_note_are_different_errors() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let store = cramped(&directory).await;

    let full = store
        .create(note(UnixTime::now()))
        .await
        .expect_err("refused");
    let missing = store
        .burn(
            blindnote_store::Id::random().expect("random"),
            UnixTime::now(),
        )
        .await
        .expect_err("not there");

    assert!(matches!(full, StoreError::Full { .. }));
    assert!(matches!(missing, StoreError::Unavailable));
    assert_ne!(full.to_string(), missing.to_string());
}
