#![forbid(unsafe_code)]

//! Entry point for the `blindnote` binary.

use std::process::ExitCode;

use blindnote_server::{Config, run};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(%error, "invalid configuration");
            return ExitCode::FAILURE;
        }
    };

    match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "server stopped");
            ExitCode::FAILURE
        }
    }
}

/// Installs the log subscriber, at `info` unless `RUST_LOG` says otherwise.
///
/// What ends up in these logs is startup, shutdown, configuration failures and
/// aggregate counters. Never a note id, never a request body, never a client
/// address (CLAUDE.md rule 10, section 8.5, section 8.6). There is no access
/// log at all, which is why no `TraceLayer` is installed in `router()`.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
