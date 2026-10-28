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
}
