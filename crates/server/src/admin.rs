//! The operator statistics page.
//!
//! A single authed, read-only page at `/admin` showing the anonymous aggregate
//! counters. It exists only when `BLINDNOTE_ADMIN_TOKEN` is set; without it the
//! route is not mounted, so a request for `/admin` is an ordinary 404 -- the
//! admin surface is absent, not merely hidden behind a 401.
//!
//! What the page can show is bounded by what this process knows, which is
//! nothing about any individual note: only totals. So even a leaked page leaks
//! a handful of integers, never a note. The authentication (HTTP Basic, over a
//! constant-time comparison of SHA-256 digests so neither the token nor its
//! length leaks through timing) and the strict, script-free content security
//! policy are there anyway, because an operator page anyone could read is still
//! a mild information leak and a bad habit.

use std::sync::{Arc, LazyLock};

use axum::Router;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use blindnote_store::{Stats, Store};
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;
use tower_governor::GovernorLayer;

use crate::rate_limit::Limiters;

/// The path the stats page is served at.
pub(crate) const PATH: &str = "/admin";

/// The page's inline stylesheet. Fixed, so its hash can be pinned in the CSP.
const STYLE: &str = "\
:root{color-scheme:light dark}\
*{box-sizing:border-box}\
body{margin:0;min-height:100vh;display:grid;place-items:center;\
font:16px/1.5 system-ui,-apple-system,sans-serif;background:#faf7f3;color:#2a1a0f}\
main{width:min(30rem,92vw);padding:2rem}\
h1{margin:0;font-size:1.4rem}\
.sub{margin:.25rem 0 1.5rem;color:#6b5444}\
dl{margin:0;display:grid;gap:.5rem}\
dl>div{display:flex;justify-content:space-between;align-items:baseline;gap:1rem;\
padding:.75rem 1rem;border:1px solid #e7ddd3;border-radius:.75rem;background:#fff}\
dt{color:#6b5444}\
dd{margin:0;font-size:1.5rem;font-weight:600;font-variant-numeric:tabular-nums}\
.note{margin-top:1.5rem;font-size:.8rem;line-height:1.4;color:#8a7360}\
@media(prefers-color-scheme:dark){\
body{background:#1a120d;color:#fef8f2}\
.sub,dt,.note{color:#d9c2b0}\
dl>div{background:#2e2018;border-color:#3f2c22}}";

/// The content security policy for the stats page.
///
/// Its own policy, separate from the application's: it still denies scripts,
/// framing, forms and every third party, and the one thing it allows -- the
/// inline stylesheet -- is pinned by the hash of that stylesheet's bytes, so no
/// `'unsafe-inline'` is granted. `headers::apply` serves this on the admin path
/// and the application policy everywhere else.
static CSP: LazyLock<HeaderValue> = LazyLock::new(|| {
    let hash = STANDARD.encode(Sha256::digest(STYLE.as_bytes()));
    let policy = format!(
        "default-src 'none'; style-src 'sha256-{hash}'; base-uri 'none'; \
         form-action 'none'; frame-ancestors 'none'"
    );
    HeaderValue::from_str(&policy)
        .unwrap_or_else(|_| HeaderValue::from_static("default-src 'none'"))
});

/// The CSP value for the stats page.
pub(crate) fn csp_header() -> HeaderValue {
    CSP.clone()
}

/// What the admin handler needs.
#[derive(Clone)]
struct AdminState {
    store: Store,
    token: Arc<str>,
}

/// Builds the admin route, or nothing when no token is configured.
pub(crate) fn router(store: Store, token: Option<String>, limiters: &Limiters) -> Option<Router> {
    let token = token?;
    let state = AdminState {
        store,
        token: Arc::from(token),
    };
    Some(
        Router::new()
            .route(PATH, get(handle))
            // The token is a secret long enough that guessing is hopeless, but
            // there is no reason to let anyone hammer the endpoint either.
            .layer(GovernorLayer::new(limiters.requests.clone()))
            .with_state(state),
    )
}

/// Serves the stats page, after a constant-time check of the Basic credential.
async fn handle(State(state): State<AdminState>, headers: HeaderMap) -> Response {
    if !authorized(&headers, &state.token) {
        return challenge();
    }

    match state.store.stats().await {
        Ok(counts) => page(&counts),
        Err(error) => {
            tracing::error!(%error, "could not read the usage stats");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Whether the request carries the configured token in a Basic credential.
///
/// The username is ignored and the password is the token. Both sides are
/// reduced to a fixed-length SHA-256 digest and compared in constant time, so
/// the comparison leaks neither the token's bytes nor its length.
fn authorized(headers: &HeaderMap, token: &str) -> bool {
    let Some(credential) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "))
    else {
        return false;
    };

    let Ok(decoded) = STANDARD.decode(credential.trim()) else {
        return false;
    };

    // Everything after the first colon is the password; the username is not
    // used. A credential with no colon is malformed and rejected.
    let Some(colon) = decoded.iter().position(|&byte| byte == b':') else {
        return false;
    };
    let provided = &decoded[colon + 1..];

    let provided_hash = Sha256::digest(provided);
    let token_hash = Sha256::digest(token.as_bytes());
    bool::from(provided_hash.as_slice().ct_eq(token_hash.as_slice()))
}

/// A 401 that asks the browser for a Basic credential.
fn challenge() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [
            (
                WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"blindnote\", charset=\"UTF-8\""),
            ),
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
        ],
        "unauthorized",
    )
        .into_response()
}

