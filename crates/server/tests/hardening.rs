//! Headers, limits and refusals.

mod common;

use common::{SMALLEST_CIPHERTEXT, Server, ciphertext_of};
use hyper::{Method, StatusCode};

#[tokio::test]
async fn every_response_carries_the_security_headers() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/healthz").await;

    assert_eq!(
        answer.header("strict-transport-security"),
        Some("max-age=63072000; includeSubDomains; preload")
    );
    assert_eq!(answer.header("referrer-policy"), Some("no-referrer"));
    assert_eq!(answer.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(
        answer.header("cross-origin-opener-policy"),
        Some("same-origin")
    );
    assert_eq!(
        answer.header("cross-origin-embedder-policy"),
        Some("require-corp")
    );
    assert_eq!(
        answer.header("cross-origin-resource-policy"),
        Some("same-origin")
    );
    assert!(answer.header("permissions-policy").is_some());
}

/// The policy is on every response, and it denies by default.
#[tokio::test]
async fn every_response_carries_a_policy_that_denies_by_default() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/healthz").await;
    let policy = answer
        .header("content-security-policy")
        .expect("a content security policy");

    assert!(policy.starts_with("default-src 'none';"), "{policy}");
    for directive in [
        "connect-src 'self'",
        "object-src 'none'",
        "frame-ancestors 'none'",
        "base-uri 'none'",
        "require-trusted-types-for 'script'",
    ] {
        assert!(
            policy.contains(directive),
            "the policy is missing {directive}: {policy}"
        );
    }
}

/// Nothing in the policy opens a door.
///
/// `'unsafe-inline'` or a scheme source would make the hashes decorative, and
/// the policy is the reason injected code cannot send a note anywhere.
#[tokio::test]
async fn the_policy_contains_no_escape_hatch() {
    let server = Server::start().await;

    let answer = server.send(Method::GET, "/healthz").await;
    let policy = answer.header("content-security-policy").expect("a policy");

    for hatch in [
        "'unsafe-inline'",
        "'unsafe-eval'",
        "'wasm-unsafe-eval'",
        "'strict-dynamic'",
        "http:",
        "https:",
        "data:",
        "*",
    ] {
        assert!(
            !policy.contains(hatch),
            "the policy contains {hatch}: {policy}"
        );
    }
}

/// An error response is still a response.
///
/// Headers added by a layer that sits inside the error path would be missing
/// exactly when something has gone wrong, which is the worst time to lose them.
#[tokio::test]
async fn an_error_response_carries_them_too() {
    let server = Server::start().await;

    let answer = server.post_json("/api/reveal", "not json at all").await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert_eq!(answer.header("referrer-policy"), Some("no-referrer"));
}

