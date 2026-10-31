//! The anonymous aggregate counters.

mod common;

use blindnote_store::{Stats, UnixTime};
use common::{Fixture, expired_note, note};

#[tokio::test]
async fn a_fresh_store_has_zero_counters() {
    let f = Fixture::new().await;

    assert_eq!(f.store.stats().await.expect("stats"), Stats::default());
}

/// Each operation moves exactly the counter it should, and pending is the live
/// truth rather than a counter.
#[tokio::test]
async fn the_counters_follow_creates_reads_reports_and_expiries() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let read_me = f.store.create(note(now)).await.expect("stored");
    let report_me = f.store.create(note(now)).await.expect("stored");
    let _leave_me = f.store.create(note(now)).await.expect("stored");

    f.store.burn(read_me, now).await.expect("burned");
    f.store.report(report_me).await.expect("reported");

    // One more note, already expired, then a sweep to remove it.
    f.store.create(expired_note(now)).await.expect("stored");
    assert_eq!(f.store.sweep(now).await.expect("swept").notes, 1);

    let stats = f.store.stats().await.expect("stats");
    assert_eq!(stats.created, 4, "three live plus one expired");
    assert_eq!(stats.read, 1);
    assert_eq!(stats.reported, 1);
    assert_eq!(stats.expired, 1);
    assert_eq!(stats.pending, 1, "only the untouched note is still stored");
}

/// A report that hits nothing counts nothing.
///
/// The report path answers the same whether or not a note was there (no
/// existence oracle), so the counter must only move when a row was actually
/// destroyed.
#[tokio::test]
async fn a_report_that_deletes_nothing_is_not_counted() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    let id = f.store.create(note(now)).await.expect("stored");
    f.store.report(id).await.expect("first report deletes it");
    f.store
        .report(id)
        .await
        .expect("second report finds nothing");

    assert_eq!(f.store.stats().await.expect("stats").reported, 1);
}

/// The cumulative totals are written on sweep and survive a restart; pending is
/// recomputed from the rows that are actually there.
#[tokio::test]
async fn the_counters_survive_a_restart() {
    let f = Fixture::new().await;
    let now = UnixTime::now();

    f.store.create(note(now)).await.expect("stored");
    let id = f.store.create(note(now)).await.expect("stored");
    f.store.burn(id, now).await.expect("burned");

    // The sweep is what persists the counters to disk.
    f.store.sweep(now).await.expect("swept");

    let stats = f.reopen().await.stats().await.expect("stats");
    assert_eq!(stats.created, 2, "cumulative totals survive a restart");
    assert_eq!(stats.read, 1);
    assert_eq!(stats.pending, 1, "the unburned note is still stored");
}
