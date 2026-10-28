//! Notes that may be read more than once.
//!
//! Version 1 of the product only ever creates single-view notes, but the
//! column and both statements exist, so they are held to the same contract:
//! the key must survive while views remain and be destroyed with the last one.

mod common;

use blindnote_store::{StoreError, UnixTime};
use common::{CIPHERTEXT, Fixture, note_with_views};

#[tokio::test]
async fn a_three_view_note_is_readable_three_times() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f
        .store
        .create(note_with_views(now, 3))
        .await
        .expect("stored");

    for attempt in 1..=3 {
        let read = f.store.burn(id, now).await.unwrap_or_else(|error| {
            panic!("read {attempt} of 3 failed: {error}");
        });
        assert_eq!(read, CIPHERTEXT);
    }

    let fourth = f.store.burn(id, now).await.expect_err("spent");
    assert!(matches!(fourth, StoreError::Unavailable));
}

/// The interesting half: an intermediate read must not destroy the key.
///
/// If the key were taken on the first of three reads, the second read would
/// find the note row still present and its key gone -- a note that exists and
/// cannot be opened.
#[tokio::test]
async fn an_intermediate_read_keeps_the_key() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f
        .store
        .create(note_with_views(now, 2))
        .await
        .expect("stored");

    f.store.burn(id, now).await.expect("first read");
    let second = f.store.burn(id, now).await.expect("second read");

    assert_eq!(second, CIPHERTEXT);
}

#[tokio::test]
async fn a_note_with_no_views_is_refused_rather_than_stored() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let error = f
        .store
        .create(note_with_views(now, 0))
        .await
        .expect_err("nobody could ever read it");

    assert!(matches!(error, StoreError::NoViews));
}

/// The same guard, one level lower.
///
/// `create` refuses a zero-view note, so this row can only come from a future
/// bug or a hand-edited file. It is written directly to check that the burn
/// statement refuses it too -- the condition is `views_left = 1`, and the
/// obvious `views_left <= 1` would have served a note that was already spent.
#[tokio::test]
async fn a_hand_written_zero_view_row_is_still_not_served() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f
        .store
        .create(note_with_views(now, 1))
        .await
        .expect("stored");

    let conn = rusqlite::Connection::open(f.path("notes.db")).expect("the note file opens");
    let changed = conn
        .execute("UPDATE notes SET views_left = 0", [])
        .expect("the row is updated");
    assert_eq!(changed, 1);
    drop(conn);

    let error = f.store.burn(id, now).await.expect_err("no views left");

    assert!(matches!(error, StoreError::Unavailable));
}
