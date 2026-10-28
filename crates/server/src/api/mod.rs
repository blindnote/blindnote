//! The HTTP API.
//!
//! Three constraints shape every route here, and each one is the opposite of
//! what gets written by default:
//!
//! 1. **No note id ever appears in a path.** `POST /api/notes/<id>/burn` reads
//!    naturally and is wrong: a path id lands in any access log, any proxy,
//!    any future error report, and in `Referer`. Ids travel in the request
//!    body, so every endpoint is a fixed path.
//! 2. **`GET` on a burning endpoint is `405`, not `404` and not a redirect.**
//!    It has to fail loudly, so that a crawler or a link-preview bot that
//!    tries it is visible rather than appearing to work. Axum gives this for
//!    free by registering only `post`, and a test holds it.
//! 3. **Missing, expired and already-read are one response.** One shared
//!    error, from one branch of one function in the store, rendered by one arm
//!    of [`crate::error::ApiError`].

mod create;
mod report;
mod reveal;

use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::routing::post;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use blindnote_store::Store;
use tower_governor::GovernorLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::error::ApiError;
use crate::rate_limit::Limiters;

/// What every handler needs.
#[derive(Clone, Debug)]
pub(crate) struct ApiState {
    /// The two database files.
    pub(crate) store: Store,
}

/// Builds the API routes.
///
/// The create route carries a second limiter on top of the shared one: the
/// shared limiter stops a flood, and the hourly quota stops a slow drip that
/// would fill the disk over a day. They are different threats and different
/// numbers.
pub(crate) fn router(state: ApiState, limiters: &Limiters) -> Router {
    Router::new()
        .route(
            "/api/notes",
            post(create::handle).layer(GovernorLayer::new(limiters.creates.clone())),
        )
        .route("/api/reveal", post(reveal::handle))
        .route("/api/report", post(report::handle))
        // Scoped to these routes rather than applied globally, because the
        // embedded client assets will want the opposite value. A test found
        // this missing: the *error* responses set it themselves, so only the
        // successful ones were cacheable -- which is to say, only the ones
        // carrying a note id or a note's ciphertext.
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .with_state(state)
}

/// Decodes a wire value from unpadded base64url.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] with a fixed message. The offending value
/// is never quoted back: it is, or was meant to be, a note.
fn decode(value: &str) -> Result<Vec<u8>, ApiError> {
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ApiError::BadRequest("expected unpadded base64url"))
}

/// Encodes a wire value as unpadded base64url.
fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn encoding_round_trips() {
        let bytes = b"some bytes".to_vec();

        assert_eq!(decode(&encode(&bytes)).expect("decodes"), bytes);
    }

    #[test]
    fn padded_base64_is_rejected() {
        assert!(decode("YQ==").is_err(), "the wire format is unpadded");
    }

    #[test]
    fn standard_alphabet_base64_is_rejected() {
        // '+' and '/' belong to the standard alphabet; url-safe uses '-' and '_'.
        assert!(decode("a+b/c").is_err());
    }
}
