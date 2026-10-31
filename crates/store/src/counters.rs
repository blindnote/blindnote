//! Anonymous aggregate usage counters, kept in `notes.db`.
//!
//! A handful of integers -- notes created, read (burned), expired unread, and
//! reported -- with no per-note data behind them. There is no creation
//! timestamp to correlate against (see the schema), so these cannot be joined
//! back to any individual note; they are totals and nothing else.
//!
//! They live in memory as atomics so the increment on the hot path is free,
//! are loaded from the table at startup, and are written back on each sweep.
//! A crash therefore loses at most the last sweep interval's increments: the
//! cumulative totals are approximate by design. The one number that has to be
//! exact -- how many notes are pending right now -- is never a counter but a
//! live `COUNT(*)`, which cannot drift.

use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::{Connection, TransactionBehavior, params};

use crate::StoreError;

/// The in-memory counters.
#[derive(Debug, Default)]
pub(crate) struct Counters {
    created: AtomicU64,
    read: AtomicU64,
    expired: AtomicU64,
    reported: AtomicU64,
}

/// A point-in-time copy of the counters, cheap to pass across threads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Snapshot {
    pub(crate) created: u64,
    pub(crate) read: u64,
    pub(crate) expired: u64,
    pub(crate) reported: u64,
}

impl Snapshot {
    /// The counters as `(column, value)` pairs, for persisting.
    fn named(self) -> [(&'static str, u64); 4] {
        [
            ("created", self.created),
            ("read", self.read),
            ("expired", self.expired),
            ("reported", self.reported),
        ]
    }
}

impl Counters {
    /// Reads the persisted totals into fresh atomics.
    ///
    /// Unknown rows are ignored and missing ones default to zero, so a schema
    /// that gains or loses a counter does not make this fail.
    pub(crate) fn load(conn: &Connection) -> Result<Self, StoreError> {
        let counters = Self::default();

        let mut statement = conn.prepare("SELECT name, value FROM counters")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (name, value) = row?;
            if let Some(counter) = counters.by_name(&name) {
                counter.store(u64::try_from(value).unwrap_or(0), Ordering::Relaxed);
            }
        }

        Ok(counters)
    }

    /// The atomic for a column name, if it is one this build knows.
    fn by_name(&self, name: &str) -> Option<&AtomicU64> {
        match name {
            "created" => Some(&self.created),
            "read" => Some(&self.read),
            "expired" => Some(&self.expired),
            "reported" => Some(&self.reported),
            _ => None,
        }
    }

    /// One more note was created.
    pub(crate) fn inc_created(&self) {
        self.created.fetch_add(1, Ordering::Relaxed);
    }

    /// One more note was read (burned).
    pub(crate) fn inc_read(&self) {
        self.read.fetch_add(1, Ordering::Relaxed);
    }

    /// One more note was destroyed by a report.
    pub(crate) fn inc_reported(&self) {
        self.reported.fetch_add(1, Ordering::Relaxed);
    }

    /// `count` notes expired unread and were swept.
    pub(crate) fn add_expired(&self, count: u64) {
        self.expired.fetch_add(count, Ordering::Relaxed);
    }

    /// A copy of the current totals.
    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            created: self.created.load(Ordering::Relaxed),
            read: self.read.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
            reported: self.reported.load(Ordering::Relaxed),
        }
    }
}

/// Writes a snapshot back to the table, in one transaction.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if a statement fails.
pub(crate) fn persist(conn: &mut Connection, snapshot: Snapshot) -> Result<(), StoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for (name, value) in snapshot.named() {
        tx.execute(
            "UPDATE counters SET value = ?1 WHERE name = ?2",
            params![i64::try_from(value).unwrap_or(i64::MAX), name],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// How many notes are stored right now. Exact, not a counter.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the query fails.
pub(crate) fn pending(conn: &Connection) -> Result<u64, StoreError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
    Ok(u64::try_from(count).unwrap_or(0))
}
