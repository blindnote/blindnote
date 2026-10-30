#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, reason = "unwrap is acceptable in test code")
)]

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

mod api;
mod assets;
mod config;
mod csp;
mod error;
mod headers;
mod rate_limit;
mod router;
mod sweeper;

pub use crate::config::{Config, ConfigError};

use std::io;
use std::net::SocketAddr;

use blindnote_store::{FreeSpaceFloor, Store, StoreError};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

use crate::api::ApiState;
use crate::rate_limit::Limiters;
use crate::router::Layers;

/// Why the server stopped.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The databases could not be opened, configured or migrated.
    #[error("storage could not be opened: {0}")]
    Storage(#[from] StoreError),

    /// A rate limiter could not be built from the configuration.
    #[error("{setting} does not describe a usable rate limit")]
    Limiter {
        /// The environment variable, or variables, to look at.
        setting: &'static str,
    },

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

/// Binds the configured address and serves until a shutdown signal.
///
/// # Errors
///
/// Returns [`RunError::Bind`] if the address cannot be bound, and otherwise
/// whatever [`serve`] returns.
pub async fn run(config: Config) -> Result<(), RunError> {
    let listener = TcpListener::bind(config.bind)
        .await
        .map_err(|source| RunError::Bind {
            addr: config.bind,
            source,
        })?;

    serve(config, listener).await
}

/// Opens storage, starts the background jobs, and serves on `listener`.
///
/// Takes an already-bound listener rather than binding one itself, so that the
/// caller can learn the address first. A test binds port zero and reads back
/// what the kernel chose; a future deployment could be handed a socket by
/// `systemd`. `config.bind` is ignored here -- the listener is the truth.
///
/// Shutdown is graceful: the listener stops accepting and in-flight requests
/// are allowed to finish. That matters more here than it usually does, because
/// a request in flight may be halfway through deleting a note's key, and the
/// client is waiting to be handed the plaintext that key unlocks.
///
/// # Errors
///
/// Returns [`RunError`] when storage cannot be opened, when the configuration
/// does not describe a usable rate limit, when a signal handler cannot be
/// installed, or when the accept loop itself fails. A normal shutdown by
/// signal returns `Ok(())`.
pub async fn serve(config: Config, listener: TcpListener) -> Result<(), RunError> {
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

    let store = Store::open(&config.data_dir, FreeSpaceFloor(config.min_free_bytes)).await?;
    let limiters = Limiters::new(&config).map_err(|setting| RunError::Limiter { setting })?;

    if config.trusted_proxy.is_none() {
        // Worth saying out loud, because the failure is quiet and looks like
        // the limiter working: behind a proxy, with nothing trusted, every
        // request is attributed to the proxy and the whole internet shares one
        // bucket. Set BLINDNOTE_TRUSTED_PROXY to the proxy's address.
        tracing::warn!(
            "BLINDNOTE_TRUSTED_PROXY is unset: every request is attributed to the address it \
             arrived from, so behind a reverse proxy all clients share one rate-limit bucket"
        );
    }

    if assets::is_empty() {
        // Loud, because a binary that serves an API and no interface looks
        // healthy from every angle except a browser's. It means this was built
        // without `npm run build` in client/ first.
        tracing::error!(
            "no interface is embedded in this binary: only /api and /healthz will answer"
        );
    }

    let pruner = limiters.spawn_pruner();
    let sweeper = sweeper::spawn(store.clone(), config.sweep_interval);

    let app = router::router(
        ApiState { store },
        &limiters,
        Layers {
            request_timeout: config.request_timeout,
            max_concurrent_requests: config.max_concurrent_requests,
        },
    );

    match listener.local_addr() {
        Ok(addr) => tracing::info!(%addr, "listening"),
        Err(error) => tracing::warn!(%error, "listening on an address that cannot be read back"),
    }

    // `with_connect_info` is what puts the peer address in the request
    // extensions, and the rate limiter has nothing to key on without it.
    let outcome = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        let reason = tokio::select! {
            _ = interrupt.recv() => "SIGINT",
            _ = terminate.recv() => "SIGTERM",
        };
        tracing::info!(signal = reason, "draining in-flight requests");
    })
    .await
    .map_err(RunError::Serve);

    // Both are pure housekeeping and have nothing in flight worth waiting for
    // by the time the last request has drained.
    sweeper.abort();
    pruner.abort();

    outcome
}
