//! The burn contract: a note is readable exactly once, and the three ways of
//! not being readable are indistinguishable.

mod common;

use blindnote_crypto::Id;
use blindnote_store::{StoreError, UnixTime};
use common::{CIPHERTEXT, Fixture, expired_note, note};

#[tokio::test]
async fn a_stored_note_comes_back_byte_for_byte() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let id = f.store.create(note(now)).await.expect("stored");
    let read = f.store.burn(id, now).await.expect("burned");

    assert_eq!(read, CIPHERTEXT);
}

#[tokio::test]
async fn a_second_burn_finds_nothing() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");

    f.store.burn(id, now).await.expect("burned");
    let again = f.store.burn(id, now).await.expect_err("already burned");

    assert!(matches!(again, StoreError::Unavailable));
}

#[tokio::test]
async fn an_id_that_never_existed_finds_nothing() {
    let f = Fixture::new().await;

    let error = f
        .store
        .burn(Id::random().expect("random"), UnixTime::now())
        .await
        .expect_err("never existed");

    assert!(matches!(error, StoreError::Unavailable));
}

#[tokio::test]
async fn an_expired_note_finds_nothing() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(expired_note(now)).await.expect("stored");

    let error = f.store.burn(id, now).await.expect_err("expired");

    assert!(matches!(error, StoreError::Unavailable));
}

/// The three unavailable states have to be one response, and the cheapest way
/// to keep them that way is to assert that they are literally the same value.
#[tokio::test]
async fn missing_expired_and_burned_are_the_same_error() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let burned = f.store.create(note(now)).await.expect("stored");
    f.store.burn(burned, now).await.expect("burned");
    let expired = f.store.create(expired_note(now)).await.expect("stored");
    let missing = Id::random().expect("random");

    let mut rendered = Vec::new();
    for id in [burned, expired, missing] {
        let error = f.store.burn(id, now).await.expect_err("unavailable");
        rendered.push(error.to_string());
    }

    assert_eq!(rendered[0], rendered[1]);
    assert_eq!(rendered[1], rendered[2]);
}

/// The atomicity claim, stated as a race.
///
/// Sixteen readers arrive at one note at once. Exactly one may be handed the
/// ciphertext; the other fifteen must be told the note is gone. Anything else
/// means two people read a one-time note.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn exactly_one_of_sixteen_simultaneous_burns_wins() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");

    let attempts: Vec<_> = (0..16)
        .map(|_| {
            let store = f.store.clone();
            tokio::spawn(async move { store.burn(id, now).await })
        })
        .collect();

    let mut winners = 0;
    let mut unavailable = 0;
    for attempt in attempts {
        match attempt.await.expect("the task did not panic") {
            Ok(read) => {
                assert_eq!(read, CIPHERTEXT);
                winners += 1;
            }
            Err(StoreError::Unavailable) => unavailable += 1,
            Err(error) => panic!("unexpected failure: {error}"),
        }
    }

    assert_eq!(winners, 1, "a one-time note was handed out {winners} times");
    assert_eq!(unavailable, 15);
}

#[tokio::test]
async fn two_notes_do_not_share_anything() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let first = f.store.create(note(now)).await.expect("stored");
    let second = f.store.create(note(now)).await.expect("stored");

    assert_ne!(first, second);
    f.store.burn(first, now).await.expect("burned");
    assert_eq!(
        f.store.burn(second, now).await.expect("still there"),
        CIPHERTEXT,
        "burning one note must not affect another"
    );
}

#[tokio::test]
async fn notes_survive_a_restart() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");

    let after_restart = f.reopen(now).await;

    assert_eq!(
        after_restart.burn(id, now).await.expect("burned"),
        CIPHERTEXT
    );
}
