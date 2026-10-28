//! Creating, reading and reporting a note over HTTP.

mod common;

use common::{SMALLEST_CIPHERTEXT, Server, ciphertext_of};
use hyper::StatusCode;

/// A note id that is syntactically valid and belongs to nothing.
const UNKNOWN_ID: &str = "_____________________w";

#[tokio::test]
async fn a_note_comes_back_exactly_as_it_went_in() {
    let server = Server::start().await;
    let ciphertext = ciphertext_of(1024);
    let id = server.create(&ciphertext).await;

    let answer = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{id}"}}"#))
        .await;

    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.json()["ciphertext"], ciphertext);
}

#[tokio::test]
async fn an_id_is_22_characters_of_base64url() {
    let server = Server::start().await;

    let id = server.create(SMALLEST_CIPHERTEXT).await;

    assert_eq!(id.len(), 22, "128 bits, unpadded base64url");
    assert!(
        id.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "an id has to survive a URL fragment unescaped: {id}"
    );
}

#[tokio::test]
async fn reading_a_note_twice_fails_the_second_time() {
    let server = Server::start().await;
    let id = server.create(SMALLEST_CIPHERTEXT).await;
    let body = format!(r#"{{"id":"{id}"}}"#);

    server.post_json("/api/reveal", &body).await;
    let second = server.post_json("/api/reveal", &body).await;

    assert_eq!(second.status, StatusCode::NOT_FOUND);
}

/// The no-oracle rule, asserted as byte equality.
///
/// A note that was read and a note that never existed must be
/// indistinguishable. Comparing the whole answer -- status, body and the
/// headers a client can see -- is the cheapest way to notice if a later change
/// adds a "helpful" difference between them.
#[tokio::test]
async fn a_read_note_and_an_unknown_note_answer_identically() {
    let server = Server::start().await;
    let id = server.create(SMALLEST_CIPHERTEXT).await;
    let read_body = format!(r#"{{"id":"{id}"}}"#);
    server.post_json("/api/reveal", &read_body).await;

    let burned = server.post_json("/api/reveal", &read_body).await;
    let unknown = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{UNKNOWN_ID}"}}"#))
        .await;

    assert_eq!(burned.status, unknown.status);
    assert_eq!(burned.body, unknown.body);
    assert_eq!(
        burned.header("content-type"),
        unknown.header("content-type")
    );
    assert_eq!(
        burned.header("content-length"),
        unknown.header("content-length")
    );
}

/// The unavailable answer says nothing beyond being unavailable.
#[tokio::test]
async fn the_unavailable_answer_carries_no_detail() {
    let server = Server::start().await;

    let answer = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{UNKNOWN_ID}"}}"#))
        .await;

    assert_eq!(answer.status, StatusCode::NOT_FOUND);
    assert_eq!(answer.json()["error"], "unavailable");
    assert!(
        answer.json().get("detail").is_none(),
        "an unavailable note has no detail to give: {}",
        answer.body
    );
}

#[tokio::test]
async fn a_reported_note_can_no_longer_be_read() {
    let server = Server::start().await;
    let id = server.create(SMALLEST_CIPHERTEXT).await;
    let body = format!(r#"{{"id":"{id}"}}"#);

    let report = server.post_json("/api/report", &body).await;
    assert_eq!(report.status, StatusCode::NO_CONTENT);

    let reveal = server.post_json("/api/reveal", &body).await;
    assert_eq!(reveal.status, StatusCode::NOT_FOUND);
}

/// Reporting is not an existence check.
///
/// It is unauthenticated and destructive, so an answer that distinguished "no
/// such note" would be the easiest oracle in the API.
#[tokio::test]
async fn reporting_an_unknown_note_looks_like_any_other_report() {
    let server = Server::start().await;
    let known = server.create(SMALLEST_CIPHERTEXT).await;

    let real = server
        .post_json("/api/report", &format!(r#"{{"id":"{known}"}}"#))
        .await;
    let unknown = server
        .post_json("/api/report", &format!(r#"{{"id":"{UNKNOWN_ID}"}}"#))
        .await;

    assert_eq!(real.status, StatusCode::NO_CONTENT);
    assert_eq!(unknown.status, real.status);
    assert_eq!(unknown.body, real.body);
}

#[tokio::test]
async fn two_notes_do_not_interfere() {
    let server = Server::start().await;
    let first_text = ciphertext_of(64);
    let second_text = ciphertext_of(128);
    let first = server.create(&first_text).await;
    let second = server.create(&second_text).await;

    server
        .post_json("/api/reveal", &format!(r#"{{"id":"{first}"}}"#))
        .await;
    let answer = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{second}"}}"#))
        .await;

    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.json()["ciphertext"], second_text);
}

#[tokio::test]
async fn every_api_answer_is_uncacheable() {
    let server = Server::start().await;
    let id = server.create(SMALLEST_CIPHERTEXT).await;

    let created = server
        .post_json(
            "/api/notes",
            &format!(r#"{{"ciphertext":"{SMALLEST_CIPHERTEXT}"}}"#),
        )
        .await;
    let revealed = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{id}"}}"#))
        .await;
    let missing = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{UNKNOWN_ID}"}}"#))
        .await;

    for answer in [created, revealed, missing] {
        assert_eq!(
            answer.header("cache-control"),
            Some("no-store"),
            "a note must not be cacheable: {answer:?}"
        );
    }
}
