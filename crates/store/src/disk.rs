//! How much room is left.
//!
//! This exists for one rule: **notes are never lost**. A full disk is the one
//! ordinary failure that can destroy a note somebody is waiting to read --
//! SQLite cannot commit, the write-ahead log cannot grow, and the sweeper
//! cannot delete. The answer is to stop accepting *new* notes while there is
//! still room to finish serving the ones already stored, which means refusing
//! early rather than at the moment the disk fills.
//!
//! Only creation is gated. Reading, reporting and sweeping all stay open on a
//! full disk, because all three make room rather than consume it.

use std::path::Path;

use crate::StoreError;

/// Bytes available on the filesystem holding `path`.
///
/// Uses the count available to an unprivileged process rather than the total
/// free count: on most filesystems a few percent is reserved for root, and
/// counting that would mean refusing new notes only after the writes had
/// already started failing.
///
/// # Errors
///
/// Returns [`StoreError::DataDir`] if the filesystem cannot be queried.
pub(crate) fn free_bytes(path: &Path) -> Result<u64, StoreError> {
    let stats = rustix::fs::statvfs(path).map_err(|error| StoreError::DataDir {
        path: path.to_path_buf(),
        source: error.into(),
    })?;

    Ok(stats.f_bavail.saturating_mul(stats.f_frsize))
}

#[cfg(test)]
mod tests {
    use super::free_bytes;

    #[test]
    fn a_real_directory_reports_some_free_space() {
        let directory = tempfile::tempdir().expect("a temporary directory");

        let free = free_bytes(directory.path()).expect("the filesystem answers");

        assert!(
            free > 0,
            "the filesystem running these tests reports no free space"
        );
    }

    #[test]
    fn a_missing_directory_is_an_error_rather_than_zero() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let missing = directory.path().join("no-such-directory");

        // Zero would read as "the disk is full" and refuse every note, which
        // is the wrong answer to "this path does not exist".
        assert!(free_bytes(&missing).is_err());
    }
}
