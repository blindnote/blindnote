//! The operator statistics page.

mod common;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::{SMALLEST_CIPHERTEXT, Server};
use hyper::{Method, StatusCode};

/// A long-enough stand-in for the real random token.
const TOKEN: &str = "test-admin-token-7f3a9c21e5b84d06";

/// An `Authorization: Basic` value for `admin:<token>`.
fn basic(token: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("admin:{token}")))
}

/// Starts a server with the stats page enabled.
async fn with_admin() -> Server {
    Server::start_with(|config| config.admin_token = Some(TOKEN.to_owned())).await
}

/// Without a token the route does not exist: an ordinary 404, not a 401.
///
/// The admin surface is absent, not merely hidden -- there is nothing to probe.
#[tokio::test]
async fn the_page_is_absent_without_a_token() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/admin").await;

    assert_eq!(answer.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_request_without_credentials_is_challenged() {
    let server = with_admin().await;

    let answer = server.send(Method::GET, "/admin").await;

    assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
    assert!(
        answer
            .header("www-authenticate")
            .is_some_and(|value| value.starts_with("Basic")),
        "a 401 must ask for a credential"
    );
}

#[tokio::test]
async fn the_wrong_token_is_rejected() {
    let server = with_admin().await;

    let answer = server
        .send_with_header(Method::GET, "/admin", "authorization", &basic("wrong"))
        .await;

    assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
}

/// The right token shows the page, and the counts reflect what happened.
#[tokio::test]
async fn the_token_shows_the_counts() {
    let server = with_admin().await;

    // One note, created and still pending.
    server.create(SMALLEST_CIPHERTEXT).await;

    let answer = server
        .send_with_header(Method::GET, "/admin", "authorization", &basic(TOKEN))
        .await;

    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(
        answer.header("content-type"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(answer.header("cache-control"), Some("no-store"));
    assert!(
        answer.body.contains("<dt>Pending</dt><dd>1</dd>"),
        "the live pending count should be 1: {}",
        answer.body
    );
    assert!(
        answer.body.contains("<dt>Created</dt><dd>1</dd>"),
        "the created total should be 1: {}",
        answer.body
    );
    assert!(
        answer.body.contains("<dt>Read rate</dt><dd>—</dd>"),
        "with nothing read or expired yet, the read rate is undefined: {}",
        answer.body
    );
}

/// The page carries its own strict policy: the inline style is pinned by hash,
/// and nothing may run a script.
#[tokio::test]
async fn the_page_pins_its_style_and_forbids_scripts() {
    let server = with_admin().await;

    let answer = server
        .send_with_header(Method::GET, "/admin", "authorization", &basic(TOKEN))
        .await;

    let policy = answer
        .header("content-security-policy")
        .expect("a policy on the admin page");

    assert!(policy.starts_with("default-src 'none'"), "{policy}");
    assert!(policy.contains("style-src 'sha256-"), "{policy}");
    assert!(!policy.contains("'unsafe-inline'"), "{policy}");
    assert!(!policy.contains("script-src"), "{policy}");
}
