//! The abuse-report path: destroy a note by its link, without reading it.

mod common;

use blindnote_crypto::Id;
use blindnote_store::{StoreError, UnixTime};
use common::{Fixture, expired_note, note, note_with_views};

#[tokio::test]
async fn a_reported_note_becomes_unreadable() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");

    f.store.report(id).await.expect("reported");

    assert!(matches!(
        f.store.burn(id, now).await.expect_err("gone"),
        StoreError::Unavailable
    ));
}

/// Reporting must not say whether there was anything to report.
///
/// Every other endpoint is careful not to be an existence oracle. A report
/// endpoint that answered "no such note" would be one, and it would be the
/// easiest one to use: unauthenticated, and destructive only when it hits.
#[tokio::test]
async fn reporting_an_unknown_id_succeeds_silently() {
    let f = Fixture::new().await;

    let outcome = f.store.report(Id::random().expect("random")).await;

    assert!(
        outcome.is_ok(),
        "reporting an unknown id must look like any other report"
    );
}

#[tokio::test]
async fn reporting_twice_succeeds_both_times() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(note(now)).await.expect("stored");

    f.store.report(id).await.expect("first report");
    f.store.report(id).await.expect("second report");
}

/// A report is not a read, so remaining views do not protect the note.
#[tokio::test]
async fn a_report_destroys_a_multi_view_note_outright() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f
        .store
        .create(note_with_views(now, 5))
        .await
        .expect("stored");

    f.store.report(id).await.expect("reported");

    assert!(matches!(
        f.store.burn(id, now).await.expect_err("gone"),
        StoreError::Unavailable
    ));
}

/// An expired note can still be reported.
///
/// The row is still on disk until the next sweep, and a takedown request
/// should not depend on the reporter's note still being within its TTL.
#[tokio::test]
async fn an_expired_note_can_still_be_reported() {
    let f = Fixture::new().await;
    let now = UnixTime::now();
    let id = f.store.create(expired_note(now)).await.expect("stored");

    f.store.report(id).await.expect("reported");

    assert_eq!(
        f.store.sweep(now).await.expect("swept").notes,
        0,
        "the report should already have removed the row"
    );
}
