//! Wall-clock time, as the only unit the schema stores.

use std::time::{SystemTime, UNIX_EPOCH};

/// A point in time, as whole seconds since the Unix epoch.
///
/// Seconds rather than milliseconds because the shortest TTL is ten minutes
/// and nothing here benefits from finer resolution.
///
/// The store never reads the clock itself: every operation that depends on
/// "now" takes it as an argument. That is not ceremony -- it is what makes the
/// expiry logic testable without sleeping, and it keeps one notion of "now"
/// across the several statements that make up a single burn.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnixTime(i64);

impl UnixTime {
    /// Reads the system clock.
    ///
    /// A clock set before the Unix epoch yields zero, which makes every note
    /// look expired and so refuses reads rather than serving them. Failing
    /// closed on a nonsensical clock is the right direction for this service.
    #[must_use]
    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
            });
        Self(seconds)
    }

    /// Builds a time from raw seconds. Mainly for tests and for row decoding.
    #[must_use]
    pub const fn from_seconds(seconds: i64) -> Self {
        Self(seconds)
    }

    /// The raw seconds, for binding into a SQL statement.
    #[must_use]
    pub const fn as_seconds(self) -> i64 {
        self.0
    }

    /// The time `seconds` later, saturating rather than overflowing.
    #[must_use]
    pub const fn plus_seconds(self, seconds: u32) -> Self {
        Self(self.0.saturating_add(seconds as i64))
    }

    /// The UTC calendar day, as `YYYY-MM-DD`.
    ///
    /// Only used to bucket aggregate daily stats. The civil-date arithmetic is
    /// the standard days-since-epoch conversion, inlined rather than pulling in
    /// a date crate for one format string.
    #[must_use]
    pub(crate) fn day(self) -> String {
        let (year, month, day) = civil_from_days(self.0.div_euclid(86_400));
        format!("{year:04}-{month:02}-{day:02}")
    }
}

/// Civil `(year, month, day)` from a count of days since 1970-01-01.
///
/// Howard Hinnant's `civil_from_days`, whose intermediate values are bounded to
/// `[0, 146096]` by the `era` subtraction, which is what makes the casts exact.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "era brings the day-of-era into [0, 146096] and the month/day into their small ranges, so every cast is in range"
)]
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::UnixTime;

    #[test]
    fn now_is_after_2026() {
        // 2026-01-01T00:00:00Z. A clock before this means the host is wrong,
        // not that this assertion is.
        assert!(UnixTime::now().as_seconds() > 1_767_225_600);
    }

    #[test]
    fn plus_seconds_saturates_instead_of_overflowing() {
        let far_future = UnixTime::from_seconds(i64::MAX);

        assert_eq!(far_future.plus_seconds(86_400), far_future);
    }

    #[test]
    fn times_order_by_their_seconds() {
        assert!(UnixTime::from_seconds(10) < UnixTime::from_seconds(11));
    }

    #[test]
    fn day_is_the_utc_calendar_date() {
        assert_eq!(UnixTime::from_seconds(0).day(), "1970-01-01");
        // 2026-01-01T00:00:00Z, and the second before it is still 2025.
        assert_eq!(UnixTime::from_seconds(1_767_225_600).day(), "2026-01-01");
        assert_eq!(UnixTime::from_seconds(1_767_225_599).day(), "2025-12-31");
        // 2026-10-31T00:00:00Z.
        assert_eq!(UnixTime::from_seconds(1_793_404_800).day(), "2026-10-31");
    }
}
