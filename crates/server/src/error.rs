//! Every failure the API can hand back, and the one that matters most.
//!
//! [`ApiError::Unavailable`] is the whole of the no-oracle rule. A note that
//! never existed, one that has expired and one that has already been read all
//! produce it, and it renders as one status, one body and one set of headers.
//! There is no variant to add a detail to, which is the point: a later
//! "helpful" distinction between them would have to change this type, and that
//! is a conversation rather than an accident.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use blindnote_store::StoreError;
use serde::Serialize;

/// Why a request could not be answered.
#[derive(Debug)]
pub(crate) enum ApiError {
    /// The note does not exist, has expired, or has already been read.
    Unavailable,

    /// The request was malformed.
    ///
    /// The detail is a fixed string chosen here, never anything derived from
    /// the request. An error message that quotes the body is an error message
    /// that can quote a note.
    BadRequest(&'static str),

    /// The request body was larger than the body limit.
    ///
    /// Separate from [`Self::BadRequest`] because the two are different
    /// instructions: one says "send something else", the other says "send
    /// less". A client that cannot tell them apart retries the same oversized
    /// body forever.
    TooLarge,

    /// The service is not accepting new notes.
    ///
    /// Distinct from [`Self::Internal`] because it is neither the client's
    /// fault nor a bug: the disk is nearly full, and reading an existing note
    /// still works. A client that is told "internal error" has no reason to
    /// believe the second part.
    Full,

    /// Something failed on our side.
    Internal,
}

/// The body shape every error shares.
#[derive(Serialize)]
struct Body {
    /// A short machine-readable tag.
    error: &'static str,
    /// What the client could do differently, when that is knowable.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'static str>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            // 404 rather than 410: "gone" would be an admission that it once
            // existed, which is exactly the distinction this response exists
            // to withhold.
            Self::Unavailable => (
                StatusCode::NOT_FOUND,
                Body {
                    error: "unavailable",
                    detail: None,
                },
            ),
            Self::BadRequest(detail) => (
                StatusCode::BAD_REQUEST,
                Body {
                    error: "bad_request",
                    detail: Some(detail),
                },
            ),
            Self::TooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Body {
                    error: "too_large",
                    detail: Some("the note is larger than 64 KB"),
                },
            ),
            // No `Retry-After`. How long is genuinely unknown -- it depends on
            // when the stored notes expire or are read -- and a guessed value
            // would be a worse answer than none.
            Self::Full => (
                StatusCode::SERVICE_UNAVAILABLE,
                Body {
                    error: "full",
                    detail: Some("new notes are not being accepted right now"),
                },
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body {
                    error: "internal",
                    detail: None,
                },
            ),
        };

        (
            status,
            [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
            Json(body),
        )
            .into_response()
    }
}

impl ApiError {
    /// Turns a JSON extraction failure into an answer that quotes nothing.
    ///
    /// Axum's own rejection explains the problem by showing the input, and the
    /// input here is, or was meant to be, a note. So the message is replaced
    /// with a fixed one -- but the *status* is kept when it was a size
    /// complaint, because "too large" is advice a client can act on and a
    /// status code quotes nothing.
    pub(crate) fn from_json_rejection(rejection: &JsonRejection, expected: &'static str) -> Self {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            return Self::TooLarge;
        }

        Self::BadRequest(expected)
    }
}

impl From<StoreError> for ApiError {
    /// Collapses every storage failure into one of three answers.
    ///
    /// Only [`StoreError::Unavailable`] survives as itself. Everything else is
    /// an internal failure that is logged here, once, and described to the
    /// client as nothing at all. None of these messages can carry a note id:
    /// ids are bound as SQL parameters and never interpolated, and the crypto
    /// and store error types redact or omit everything note-specific.
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Unavailable => Self::Unavailable,
            // Logged by the store, which has the numbers. Repeating it here
            // would put two lines in the log for one refusal.
            StoreError::Full { .. } => Self::Full,
            other => {
                tracing::error!(error = %other, "storage failure");
                Self::Internal
            }
        }
    }
}
