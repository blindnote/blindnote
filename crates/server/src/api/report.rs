//! `POST /api/report` -- destroy a note on an abuse report.
//!
//! The only takedown mechanism that does not break zero knowledge: anybody
//! holding a link can have the note it points at destroyed, and the server
//! decrypts nothing to do it. It grants no new power either -- whoever holds
//! the link could already have destroyed the note by reading it.
//!
//! This matters for durability, not only for politeness. A hosting provider
//! can suspend an entire account over abuse, and a suspended machine means
//! every pending note dies. An abuse team that meets a responsive operator is
//! part of keeping notes alive.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use blindnote_store::Id;
use serde::Deserialize;

use crate::api::ApiState;
use crate::error::ApiError;

/// The request body.
#[derive(Debug, Deserialize)]
pub(crate) struct Request {
    /// The note id, unpadded base64url, taken from the reported link.
    id: String,
}

/// Destroys a note, and says nothing about whether there was one.
///
/// Always `204`, for a note that existed and for one that never did. Every
/// other endpoint is careful not to be an existence oracle; a report endpoint
/// that answered "no such note" would be the easiest one in the API, being
/// unauthenticated and destructive only when it hits.
pub(crate) async fn handle(
    State(state): State<ApiState>,
    request: Result<Json<Request>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let Json(request) = request.map_err(|rejection| {
        ApiError::from_json_rejection(&rejection, "expected a JSON object with an id field")
    })?;

    let id = Id::from_base64url(&request.id)
        .map_err(|_| ApiError::BadRequest("an id is 22 unpadded base64url characters"))?;

    state.store.report(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
