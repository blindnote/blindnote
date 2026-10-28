//! The periodic job that makes "self-destructing" true for unread notes.
//!
//! The burn statement refuses to *serve* an expired note but leaves the row
//! where it is. Without this task, a note nobody ever opened would sit on disk
//! until the disk filled -- and that is exactly the case a user worries about
//! when they choose a short expiry.

use std::time::Duration;

use blindnote_store::{Store, UnixTime};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

/// Starts sweeping, and keeps sweeping until the handle is dropped or aborted.
///
/// The first tick fires immediately, which is deliberate: whatever expired
/// while the process was down should go on the way up, not one interval later.
///
/// A failed sweep is logged and the next tick tries again. Giving up would be
/// worse than retrying -- the rows are still there either way, and the only
/// thing stopping would achieve is silence.
pub(crate) fn spawn(store: Store, interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // Skip missed ticks rather than firing them back to back. A sweep that
        // ran long means the database is busy, and the answer to a busy
        // database is not several more sweeps at once.
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;

            match store.sweep(UnixTime::now()).await {
                // Counts only. Nothing here identifies a note, which is why a
                // sweep can be logged at all.
                Ok(report) if report.notes > 0 || report.keys > 0 => {
                    tracing::info!(
                        notes = report.notes,
                        keys = report.keys,
                        "swept expired notes"
                    );
                }
                Ok(_) => {}
                Err(error) => tracing::error!(%error, "the sweep failed"),
            }
        }
    })
}
