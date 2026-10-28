#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

//! BlindNote's HTTP server.
//!
//! One-time-read, end-to-end encrypted, self-destructing notes. The plaintext
//! and the encryption key never reach this process: the browser encrypts, and
//! the key travels in the URL fragment, which is never sent in an HTTP request.
//! What this crate holds is therefore ciphertext it cannot read, plus the
//! 32-byte per-note key that makes the ciphertext recoverable -- and deleting
//! that key is what "burning" a note means.
//!
//! The rules this code is written against are in `CLAUDE.md` at the repository
//! root. Where a function looks like it is missing something obvious, the
//! comment above it says which rule asked for the omission.

mod config;
mod router;

pub use crate::config::{Config, ConfigError};
pub use crate::router::router;

use std::io;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

/// Why the server stopped.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The listener could not be bound.
    #[error("cannot bind {addr}: {source}")]
    Bind {
        /// Address that was asked for.
        addr: SocketAddr,
        /// What the operating system said.
        #[source]
        source: io::Error,
    },

    /// A shutdown signal handler could not be installed.
    ///
    /// This is fatal rather than ignorable: without it the process would be
    /// killed outright on deploy, cutting any burn that was in flight.
    #[error("cannot install the {signal} handler: {source}")]
    Signal {
        /// Name of the signal whose handler failed to install.
        signal: &'static str,
        /// What the operating system said.
        #[source]
        source: io::Error,
    },

    /// The accept loop failed.
    #[error("the server loop failed: {0}")]
    Serve(#[source] io::Error),
}

/// Binds the listener and serves until a shutdown signal arrives.
///
/// Shutdown is graceful: the listener stops accepting and in-flight requests
/// are allowed to finish. That matters more here than it usually does, because
/// a request in flight may be halfway through deleting a note's key, and the
/// client is waiting to be handed the plaintext that key unlocks.
///
/// # Errors
///
/// Returns [`RunError`] when a signal handler cannot be installed, when the
/// configured address cannot be bound, or when the accept loop itself fails.
/// A normal shutdown by signal returns `Ok(())`.
pub async fn run(config: Config) -> Result<(), RunError> {
    // Both handlers are installed before the first connection is accepted. An
    // installation failure must not surface as "shutdown does not work" one
    // deploy later.
    let mut interrupt = signal(SignalKind::interrupt()).map_err(|source| RunError::Signal {
        signal: "SIGINT",
        source,
    })?;
    let mut terminate = signal(SignalKind::terminate()).map_err(|source| RunError::Signal {
        signal: "SIGTERM",
        source,
    })?;

    let listener = TcpListener::bind(config.bind)
        .await
        .map_err(|source| RunError::Bind {
            addr: config.bind,
            source,
        })?;
    tracing::info!(addr = %config.bind, "listening");

    axum::serve(listener, router())
        .with_graceful_shutdown(async move {
            let reason = tokio::select! {
                _ = interrupt.recv() => "SIGINT",
                _ = terminate.recv() => "SIGTERM",
            };
            tracing::info!(signal = reason, "draining in-flight requests");
        })
        .await
        .map_err(RunError::Serve)
}
