//! Statements against the note file.

use blindnote_crypto::{Id, Nonce};
use rusqlite::{Connection, OptionalExtension as _, Row, TransactionBehavior, params};

use crate::{StoreError, UnixTime};

/// A note row, taken out of the way of whoever asked for it.
#[derive(Debug)]
pub(crate) struct Taken {
    /// Handle into the key file.
    pub(crate) key_id: Id,
    /// The sealed payload.
    pub(crate) blob: Vec<u8>,
    /// Nonce the payload was sealed under.
    pub(crate) nonce: Nonce,
    /// Whether that was the last permitted view.
    ///
    /// True means the row is gone and the key must be destroyed. False means
    /// the row survives with one fewer view and the key has to stay.
    pub(crate) was_final: bool,
}

/// The three columns every read of this table wants, as raw bytes.
///
/// Decoding into the crypto types happens outside the closure: a `rusqlite`
/// row mapper can only fail with a `rusqlite::Error`, and a wrong-length blob
/// is not one of those.
type RawRow = (Vec<u8>, Vec<u8>, Vec<u8>);

/// Reads `key_id`, `blob` and `nonce` without interpreting them.
fn raw_row(row: &Row<'_>) -> rusqlite::Result<RawRow> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
}

/// Turns raw columns into the types the rest of the crate uses.
fn decode(raw: RawRow, was_final: bool) -> Result<Taken, StoreError> {
    let (key_id, blob, nonce) = raw;
    Ok(Taken {
        key_id: Id::from_slice(&key_id)?,
        blob,
        nonce: Nonce::from_slice(&nonce)?,
        was_final,
    })
}

/// Inserts a note row.
///
/// # Errors
///
/// Returns [`StoreError::IdCollision`] if the id is already taken, which with
/// 128 random bits means the random number generator is broken rather than
/// that a retry is in order. Otherwise [`StoreError::Sqlite`].
pub(crate) fn insert(
    conn: &Connection,
    id: &Id,
    key_id: &Id,
    blob: &[u8],
    nonce: &Nonce,
    expires_at: UnixTime,
    views_left: u32,
) -> Result<(), StoreError> {
    let result = conn.execute(
        "INSERT INTO notes (id, key_id, blob, nonce, expires_at, views_left)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id.as_bytes().as_slice(),
            key_id.as_bytes().as_slice(),
            blob,
            nonce.as_bytes().as_slice(),
            expires_at.as_seconds(),
            i64::from(views_left),
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

/// Takes a note for reading: removes it, or spends one of its remaining views.
///
/// Both statements run inside one immediate transaction, so a second caller
/// arriving at the same moment finds the row already gone. That is the whole
/// of the atomic-burn claim for this file, and it is why the transaction is
/// explicit: SQLite's own documentation says the changes of a `RETURNING`
/// statement happen on the first step but "the commit does not occur until the
/// statement finishes", so relying on autocommit around a statement that is
/// stepped once would be relying on something undocumented.
///
/// # Errors
///
/// Returns [`StoreError::Unavailable`] when the note does not exist, has
/// expired, or has already been taken. One error for all three, produced by
/// one branch, so that no caller can tell them apart and none has to try.
///
/// A row with no views left matches neither statement and is therefore never
/// served. `create` refuses to write such a row, so this is a guard against a
/// future bug or a hand-edited database rather than against normal input --
/// which is exactly where a `views_left <= 1` condition would have handed out
/// a note that was supposed to be spent.
pub(crate) fn take(conn: &mut Connection, id: &Id, now: UnixTime) -> Result<Taken, StoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let bound = params![id.as_bytes().as_slice(), now.as_seconds()];

    let last_view = tx
        .query_row(
            "DELETE FROM notes
              WHERE id = ?1 AND expires_at > ?2 AND views_left = 1
              RETURNING key_id, blob, nonce",
            bound,
            raw_row,
        )
        .optional()?;

    let taken = match last_view {
        Some(raw) => Some(decode(raw, true)?),
        None => tx
            .query_row(
                "UPDATE notes SET views_left = views_left - 1
                  WHERE id = ?1 AND expires_at > ?2 AND views_left > 1
                  RETURNING key_id, blob, nonce",
                bound,
                raw_row,
            )
            .optional()?
            .map(|raw| decode(raw, false))
            .transpose()?,
    };

    tx.commit()?;

    taken.ok_or(StoreError::Unavailable)
}

/// Deletes a note outright, whatever its expiry or remaining views.
///
/// This is the abuse-report path. Anybody holding a link can destroy the note
/// it points at, which is the only takedown mechanism that does not require
/// reading anything. It grants no new power: whoever holds the link could
/// already have burned the note by reading it.
///
/// Returns the key id when a row was removed, so the caller can destroy the
/// key as well.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the statement fails.
pub(crate) fn delete(conn: &mut Connection, id: &Id) -> Result<Option<Id>, StoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

    let key_id = tx
        .query_row(
            "DELETE FROM notes WHERE id = ?1 RETURNING key_id",
            params![id.as_bytes().as_slice()],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;

    tx.commit()?;

    key_id
        .map(|bytes| Id::from_slice(&bytes))
        .transpose()
        .map_err(Into::into)
}

/// Deletes every note whose deadline has passed.
///
/// The burn statement refuses to *serve* an expired note but leaves the row
/// alone. Without this sweep a note nobody ever opened would sit on disk until
/// the disk filled, and "self-destructing" would be false for exactly the case
/// users worry about.
///
/// # Errors
///
/// Returns [`StoreError::Sqlite`] if the statement fails.
pub(crate) fn delete_expired(conn: &Connection, now: UnixTime) -> Result<usize, StoreError> {
    let removed = conn.execute(
        "DELETE FROM notes WHERE expires_at <= ?1",
        params![now.as_seconds()],
    )?;
    Ok(removed)
}
