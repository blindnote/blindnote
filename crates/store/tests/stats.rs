//! Daily aggregate stats and the approximate unique-visitor count.

mod common;

use blindnote_store::UnixTime;
use common::{Fixture, note};

/// 2026-10-31T00:00:00Z.
const DAY_ONE: i64 = 1_793_404_800;
/// 2026-11-01T00:00:00Z (one day later).
const DAY_TWO: i64 = DAY_ONE + 86_400;

#[tokio::test]
async fn views_and_uniques_land_in_the_days_row() {
    let f = Fixture::new().await;
    let now = UnixTime::from_seconds(DAY_ONE);

    // Five views from three distinct addresses.
    for address in [
        [1_u8, 1, 1, 1],
        [1, 1, 1, 1],
        [2, 2, 2, 2],
        [2, 2, 2, 2],
        [3, 3, 3, 3],
    ] {
        f.store.record_view(now, &address);
    }

    // Some note activity on the same day.
    let read_me = f.store.create(note(now)).await.expect("stored");
    let _leave_me = f.store.create(note(now)).await.expect("stored");
    f.store.burn(read_me, now).await.expect("burned");

    // The sweep is what writes the daily row.
    f.store.sweep(now).await.expect("swept");

    // Cumulative total carries the views too.
    assert_eq!(f.store.stats().await.expect("stats").views, 5);

    let rows = f.store.daily_stats(7).await.expect("daily");
    assert_eq!(rows.len(), 1, "one day of activity");

    let today = &rows[0];
    assert_eq!(today.day, "2026-10-31");
    assert_eq!(today.views, 5);
    assert_eq!(today.created, 2);
    assert_eq!(today.read, 1);
    assert!(
        (2..=4).contains(&today.uniques),
        "three distinct addresses should estimate to ~3, got {}",
        today.uniques
    );
}

#[tokio::test]
async fn each_utc_day_gets_its_own_row() {
    let f = Fixture::new().await;

    let day_one = UnixTime::from_seconds(DAY_ONE);
    f.store.record_view(day_one, &[10_u8, 0, 0, 1]);
    f.store.record_view(day_one, &[10, 0, 0, 2]);
    f.store.sweep(day_one).await.expect("swept day one");

    let day_two = UnixTime::from_seconds(DAY_TWO);
    f.store.record_view(day_two, &[20_u8, 0, 0, 1]);
    f.store.record_view(day_two, &[20, 0, 0, 2]);
    f.store.record_view(day_two, &[20, 0, 0, 3]);
    f.store.sweep(day_two).await.expect("swept day two");

    let rows = f.store.daily_stats(7).await.expect("daily");
    assert_eq!(rows.len(), 2, "two distinct days");

    // Newest first.
    assert_eq!(rows[0].day, "2026-11-01");
    assert_eq!(
        rows[0].views, 3,
        "the second day's views, not the running total"
    );
    assert_eq!(rows[1].day, "2026-10-31");
    assert_eq!(rows[1].views, 2);

    // Each day's uniques are isolated: the salt and sketch rotate at midnight.
    assert!((1..=3).contains(&rows[1].uniques));
    assert!((2..=4).contains(&rows[0].uniques));
}

#[tokio::test]
async fn repeated_views_from_one_address_count_once() {
    let f = Fixture::new().await;
    let now = UnixTime::from_seconds(DAY_ONE);

    for _ in 0..500 {
        f.store.record_view(now, b"the-same-visitor");
    }
    f.store.sweep(now).await.expect("swept");

    let rows = f.store.daily_stats(7).await.expect("daily");
    assert_eq!(rows[0].views, 500, "every view is counted");
    assert!(
        rows[0].uniques <= 2,
        "one address should estimate to ~1, got {}",
        rows[0].uniques
    );
}

/// The daily rows and the day's sketch survive a restart.
#[tokio::test]
async fn the_daily_rows_survive_a_restart() {
    let f = Fixture::new().await;
    let now = UnixTime::from_seconds(DAY_ONE);

    f.store.record_view(now, &[7_u8, 7, 7, 1]);
    f.store.record_view(now, &[7, 7, 7, 2]);
    f.store.sweep(now).await.expect("swept");

    let reopened = f.reopen().await;
    // Another view on the same day after the restart should add to the row, and
    // to the day's existing sketch rather than starting a new one.
    reopened.record_view(now, &[7, 7, 7, 3]);
    reopened.sweep(now).await.expect("swept again");

    let rows = reopened.daily_stats(7).await.expect("daily");
    assert_eq!(rows.len(), 1, "still one day");
    assert_eq!(rows[0].views, 3, "two before the restart, one after");
    assert!(
        (2..=4).contains(&rows[0].uniques),
        "the sketch resumed rather than resetting, got {}",
        rows[0].uniques
    );
}
