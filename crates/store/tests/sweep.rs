//! The expiry sweeper.

mod common;

use blindnote_store::{StoreError, UnixTime};
use common::{CIPHERTEXT, Fixture, expired_note, note};

#[tokio::test]
async fn a_sweep_with_nothing_to_do_removes_nothing() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    f.store.create(note(now)).await.expect("stored");

    let report = f.store.sweep(now).await.expect("swept");

    assert_eq!(report.notes, 0);
    assert_eq!(report.keys, 0);
}

#[tokio::test]
async fn a_sweep_leaves_live_notes_readable() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let live = f.store.create(note(now)).await.expect("stored");
    f.store.create(expired_note(now)).await.expect("stored");

    let report = f.store.sweep(now).await.expect("swept");
    assert_eq!(report.notes, 1, "only the expired note should go");

    assert_eq!(
        f.store.burn(live, now).await.expect("still there"),
        CIPHERTEXT
    );
}

#[tokio::test]
async fn a_sweep_removes_both_rows_of_an_expired_note() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(expired_note(now)).await.expect("stored");

    let report = f.store.sweep(now).await.expect("swept");

    assert_eq!(report.notes, 1);
    assert_eq!(report.keys, 1);
    assert!(matches!(
        f.store.burn(id, now).await.expect_err("gone"),
        StoreError::Unavailable
    ));
}

#[tokio::test]
async fn sweeping_twice_is_harmless() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    f.store.create(expired_note(now)).await.expect("stored");

    let first = f.store.sweep(now).await.expect("swept");
    let second = f.store.sweep(now).await.expect("swept again");

    assert_eq!(first.notes, 1);
    assert_eq!(second.notes, 0);
    assert_eq!(second.keys, 0);
}

/// A note burned by a reader leaves nothing for the sweeper to find.
///
/// If the burn path were leaving a row behind, the sweeper's counts would say
/// so here rather than in production three weeks later.
#[tokio::test]
async fn a_burned_note_leaves_the_sweeper_nothing() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");
    f.store.burn(id, now).await.expect("burned");

    let far_future = now.plus_seconds(10 * common::TTL_SECONDS);
    let report = f.store.sweep(far_future).await.expect("swept");

    assert_eq!(report.notes, 0);
    assert_eq!(report.keys, 0);
}

/// Expiry is inclusive of its own second.
///
/// A note whose deadline is exactly now is expired, not alive. Pinning the
/// boundary matters because the burn condition and the sweep condition have to
/// agree on it: `expires_at > now` to serve, `expires_at <= now` to delete. If
/// they disagreed, one second's worth of notes would be both unservable and
/// unsweepable.
#[tokio::test]
async fn a_note_expiring_exactly_now_is_gone() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let id = f
        .store
        .create(blindnote_store::NewNote {
            client_ciphertext: CIPHERTEXT.to_vec(),
            expires_at: now,
            views_left: 1,
        })
        .await
        .expect("stored");

    assert!(matches!(
        f.store.burn(id, now).await.expect_err("not served"),
        StoreError::Unavailable
    ));
    assert_eq!(f.store.sweep(now).await.expect("swept").notes, 1);
}
