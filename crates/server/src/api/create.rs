//! `POST /api/notes` -- store a note the browser has already encrypted.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use blindnote_store::{NewNote, UnixTime};
use serde::{Deserialize, Serialize};

use crate::api::{ApiState, decode, encode};
use crate::error::ApiError;

/// Largest note the product offers, counted in the plaintext a user types.
const MAX_PLAINTEXT_BYTES: usize = 64 * 1024;

/// Room for whatever framing the client wraps around its ciphertext.
///
/// The browser currently sends `nonce ‖ ciphertext ‖ tag`, which is 28 bytes
/// of overhead. The exact format is deliberately not this server's business --
/// it cannot read the payload and has no reason to parse it -- so the bound is
/// a round number above anything a sane framing would add, rather than a sum
/// that has to be corrected every time the client changes.
const CLIENT_FRAMING_HEADROOM: usize = 512;

/// Largest opaque payload we accept.
///
/// The user-facing 64 KB limit is enforced in the browser, before encryption,
/// which is the only place it can produce a useful message. This is the
/// backstop.
const MAX_PAYLOAD_BYTES: usize = MAX_PLAINTEXT_BYTES + CLIENT_FRAMING_HEADROOM;

/// Smallest payload we accept: an AES-GCM tag, and nothing else.
///
/// This also rejects an empty string, which is the shape a broken client sends
/// when its encryption step silently did nothing -- a failure mode worth
/// catching here rather than storing.
const MIN_PAYLOAD_BYTES: usize = 16;

/// How long a note may live.
///
/// An enumeration rather than a number of seconds, so that the set of
/// permitted lifetimes is stated once, here, and cannot be widened by a
/// request. A client asking for a year gets a `400`, not a year.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
pub(crate) enum Ttl {
    /// Ten minutes.
    #[serde(rename = "10m")]
    TenMinutes,
    /// One hour.
    #[serde(rename = "1h")]
    OneHour,
    /// Twenty-four hours, and the default.
    ///
    /// Also the maximum in this version. A short ceiling narrows the abuse
    /// window and the data-loss window at the same time.
    #[default]
    #[serde(rename = "24h")]
    OneDay,
}

impl Ttl {
    /// The lifetime in seconds.
    const fn seconds(self) -> u32 {
        match self {
            Self::TenMinutes => 10 * 60,
            Self::OneHour => 60 * 60,
            Self::OneDay => 24 * 60 * 60,
        }
    }
}

/// The request body.
#[derive(Debug, Deserialize)]
pub(crate) struct Request {
    /// The browser's ciphertext, unpadded base64url.
    ciphertext: String,
    /// How long the note should live. Defaults to 24 hours.
    #[serde(default)]
    ttl: Ttl,
}

/// The response body.
#[derive(Debug, Serialize)]
pub(crate) struct Response {
    /// The note id, unpadded base64url, for the fragment of the link.
    id: String,
}

/// Stores a note and returns its id.
///
/// The id goes back in the body and the client puts it in a URL *fragment*
/// together with the key. Neither ever reaches this server again except as the
/// body of a later request.
pub(crate) async fn handle(
    State(state): State<ApiState>,
    request: Result<Json<Request>, JsonRejection>,
) -> Result<Json<Response>, ApiError> {
    // The rejection is mapped by hand rather than allowed through. Axum's own
    // message is helpful and quotes the body back, and the body is a note.
    let Json(request) = request.map_err(|rejection| {
        ApiError::from_json_rejection(&rejection, "expected a JSON object with a ciphertext field")
    })?;

    let ciphertext = decode(&request.ciphertext)?;
    if ciphertext.len() < MIN_PAYLOAD_BYTES {
        return Err(ApiError::BadRequest(
            "the ciphertext is too short to be one",
        ));
    }
    if ciphertext.len() > MAX_PAYLOAD_BYTES {
        return Err(ApiError::BadRequest("the note is larger than 64 KB"));
    }

    let id = state
        .store
        .create(NewNote {
            client_ciphertext: ciphertext,
            expires_at: UnixTime::now().plus_seconds(request.ttl.seconds()),
            // One read. The column allows more and the store honours it, but
            // nothing in this version offers it.
            views_left: 1,
        })
        .await?;

    Ok(Json(Response {
        id: encode(id.as_bytes()),
    }))
}

#[cfg(test)]
mod tests {
    use super::{MAX_PAYLOAD_BYTES, MAX_PLAINTEXT_BYTES, Ttl};

    #[test]
    fn the_default_lifetime_is_a_day() {
        assert_eq!(Ttl::default(), Ttl::OneDay);
        assert_eq!(Ttl::default().seconds(), 86_400);
    }

    #[test]
    fn no_lifetime_exceeds_the_version_1_ceiling() {
        for ttl in [Ttl::TenMinutes, Ttl::OneHour, Ttl::OneDay] {
            assert!(ttl.seconds() <= 86_400, "{ttl:?} is longer than 24 hours");
        }
    }

    #[test]
    fn the_lifetimes_are_the_three_the_product_offers() {
        assert_eq!(Ttl::TenMinutes.seconds(), 600);
        assert_eq!(Ttl::OneHour.seconds(), 3600);
        assert_eq!(Ttl::OneDay.seconds(), 86_400);
    }

    #[test]
    fn a_lifetime_is_named_on_the_wire_not_counted() {
        // A client cannot ask for an arbitrary number of seconds, which is the
        // reason this is an enumeration.
        assert!(serde_json::from_str::<Ttl>("\"10m\"").is_ok());
        assert!(serde_json::from_str::<Ttl>("600").is_err());
        assert!(serde_json::from_str::<Ttl>("\"365d\"").is_err());
    }

    #[test]
    fn the_ciphertext_ceiling_allows_a_full_size_note() {
        assert_eq!(MAX_CIPHERTEXT_BYTES, MAX_PLAINTEXT_BYTES + 16);
    }
}