/// The read rate as a display string: read / (read + expired), or an em dash
/// when no note has ended yet. Derived from the counters, with no per-note data
/// behind it; integer arithmetic, so no float and no rounding surprises.
fn read_rate(read: u64, expired: u64) -> String {
    match read.saturating_mul(100).checked_div(read + expired) {
        None => "—".to_owned(),
        Some(percent) => format!("{percent}%"),
    }
}

/// Renders the stats page.
fn page(stats: &Stats) -> Response {
    let Stats {
        created,
        read,
        expired,
        reported,
        pending,
    } = *stats;

    let rate = read_rate(read, expired);

    let body = format!(
        "<!doctype html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<meta name=\"robots\" content=\"noindex\">\n\
<title>BlindNote stats</title>\n\
<style>{STYLE}</style>\n\
</head>\n\
<body>\n\
<main>\n\
<h1>BlindNote</h1>\n\
<p class=\"sub\">Anonymous aggregate counters. No per-note data.</p>\n\
<dl>\n\
<div><dt>Pending</dt><dd>{pending}</dd></div>\n\
<div><dt>Created</dt><dd>{created}</dd></div>\n\
<div><dt>Read</dt><dd>{read}</dd></div>\n\
<div><dt>Expired unread</dt><dd>{expired}</dd></div>\n\
<div><dt>Reported</dt><dd>{reported}</dd></div>\n\
<div><dt>Read rate</dt><dd>{rate}</dd></div>\n\
</dl>\n\
<p class=\"note\">Read rate is read / (read + expired unread): of the notes that ended, the \
share that were read. Pending is a live count. Created, read, expired and reported are \
cumulative totals, persisted on each sweep, so they are approximate across a restart.</p>\n\
</main>\n\
</body>\n\
</html>\n"
    );

    (
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            ),
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
        ],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::{STYLE, authorized, csp_header, read_rate};
    use axum::http::HeaderMap;
    use axum::http::header::AUTHORIZATION;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;

    /// Builds an `Authorization: Basic` header for `user:pass`.
    fn basic(user: &str, pass: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let encoded = STANDARD.encode(format!("{user}:{pass}"));
        headers.insert(
            AUTHORIZATION,
            format!("Basic {encoded}").parse().expect("a valid header"),
        );
        headers
    }

    #[test]
    fn the_right_token_is_accepted_whatever_the_username() {
        assert!(authorized(&basic("admin", "s3cret"), "s3cret"));
        assert!(authorized(&basic("", "s3cret"), "s3cret"));
    }

    #[test]
    fn a_wrong_or_missing_token_is_rejected() {
        assert!(!authorized(&basic("admin", "wrong"), "s3cret"));
        assert!(!authorized(&HeaderMap::new(), "s3cret"));
    }

    #[test]
    fn a_prefix_of_the_token_is_not_enough() {
        // The digest comparison must not accept a shorter password that happens
        // to be a prefix: both sides are hashed to a fixed width first.
        assert!(!authorized(&basic("admin", "s3cre"), "s3cret"));
    }

    #[test]
    fn the_read_rate_is_the_share_of_ended_notes_read() {
        assert_eq!(read_rate(0, 0), "—");
        assert_eq!(read_rate(0, 5), "0%");
        assert_eq!(read_rate(1, 1), "50%");
        assert_eq!(read_rate(3, 1), "75%");
    }

    #[test]
    fn the_admin_policy_pins_the_style_and_allows_no_script() {
        let policy = csp_header();
        let policy = policy.to_str().expect("ascii");

        assert!(policy.contains("style-src 'sha256-"), "{policy}");
        assert!(!policy.contains("'unsafe-inline'"), "{policy}");
        assert!(!policy.contains("script-src"), "{policy}");
        assert!(policy.starts_with("default-src 'none'"), "{policy}");
    }

    #[test]
    fn the_pinned_style_hash_is_the_served_style() {
        use sha2::{Digest as _, Sha256};
        let expected = STANDARD.encode(Sha256::digest(STYLE.as_bytes()));
        assert!(
            csp_header()
                .to_str()
                .expect("ascii")
                .contains(&format!("'sha256-{expected}'"))
        );
    }
}
