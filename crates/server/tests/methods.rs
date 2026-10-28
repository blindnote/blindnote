//! Which methods and which paths exist, and what the others answer.
//!
//! Two hard rules live here, and both are about something *not* existing.

mod common;

use common::{SMALLEST_CIPHERTEXT, Server};
use hyper::{Method, StatusCode};

/// No `GET` ever burns a note.
///
/// A link pasted into a chat window is fetched by a preview bot before the
/// recipient ever sees it. If `GET` burned, the recipient would open their own
/// note and be told it was already read.
///
/// 405 and not 404, and not a redirect: it has to fail loudly, so that a
/// crawler or a bot that tries it is visible rather than appearing to work.
#[tokio::test]
async fn get_on_a_burning_endpoint_is_405() {
    let server = Server::start().await;

    for path in ["/api/reveal", "/api/report", "/api/notes"] {
        let answer = server.send(Method::GET, path).await;

        assert_eq!(
            answer.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "GET {path} should be refused loudly"
        );
    }
}

#[tokio::test]
async fn a_405_says_which_method_is_allowed() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/api/reveal").await;

    assert_eq!(answer.header("allow"), Some("POST"));
}

#[tokio::test]
async fn head_and_delete_are_also_refused() {
    let server = Server::start().await;

    for method in [Method::HEAD, Method::DELETE, Method::PUT, Method::PATCH] {
        let answer = server.send(method.clone(), "/api/reveal").await;

        assert_eq!(
            answer.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} /api/reveal should be refused"
        );
    }
}

/// No route's path contains a note id.
///
/// `POST /api/notes/<id>/burn` is the shape this API would have had if nobody
/// had thought about it, and a path id lands in any access log, any proxy, any
/// future error report, and in `Referer`. There is no such route, and this
/// test is what keeps it that way: a 404 here means no handler was matched,
/// whereas a 405 would mean the path exists.
#[tokio::test]
async fn there_is_no_route_whose_path_carries_a_note_id() {
    let server = Server::start().await;
    let id = server.create(SMALLEST_CIPHERTEXT).await;

    for path in [
        format!("/api/notes/{id}"),
        format!("/api/notes/{id}/burn"),
        format!("/api/reveal/{id}"),
        format!("/{id}"),
    ] {
        let answer = server.send(Method::POST, &path).await;

        assert_eq!(
            answer.status,
            StatusCode::NOT_FOUND,
            "{path} matched a route, so a note id can reach a request line"
        );
    }
}

#[tokio::test]
async fn the_health_probe_answers_without_describing_the_deployment() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/healthz").await;

    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.body, "ok");
    assert_eq!(answer.header("cache-control"), Some("no-store"));
}

/// Nothing in a response names the software serving it.
#[tokio::test]
async fn no_response_advertises_the_server() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/healthz").await;

    assert_eq!(answer.header("server"), None);
    assert_eq!(answer.header("x-powered-by"), None);
}
