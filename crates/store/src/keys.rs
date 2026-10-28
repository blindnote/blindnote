//! Statements against the key file.
//!
//! Everything in this module is about one question: after a burn, do the 32
//! bytes still exist anywhere in this file? Each deletion is therefore
//! followed by a `TRUNCATE` checkpoint, because `PRAGMA secure_delete` only
//! promises to scrub "ordinary tables" and says nothing at all about the
//! write-ahead log.

use blindnote_crypto::{Id, NoteKey};
use rusqlite::{Connection, OptionalExtension as _, TransactionBehavior, params};

use crate::{StoreError, UnixTime, db};

/// Stores a per-note key under its handle.
///
/// `expires_at` is a copy of the note's deadline. It is what lets this file
/// expire its own rows without consulting the note file -- see the comment on
/// the schema for why that independence is not optional.
///
/// # Errors
///
/// Returns [`StoreError::IdCollision`] on a duplicate handle, otherwise
/// [`StoreError::Sqlite`].
pub(crate) fn insert(
    conn: &Connection,
    key_id: &Id,
    key: &NoteKey,
    expires_at: UnixTime,
) -> Result<(), StoreError> {
    let result = conn.execute(
        "INSERT INTO note_keys (key_id, k_note, expires_at) VALUES (?1, ?2, ?3)",
        params![
            key_id.as_bytes().as_slice(),
            key.expose_secret().as_slice(),
            expires_at.as_seconds(),
        ],
    );

    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Err(StoreError::IdCollision)
        }
        Err(error) => Err(StoreError::Sqlite(error)),
    }
}

/// Takes a key and destroys it: the burn.
///
/// The row is deleted inside a transaction, the transaction is committed, and
/// only then is the WAL truncated. In that order, because a checkpoint can
/// only write back what has been committed.
///
/// # Errors
///
/// Returns [`StoreError::Unavailable`] if the handle has no row, which happens
/// when a previous create crashed between its two writes. The note's
/// ciphertext is then unrecoverable, and saying so in the same terms as "the
/// note is gone" is both true and all the caller needs.
pub(crate) fn take(conn: &mut Connection, key_id: &Id) -> Result<NoteKey, StoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

    let raw = tx
        .query_row(
            "DELETE FROM note_keys WHERE key_id = ?1 RETURNING k_note",
            params![key_id.as_bytes().as_slice()],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;

    tx.commit()?;

    let raw = raw.ok_or(StoreError::Unavailable)?;
    let key = NoteKey::from_slice(&raw)?;

    truncate_log(conn, "burn");

    Ok(key)
}

/// Reads a key without destroying it.
///
/// Only reachable for a note with more than one view left. In that case the
/// note has not been burned, so neither has its key.
///
/// # Errors
///
/// Returns [`StoreError::Unavailable`] if the handle has no row.
pub(crate) fn read(conn: &Connection, key_id: &Id) -> Result<NoteKey, StoreError> {
    let raw = conn
        .query_row(
            "SELECT k_note FROM note_keys WHERE key_id = ?1",
            params![key_id.as_bytes().as_slice()],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .ok_or(StoreError::Unavailable)?;

    Ok(NoteKey::from_slice(&raw)?)
}

/// Destroys a key without handing it over: the abuse-report path.
///
/// Says nothing about whether there was a row to destroy. The report endpoint
/// must not become the existence oracle that every other endpoint is careful
/// not to be.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the statement fails.
pub(crate) fn delete(conn: &Connection, key_id: &Id) -> Result<(), StoreError> {
    conn.execute(
        "DELETE FROM note_keys WHERE key_id = ?1",
        params![key_id.as_bytes().as_slice()],
    )?;

    truncate_log(conn, "report");

    Ok(())
}

/// Destroys every key whose deadline has passed.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the statement fails.
pub(crate) fn delete_expired(conn: &Connection, now: UnixTime) -> Result<usize, StoreError> {
    let removed = conn.execute(
        "DELETE FROM note_keys WHERE expires_at <= ?1",
        params![now.as_seconds()],
    )?;

    // Unconditionally, including when nothing matched. A statement that
    // deletes no rows still commits, and that commit still leaves frames in
    // the log -- measured: after a sweep with nothing to do, the key log held
    // 8 KB. Those frames carry no key material, so this is tidiness rather
    // than a fix; but "the key log is empty" is a claim worth being able to
    // check at any moment rather than only just after a burn.
    truncate_log(conn, "sweep");

    Ok(removed)
}

/// Truncates the write-ahead log, reporting rather than hiding a failure.
///
/// Deliberately infallible from the caller's point of view. By the time this
/// runs the row is already deleted and committed; failing the request would
/// not bring the key back, and it would turn a logging problem into a user
/// seeing an error for an operation that succeeded. What must not happen is
/// silence: a checkpoint that gave up leaves key bytes in the log, and the
/// next sweep is what cleans up after it.
fn truncate_log(conn: &Connection, after: &'static str) {
    match db::checkpoint_truncate(conn) {
        Ok(true) => {}
        Ok(false) => tracing::warn!(
            after,
            "the key log was busy and could not be truncated; key bytes may persist until the next sweep"
        ),
        Err(error) => tracing::error!(after, %error, "truncating the key log failed"),
    }
}
