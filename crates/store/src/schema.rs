//! The schemas, and a migration runner keyed on `PRAGMA user_version`.
//!
//! No migration framework and no migrations table: `user_version` is a 32-bit
//! integer in the database header that SQLite maintains for exactly this, and
//! a list of statements in source order is the whole mechanism.

use rusqlite::{Connection, TransactionBehavior};

use crate::StoreError;

/// One forward step of a schema.
///
/// There is no `down`. A rollback on this data would mean deciding what to do
/// with notes that exist under the newer schema, and the answer -- for a
/// service whose rows live at most a day -- is to wait a day, not to write a
/// reverse migration nobody will have tested.
struct Migration {
    /// The `user_version` this migration produces.
    version: i32,
    /// Statements to run, as one batch inside one transaction.
    sql: &'static str,
}

/// Schema of the note file.
///
/// `id` is 16 random bytes, not an autoincrementing integer: it is half of the
/// secret in a link, so it has to be unguessable.
///
/// There is deliberately no creation timestamp. Coarsening one while
/// `expires_at` stayed precise bought nothing -- the TTL menu is public, so
/// `expires_at` minus the TTL recovers the exact moment -- and no code path
/// needs it.
const NOTES_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: "CREATE TABLE notes (
              id         BLOB    NOT NULL PRIMARY KEY,
              key_id     BLOB    NOT NULL,
              blob       BLOB    NOT NULL,
              nonce      BLOB    NOT NULL,
              expires_at INTEGER NOT NULL,
              views_left INTEGER NOT NULL
          ) STRICT;

          CREATE INDEX notes_expires_at ON notes (expires_at);",
    },
    // Anonymous aggregate counters -- totals only, no per-note data. There is
    // no row here that could be joined back to a note (notes carry no creation
    // time), so this holds exactly four integers and nothing that identifies
    // anyone. `WITHOUT ROWID` for the same reason as the key table: the whole
    // row is its key.
    Migration {
        version: 2,
        sql: "CREATE TABLE counters (
                  name  TEXT    NOT NULL PRIMARY KEY,
                  value INTEGER NOT NULL
              ) STRICT, WITHOUT ROWID;

              INSERT INTO counters (name, value) VALUES
                  ('created', 0), ('read', 0), ('expired', 0), ('reported', 0);",
    },
];

/// Schema of the key file.
///
/// `WITHOUT ROWID` so that the 32 secret bytes live in exactly one B-tree
/// rather than in a table and a separate primary-key index.
///
/// `expires_at` is duplicated from the note row on purpose. SQLite's own
/// documentation says that transactions across attached databases "are atomic
/// for each individual database, but are not atomic across all databases as a
/// set", so there is no way to delete a note and its key in one atomic step.
/// Carrying the deadline here means this file can expire its own rows without
/// consulting the other one: a key written during a crashed create is still
/// destroyed on time, and there is no orphan to garbage-collect and no race
/// against a note row that has not landed yet.
const KEYS_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: "CREATE TABLE note_keys (
              key_id     BLOB    NOT NULL PRIMARY KEY,
              k_note     BLOB    NOT NULL,
              expires_at INTEGER NOT NULL
          ) STRICT, WITHOUT ROWID;

          CREATE INDEX note_keys_expires_at ON note_keys (expires_at);",
}];

/// Brings the note file up to the latest schema.
///
/// # Errors
///
/// See [`run`].
pub(crate) fn migrate_notes(conn: &mut Connection) -> Result<(), StoreError> {
    run(conn, "notes", NOTES_MIGRATIONS)
}

/// Brings the key file up to the latest schema.
///
/// # Errors
///
/// See [`run`].
pub(crate) fn migrate_keys(conn: &mut Connection) -> Result<(), StoreError> {
    run(conn, "keys", KEYS_MIGRATIONS)
}

/// Applies every migration newer than the file's current `user_version`.
///
/// # Errors
///
/// Returns [`StoreError::SchemaTooNew`] when the file was written by a newer
/// binary. Refusing is the point: an old binary that carried on would write
/// rows a newer schema does not describe, and these rows are users' notes.
///
/// Returns [`StoreError::Sqlite`] if a migration fails, in which case nothing
/// from that migration is applied -- each one runs in its own transaction
/// together with the `user_version` bump.
fn run(
    conn: &mut Connection,
    database: &'static str,
    migrations: &[Migration],
) -> Result<(), StoreError> {
    let known = migrations.last().map_or(0, |last| last.version);
    let found: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if found > known {
        return Err(StoreError::SchemaTooNew {
            database,
            found,
            known,
        });
    }

    for migration in migrations.iter().filter(|m| m.version > found) {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(migration.sql)?;
        // user_version takes no bound parameter, so it is formatted in. The
        // value is an i32 from a const in this file, never from input.
        tx.execute_batch(&format!("PRAGMA user_version = {};", migration.version))?;
        tx.commit()?;

        tracing::info!(database, version = migration.version, "schema migrated");
    }

    Ok(())
}
