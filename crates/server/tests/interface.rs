//! Serving the embedded interface.
//!
//! Every test here is skipped when nothing is embedded, which is the case in a
//! checkout where `client/dist` has not been built. The alternative would be
//! to make `cargo test` depend on `npm`, and a Rust change should not need a
//! Node toolchain to prove itself.

mod common;

use common::{SMALLEST_CIPHERTEXT, Server};
use hyper::{Method, StatusCode};

/// Whether this binary was built with an interface in it.
async fn has_an_interface(server: &Server) -> bool {
    server.send(Method::GET, "/").await.status == StatusCode::OK
}

#[tokio::test]
async fn the_root_serves_the_page() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let answer = server.send(Method::GET, "/").await;

    assert_eq!(
        answer.header("content-type"),
        Some("text/html; charset=utf-8")
    );
    assert!(
        answer.body.contains("<div id=\"root\">"),
        "not the application's page"
    );
}

/// The page itself is never cached.
///
/// It names the content-hashed asset files, and the policy that will pin their
/// hashes is served with it. A stale copy would reference scripts the current
/// policy does not allow -- and it is a few hundred bytes compressed, so
/// re-fetching it costs nothing and guarantees a visitor runs what was
/// published.
#[tokio::test]
async fn the_page_is_never_cached() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let answer = server.send(Method::GET, "/").await;

    assert_eq!(answer.header("cache-control"), Some("no-store"));
}

/// Content-hashed files are immutable; nothing else claims to be.
#[tokio::test]
async fn hashed_assets_are_immutable_and_fonts_are_not() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let page = server.send(Method::GET, "/").await.body;
    let script = find_src(&page, ".js").expect("the page references a script");

    let answer = server.send(Method::GET, &script).await;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(
        answer.header("cache-control"),
        Some("public, max-age=31536000, immutable")
    );

    let font = server
        .send(Method::GET, "/fonts/raleway-latin-400-normal.woff2")
        .await;
    assert_eq!(font.status, StatusCode::OK);
    assert_eq!(
        font.header("cache-control"),
        Some("public, max-age=86400, must-revalidate"),
        "a name without a content hash must not be trusted for a year"
    );
}

#[tokio::test]
async fn an_asset_answers_304_to_its_own_etag() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let first = server.send(Method::GET, "/").await;
    let etag = first.header("etag").expect("an ETag").to_owned();

    let second = server
        .send_with_header(Method::GET, "/", "if-none-match", &etag)
        .await;

    assert_eq!(second.status, StatusCode::NOT_MODIFIED);
    assert!(second.body.is_empty(), "a 304 carries no body");
}

/// A client that asks for Brotli gets it; one that does not, does not.
#[tokio::test]
async fn brotli_is_served_only_when_it_is_asked_for() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let page = server.send(Method::GET, "/").await.body;
    let script = find_src(&page, ".js").expect("the page references a script");

    let compressed = server
        .send_with_header(Method::GET, &script, "accept-encoding", "gzip, br")
        .await;
    assert_eq!(compressed.header("content-encoding"), Some("br"));
    assert_eq!(compressed.header("vary"), Some("accept-encoding"));

    let plain = server
        .send_with_header(Method::GET, &script, "accept-encoding", "gzip")
        .await;
    assert_eq!(plain.status, StatusCode::OK);
    assert_eq!(
        plain.header("content-encoding"),
        None,
        "a client that did not ask for Brotli must not be sent it"
    );
}

/// A woff2 is already Brotli inside, so a second pass is not stored.
#[tokio::test]
async fn a_font_is_not_compressed_twice() {
    let server = Server::start().await;
    if !has_an_interface(&server).await {
        return;
    }

    let answer = server
        .send_with_header(
            Method::GET,
            "/fonts/raleway-latin-400-normal.woff2",
            "accept-encoding",
            "br",
        )
        .await;

    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(
        answer.header("content-encoding"),
        None,
        "compressing a woff2 again gains nothing and doubles the binary"
    );
}

/// The interface's catch-all must not shadow the API.
#[tokio::test]
async fn the_api_still_answers_with_the_interface_mounted() {
    let server = Server::start().await;

    let id = server.create(SMALLEST_CIPHERTEXT).await;
    let answer = server
        .post_json("/api/reveal", &format!(r#"{{"id":"{id}"}}"#))
        .await;

    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.header("cache-control"), Some("no-store"));
}

#[tokio::test]
async fn an_unknown_file_is_not_found() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/no-such-file.js").await;

    assert_eq!(answer.status, StatusCode::NOT_FOUND);
}

/// Finds the first `/assets/...<suffix>` the page references.
fn find_src(page: &str, suffix: &str) -> Option<String> {
    let start = page.find("/assets/")?;
    let rest = page.get(start..)?;
    let end = rest.find(suffix)? + suffix.len();

    Some(rest.get(..end)?.to_owned())
}
