//! The HTTP router.
//!
//! Three things a request-handling Rust service normally gets for free are
//! deliberately absent here. All three are hard rules rather than oversights,
//! so a later "this was obviously missing" commit would be a regression:
//!
//! * **No `TraceLayer`, and no access log of any kind.** A note id must never
//!   be written to disk, and an access log is the easiest way to write one by
//!   accident (CLAUDE.md rule 10, section 8.5).
//! * **No note id in any route path.** A path id lands in proxy logs, in error
//!   reports and in `Referer`. Ids travel in the request body instead
//!   (CLAUDE.md section 8, API constraint 1).
//! * **No `NormalizePathLayer`.** Two spellings of one route are two cache
//!   keys and two chances for a header set to diverge.

use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::IntoResponse;
use axum::routing::get;

/// Builds the application router.
pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
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

#[cfg(test)]
mod tests {
    use super::router;
    use axum::body::Body;
    use axum::http::header::CACHE_CONTROL;
    use axum::http::{HeaderValue, Request, StatusCode};
    use tower::ServiceExt as _;

    fn get(uri: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("a valid request")
    }

    #[tokio::test]
    async fn healthz_answers_ok() {
        let response = router().oneshot(get("/healthz")).await.expect("infallible");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn healthz_is_never_cached() {
        let response = router().oneshot(get("/healthz")).await.expect("infallible");

        assert_eq!(
            response.headers().get(CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store"))
        );
    }

    #[tokio::test]
    async fn an_unknown_path_is_not_found() {
        let response = router()
            .oneshot(get("/does-not-exist"))
            .await
            .expect("infallible");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