/// A malformed body is never quoted back.
///
/// Axum's own JSON rejection explains the problem by showing the input, and
/// the input here is, or was meant to be, a note. The handler maps the
/// rejection to a fixed message instead.
#[tokio::test]
async fn a_rejected_body_is_not_echoed() {
    let server = Server::start().await;
    let secret = "this-string-must-not-come-back";

    let answer = server
        .post_json(
            "/api/notes",
            &format!(r#"{{"ciphertext": {secret:?}, "ttl": 99}}"#),
        )
        .await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
    assert!(
        !answer.body.contains(secret),
        "the error quoted the request body: {}",
        answer.body
    );
}

#[tokio::test]
async fn a_lifetime_outside_the_menu_is_refused() {
    let server = Server::start().await;

    for ttl in ["7d", "365d", "0s", "forever"] {
        let answer = server
            .post_json(
                "/api/notes",
                &format!(r#"{{"ciphertext":"{SMALLEST_CIPHERTEXT}","ttl":"{ttl}"}}"#),
            )
            .await;

        assert_eq!(
            answer.status,
            StatusCode::BAD_REQUEST,
            "a client asked for {ttl} and was not refused"
        );
    }
}

#[tokio::test]
async fn the_three_offered_lifetimes_are_accepted() {
    let server = Server::start().await;

    for ttl in ["10m", "1h", "24h"] {
        let answer = server
            .post_json(
                "/api/notes",
                &format!(r#"{{"ciphertext":"{SMALLEST_CIPHERTEXT}","ttl":"{ttl}"}}"#),
            )
            .await;

        assert_eq!(answer.status, StatusCode::OK, "{ttl} should be on the menu");
    }
}

#[tokio::test]
async fn a_payload_over_the_ceiling_is_refused() {
    let server = Server::start().await;

    // One byte past 64 KB plus the framing headroom.
    let answer = server
        .post_json(
            "/api/notes",
            &format!(r#"{{"ciphertext":"{}"}}"#, ciphertext_of(64 * 1024 + 513)),
        )
        .await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
}

/// A full-size note plus the browser's framing has to fit.
///
/// The browser prepends a 12-byte nonce and AES-GCM appends a 16-byte tag, so
/// the largest note the composer accepts arrives as 64 KB + 28 bytes. A ceiling
/// set at plaintext plus tag would have refused it, and the user would have
/// been told their note was too large when it was not.
#[tokio::test]
async fn a_full_size_note_with_its_framing_is_accepted() {
    let server = Server::start().await;

    let answer = server
        .post_json(
            "/api/notes",
            &format!(r#"{{"ciphertext":"{}"}}"#, ciphertext_of(64 * 1024 + 28)),
        )
        .await;

    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
}

/// Past the body limit the request is cut off before a handler sees it.
#[tokio::test]
async fn an_oversized_body_is_refused_by_the_layer() {
    let server = Server::start().await;

    let answer = server
        .post_json(
            "/api/notes",
            &format!(r#"{{"ciphertext":"{}"}}"#, ciphertext_of(256 * 1024)),
        )
        .await;

    assert_eq!(answer.status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn an_empty_ciphertext_is_refused() {
    let server = Server::start().await;

    let answer = server.post_json("/api/notes", r#"{"ciphertext":""}"#).await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn padded_base64_is_refused() {
    let server = Server::start().await;

    let answer = server
        .post_json("/api/notes", r#"{"ciphertext":"AAAAAAAAAAAAAAAAAAAAAA=="}"#)
        .await;

    assert_eq!(answer.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_id_of_the_wrong_shape_is_refused() {
    let server = Server::start().await;

    for id in ["", "short", "AAAAAAAAAAAAAAAAAAAAAAAA", "not base64!"] {
        let answer = server
            .post_json("/api/reveal", &format!(r#"{{"id":{id:?}}}"#))
            .await;

        assert_eq!(
            answer.status,
            StatusCode::BAD_REQUEST,
            "{id:?} is not a possible id and should be refused as malformed"
        );
    }
}

/// The rate limiter actually limits.
///
/// Configured down to one request per second with no burst, so the second
/// request in the same instant has to be refused. The point is not the
/// numbers; it is that the layer is wired in and keyed on something.
#[tokio::test]
async fn too_many_requests_are_refused() {
    let server = Server::start_with(|config| {
        config.requests_per_second = 1;
        config.request_burst = 1;
    })
    .await;

    let body = r#"{"id":"_____________________w"}"#;
    let first = server.post_json("/api/reveal", body).await;
    let second = server.post_json("/api/reveal", body).await;

    assert_eq!(
        first.status,
        StatusCode::NOT_FOUND,
        "the first should get through"
    );
    assert_eq!(second.status, StatusCode::TOO_MANY_REQUESTS);
}

/// The hourly create quota is separate from the burst limit.
///
/// A flood and a slow drip are different threats: one would exhaust the
/// server, the other would fill the disk over a day. Here the request limit is
/// left generous and only the create quota is tightened, so a refusal can only
/// have come from the quota.
#[tokio::test]
async fn the_create_quota_is_enforced_separately() {
    let server = Server::start_with(|config| {
        config.requests_per_second = 1000;
        config.request_burst = 1000;
        config.creates_per_hour = 1;
    })
    .await;

    let body = format!(r#"{{"ciphertext":"{SMALLEST_CIPHERTEXT}"}}"#);
    let first = server.post_json("/api/notes", &body).await;
    let second = server.post_json("/api/notes", &body).await;

    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(second.status, StatusCode::TOO_MANY_REQUESTS);
}

/// The health probe is not rate limited.
///
/// A throttled probe makes an orchestrator believe the service is down, and
/// restarting a healthy process is worse than answering one extra probe.
#[tokio::test]
async fn the_health_probe_is_not_rate_limited() {
    let server = Server::start_with(|config| {
        config.requests_per_second = 1;
        config.request_burst = 1;
    })
    .await;

    for attempt in 1..=5 {
        let answer = server.send(Method::GET, "/healthz").await;

        assert_eq!(
            answer.status,
            StatusCode::OK,
            "probe {attempt} was throttled"
        );
    }
}
