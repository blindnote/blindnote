//! `POST /api/reveal` -- burn a note and hand back its ciphertext.
//!
//! `POST`, and only `POST`, because reading a note destroys it. No `GET` ever
//! burns: a link pasted into a chat window is fetched by a preview bot before
//! the recipient sees it, and a `GET` that burned would mean the recipient
//! never gets to read their own note. Axum answers `405` to any other method
//! on this path because only `post` is registered.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use blindnote_store::{Id, UnixTime};
use serde::{Deserialize, Serialize};

use crate::api::{ApiState, encode};
use crate::error::ApiError;

/// The request body.
#[derive(Debug, Deserialize)]
pub(crate) struct Request {
    /// The note id, unpadded base64url, taken from the link's fragment.
    id: String,
}

/// The response body.
#[derive(Debug, Serialize)]
pub(crate) struct Response {
    /// The browser's own ciphertext, unpadded base64url, for it to decrypt.
    ciphertext: String,
}

/// Burns a note and returns what the browser must decrypt.
///
/// By the time this returns, the note's key has been deleted. There is no
/// second chance and no undo, which is the product.
pub(crate) async fn handle(
    State(state): State<ApiState>,
    request: Result<Json<Request>, JsonRejection>,
) -> Result<Json<Response>, ApiError> {
    let Json(request) = request.map_err(|rejection| {
        ApiError::from_json_rejection(&rejection, "expected a JSON object with an id field")
    })?;

    // A syntactically impossible id is a client bug, not a question about a
    // note, so it gets its own answer. It tells a prober nothing: the shape of
    // an id is public, and no note is consulted to decide this.
    let id = Id::from_base64url(&request.id)
        .map_err(|_| ApiError::BadRequest("an id is 22 unpadded base64url characters"))?;

    let ciphertext = state.store.burn(id, UnixTime::now()).await?;

    Ok(Json(Response {
        ciphertext: encode(&ciphertext),
    }))
}
