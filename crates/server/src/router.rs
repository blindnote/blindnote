//! The HTTP router and the layers around it.
//!
//! Three things a request-handling Rust service normally gets for free are
//! deliberately absent, and all three are hard rules rather than oversights,
//! so a later "this was obviously missing" commit would be a regression:
//!
//! * **No `TraceLayer`, and no access log of any kind.** A note id must never
//!   be written to disk, and an access log is the easiest way to write one by
//!   accident.
//! * **No note id in any route path.** A path id lands in proxy logs, in error
//!   reports and in `Referer`. Ids travel in the request body instead.
//! * **No `NormalizePathLayer`.** Two spellings of one route are two cache
//!   keys and two chances for a header set to diverge.

use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower_governor::GovernorLayer;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::timeout::TimeoutLayer;

use crate::api::{self, ApiState};
use crate::assets;
use crate::headers;
use crate::rate_limit::Limiters;

/// Largest request body accepted on an API route.
///
/// A 64 KB note becomes a 64 KB + 16 byte ciphertext, which becomes about
/// 87.4 KB of unpadded base64url, plus the JSON around it. 96 KiB leaves room
/// for the wrapper and nothing like room for an attack.
const MAX_BODY_BYTES: usize = 96 * 1024;

/// How the API's layers are configured.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Layers {
    /// How long one request may take before it is cut off.
    pub(crate) request_timeout: Duration,
    /// How many requests may be in flight at once.
    pub(crate) max_concurrent_requests: usize,
}

/// Builds the application router.
///
/// `/healthz` sits outside the rate limiter and the body limit on purpose. A
/// throttled health probe makes an orchestrator believe the service is down,
/// and restarting a healthy process is a worse outcome than answering one
/// extra probe.
pub(crate) fn router(state: ApiState, limiters: &Limiters, layers: Layers) -> Router {
    let api = api::router(state, limiters).layer(
        ServiceBuilder::new()
            // Outside the concurrency limit, so a client being throttled does
            // not occupy one of the slots it is being throttled for.
            .layer(GovernorLayer::new(limiters.requests.clone()))
            .layer(ConcurrencyLimitLayer::new(layers.max_concurrent_requests))
            .layer(DefaultBodyLimit::max(MAX_BODY_BYTES)),
    );

    Router::new()
        .route("/healthz", get(healthz))
        .merge(api)
        // Last, so that its catch-all cannot shadow a route above it. Axum
        // prefers the more specific match, and a test holds that.
        .merge(assets::router())
        .layer(
            ServiceBuilder::new()
                // Outermost, so that even the 500 produced by a caught panic
                // leaves with the security headers on it.
                .layer(middleware::from_fn(headers::apply))
                // A panicking handler returns 500 and the process keeps
                // serving. Unwinding also runs the destructors that zeroize
                // key material, which `panic = "abort"` would skip.
                .layer(CatchPanicLayer::new())
                // 408 rather than the default 500: a request that ran out of
                // time is not a server fault, and a client that retries
                // should be told which of the two it was.
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    layers.request_timeout,
                )),
        )
}

/// Liveness probe.
///
/// Carries no version, build or state information. The probe is reachable from
/// the internet through the TLS terminator, and it has no reason to describe
/// the deployment to whoever asks.
async fn healthz() -> impl IntoResponse {
    (
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        "ok",
    )
}
