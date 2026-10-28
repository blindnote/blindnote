//! Opening a database file with the settings this project depends on.

use std::path::Path;

use rusqlite::Connection;

use crate::StoreError;

/// How a file's write-ahead log should behave.
///
/// The two files are not symmetric. Ciphertext lingering in the notes WAL is
/// harmless: without the key it is noise. A `k_note` lingering in the key WAL
/// is the whole secret, so that file is configured to keep as little history
/// as SQLite allows.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WalPolicy {
    /// Checkpoint every 64 pages, let the WAL settle at up to 1 MiB.
    Relaxed,
    /// Checkpoint after every commit and truncate the WAL to nothing.
    Minimal,
}

impl WalPolicy {
    /// Pages written before SQLite starts a checkpoint on its own.
    const fn autocheckpoint(self) -> u32 {
        match self {
            Self::Relaxed => 64,
            Self::Minimal => 1,
        }
    }

    /// Bytes the WAL may retain after a checkpoint.
    const fn journal_size_limit(self) -> i64 {
        match self {
            Self::Relaxed => 1024 * 1024,
            Self::Minimal => 0,
        }
    }
}

/// Opens a connection and applies every setting this project relies on.
///
/// `fresh` must be true when the file is being created, because
/// `PRAGMA auto_vacuum` cannot be changed once a table exists.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the file cannot be opened, if a pragma is
/// rejected, or if a pragma did not take effect. The last case is checked
/// rather than assumed: a `secure_delete` that silently stayed off would make
/// a documented guarantee false with nothing in the logs to say so.
pub(crate) fn open(path: &Path, policy: WalPolicy, fresh: bool) -> Result<Connection, StoreError> {
    let conn = Connection::open(path)?;

    // Before any table exists, or not at all. On the key file this is what
    // makes a deleted row's page leave the file instead of joining a freelist.
    if fresh && matches!(policy, WalPolicy::Minimal) {
        conn.execute_batch("PRAGMA auto_vacuum = FULL;")?;
    }

    conn.execute_batch(&format!(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = FULL;
         PRAGMA fullfsync = ON;
         PRAGMA checkpoint_fullfsync = ON;
         PRAGMA secure_delete = ON;
         PRAGMA temp_store = MEMORY;
         PRAGMA trusted_schema = OFF;
         PRAGMA cell_size_check = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA wal_autocheckpoint = {autocheckpoint};
         PRAGMA journal_size_limit = {journal_size_limit};",
        autocheckpoint = policy.autocheckpoint(),
        journal_size_limit = policy.journal_size_limit(),
    ))?;

    verify(&conn)?;
    Ok(conn)
}

/// Reads back the settings whose absence would quietly break a guarantee.
fn verify(conn: &Connection) -> Result<(), StoreError> {
    let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    expect(&journal_mode.to_ascii_lowercase(), "wal", "journal_mode")?;

    // SQLite reports synchronous as an integer: 2 is FULL.
    let synchronous: i64 = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    expect(&synchronous.to_string(), "2", "synchronous")?;

    // Only macOS honours this, and without it `synchronous = FULL` is a weaker
    // promise there than it reads as: Darwin's fsync() returns once the write
    // reaches the drive, not once the drive has committed it, and only
    // F_FULLFSYNC waits for the latter. Production is Linux, where fsync()
    // already does the right thing and this pragma is inert -- but a
    // durability measurement taken on a developer's Mac without it is
    // measuring something else, and would be published as if it were not.
    let fullfsync: i64 = conn.query_row("PRAGMA fullfsync", [], |row| row.get(0))?;
    expect(&fullfsync.to_string(), "1", "fullfsync")?;

    let secure_delete: i64 = conn.query_row("PRAGMA secure_delete", [], |row| row.get(0))?;
    expect(&secure_delete.to_string(), "1", "secure_delete")?;

    // 2 is MEMORY. A temporary B-tree spilled to a file would put row content
    // somewhere nothing in this project ever deletes.
    let temp_store: i64 = conn.query_row("PRAGMA temp_store", [], |row| row.get(0))?;
    expect(&temp_store.to_string(), "2", "temp_store")?;

    Ok(())
}

/// Fails if a pragma did not take the value it was given.
fn expect(found: &str, wanted: &str, pragma: &'static str) -> Result<(), StoreError> {
    if found == wanted {
        return Ok(());
    }
    Err(StoreError::Pragma {
        pragma,
        wanted: wanted.to_owned(),
        found: found.to_owned(),
    })
}

/// Runs a `TRUNCATE` checkpoint, reporting whether it had to give up.
///
/// A `TRUNCATE` checkpoint writes the WAL back into the database file and then
/// shortens the WAL to zero bytes. That second part is the reason it is used
/// here rather than the default passive checkpoint: after a burn, the point is
/// that the key's bytes stop existing in the log.
///
/// SQLite reports a busy checkpoint by returning 1 in the first column instead
/// of failing. Treating that as success is how a key quietly survives a burn,
/// so the caller is told and logs it.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the pragma itself fails.
pub(crate) fn checkpoint_truncate(conn: &Connection) -> Result<bool, StoreError> {
    let busy: i64 = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
    Ok(busy == 0)
}
