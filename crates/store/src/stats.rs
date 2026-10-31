//! Daily aggregate stats, and an approximate daily unique-visitor count.
//!
//! Two kinds of number, both identity-free. The per-day totals (views,
//! created, read, expired, reported) are kept in `stats_daily`, one row per UTC
//! day, written from the in-memory counters on the sweep cadence. The unique
//! estimate comes from a salted [`Hll`] sketch whose working state is in
//! `stats_uniques`; the salt is random, rotates each day, and the raw client
//! address is never stored -- only folded into one register and discarded. So
//! nothing here can be joined back to a note or to a visitor.
//!
//! Like the cumulative counters, the daily rows are approximate across a crash:
//! a crash loses at most the counts since the last sweep, and a handful of
//! events can land on the wrong side of a midnight boundary. For a count of
//! visitors neither matters.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::DailyStat;
use crate::StoreError;
use crate::counters::Snapshot;
use crate::hll::Hll;
use crate::time::UnixTime;

/// Length of the per-day HyperLogLog salt.
const SALT_LEN: usize = 16;

/// The working state for the current day's unique-visitor estimate, plus the
/// baseline for turning cumulative counts into a day's delta.
pub(crate) struct Daily {
    /// The UTC day the sketch is accumulating.
    day: String,
    /// This day's random salt. Never leaves the process except as its effect
    /// on the sketch's registers.
    salt: [u8; SALT_LEN],
    /// The day's approximate-unique sketch.
    hll: Hll,
    /// Cumulative counters at the last flush; the next flush records the delta.
    last: Snapshot,
}

/// A hand-written `Debug` so the salt and the raw registers never reach a log
/// or a panic message; the day and the current estimate are enough to read.
impl std::fmt::Debug for Daily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Daily")
            .field("day", &self.day)
            .field("uniques", &self.hll.estimate())
            .finish_non_exhaustive()
    }
}

impl Daily {
    /// Loads the current day's working sketch, or starts a fresh one.
    ///
    /// `baseline` is the cumulative counters as just loaded at startup, so the
    /// first flush records only what happens from now on rather than writing
    /// the whole lifetime's totals into today's row.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Sqlite`] if the sketch cannot be read, or
    /// [`StoreError::Random`] if a fresh salt cannot be generated.
    pub(crate) fn load(
        conn: &Connection,
        now: UnixTime,
        baseline: Snapshot,
    ) -> Result<Self, StoreError> {
        let day = now.day();

        let stored: Option<(Vec<u8>, Vec<u8>)> = conn
            .query_row(
                "SELECT salt, registers FROM stats_uniques WHERE day = ?1",
                params![day],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let (salt, hll) = match stored {
            Some((salt, registers)) => (salt_from(&salt)?, Hll::from_bytes(&registers)),
            None => (fresh_salt()?, Hll::new()),
        };

        Ok(Self {
            day,
            salt,
            hll,
            last: baseline,
        })
    }

    /// Folds one page view's client address into the day's sketch.
    ///
    /// On the request hot path, so it touches no disk: just a hash and a
    /// register update. A day change here rotates the salt and sketch so an
    /// address never spans two days' estimates.
    pub(crate) fn record(&mut self, now: UnixTime, address: &[u8]) {
        self.roll_to(now);
        self.hll.add(&self.salt, address);
    }

    /// Rotates onto a new UTC day: a fresh salt and an empty sketch.
    ///
    /// A salt that cannot be regenerated keeps the old one -- a vanishingly
    /// rare reuse is better than dropping the day's counting entirely.
    fn roll_to(&mut self, now: UnixTime) {
        let today = now.day();
        if self.day != today {
            self.day = today;
            self.salt = fresh_salt().unwrap_or(self.salt);
            self.hll = Hll::new();
        }
    }

    /// Writes the day's deltas and the current unique estimate, and persists
    /// the working sketch.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Sqlite`] if a statement fails.
    pub(crate) fn flush(
        &mut self,
        conn: &mut Connection,
        now: UnixTime,
        current: Snapshot,
    ) -> Result<(), StoreError> {
        self.roll_to(now);

        let delta = current.saturating_sub(self.last);
        let uniques = self.hll.estimate();

        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO stats_daily
                 (day, views, created, read, expired, reported, uniques)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(day) DO UPDATE SET
                 views    = views    + excluded.views,
                 created  = created  + excluded.created,
                 read     = read     + excluded.read,
                 expired  = expired  + excluded.expired,
                 reported = reported + excluded.reported,
                 uniques  = excluded.uniques",
            params![
                self.day,
                as_i64(delta.views),
                as_i64(delta.created),
                as_i64(delta.read),
                as_i64(delta.expired),
                as_i64(delta.reported),
                as_i64(uniques),
            ],
        )?;
        tx.execute(
            "INSERT INTO stats_uniques (day, salt, registers) VALUES (?1, ?2, ?3)
             ON CONFLICT(day) DO UPDATE SET
                 salt = excluded.salt, registers = excluded.registers",
            params![self.day, self.salt.as_slice(), self.hll.as_bytes()],
        )?;
        // Yesterday's sketch is finished with -- its estimate is already in its
        // row -- so the working table keeps only today's.
        tx.execute(
            "DELETE FROM stats_uniques WHERE day < ?1",
            params![self.day],
        )?;
        tx.commit()?;

        self.last = current;
        Ok(())
    }
}

/// The most recent `days` of daily rows, newest first.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the query fails.
pub(crate) fn recent(conn: &Connection, days: u32) -> Result<Vec<DailyStat>, StoreError> {
    let mut statement = conn.prepare(
        "SELECT day, views, created, read, expired, reported, uniques
         FROM stats_daily ORDER BY day DESC LIMIT ?1",
    )?;

    let rows = statement.query_map(params![days], |row| {
        Ok(DailyStat {
            day: row.get(0)?,
            views: u64_of(row.get(1)?),
            created: u64_of(row.get(2)?),
            read: u64_of(row.get(3)?),
            expired: u64_of(row.get(4)?),
            reported: u64_of(row.get(5)?),
            uniques: u64_of(row.get(6)?),
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Sixteen fresh random bytes for a day's salt.
fn fresh_salt() -> Result<[u8; SALT_LEN], StoreError> {
    let mut salt = [0_u8; SALT_LEN];
    getrandom::fill(&mut salt)?;
    Ok(salt)
}

/// A stored salt, or a fresh one if the stored bytes are the wrong length.
fn salt_from(bytes: &[u8]) -> Result<[u8; SALT_LEN], StoreError> {
    bytes.try_into().map_or_else(|_| fresh_salt(), Ok)
}

/// A count as an `i64` for binding, saturating rather than wrapping.
fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// A stored `i64` back as a `u64`, flooring a stray negative at zero.
fn u64_of(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}
