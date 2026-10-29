//! Security headers, applied to every response.
//!
//! These are produced by the application and never by the reverse proxy.
//! Managing a header set in two places is the easiest way to end up with two
//! header sets, and the one that matters most -- the content security policy
//! -- has to be generated from the client bundle at build time, which only the
//! application can do.
//!
//! The content security policy is in its own module, because it is built from
//! the embedded bundle's hashes rather than written out as a constant.
//!
//! `Cache-Control` is deliberately absent. Every response sets its own: API
//! answers are `no-store`, and the embedded client assets will be `immutable`.
//! A blanket value would be wrong for one of them.

use std::sync::LazyLock;

use axum::extract::Request;
use axum::http::header::CONTENT_SECURITY_POLICY;
use axum::http::header::{
    HeaderName, HeaderValue, REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS,
};
use axum::middleware::Next;
use axum::response::Response;

use crate::csp;

/// Every browser feature this application has no use for.
///
/// Written out rather than abbreviated because there is no wildcard that means
/// "and anything added later": a feature absent from this list is a feature
/// that stays enabled when the browser gains it. Revisit when the client
/// changes, not when the list looks long.
const PERMISSIONS_POLICY: &str = "accelerometer=(), ambient-light-sensor=(), autoplay=(), \
     battery=(), bluetooth=(), camera=(), display-capture=(), encrypted-media=(), \
     fullscreen=(), gamepad=(), geolocation=(), gyroscope=(), hid=(), idle-detection=(), \
     local-fonts=(), magnetometer=(), microphone=(), midi=(), payment=(), \
     picture-in-picture=(), publickey-credentials-create=(), publickey-credentials-get=(), \
     screen-wake-lock=(), serial=(), speaker-selection=(), storage-access=(), usb=(), \
     web-share=(), window-management=(), xr-spatial-tracking=()";

/// The header set, built once.
static SECURITY_HEADERS: LazyLock<[(HeaderName, HeaderValue); 7]> = LazyLock::new(|| {
    [
        // Two years, subdomains included, and submitted to the preload list
        // so that even a first visit never happens over plain HTTP.
        (
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ),
        // The key lives in the URL fragment, which is never sent in a request
        // -- but a referrer would still leak that this service was used, and
        // to whom.
        (REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
        (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
        (
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_str(PERMISSIONS_POLICY).expect("a valid header value"),
        ),
        // Cross-origin isolation. Nothing here needs to talk to another
        // window, and a page that cannot be reached by one cannot be read by
        // one either.
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderValue::from_static("same-origin"),
        ),
        (
            HeaderName::from_static("cross-origin-embedder-policy"),
            HeaderValue::from_static("require-corp"),
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ),
    ]
});

/// Adds the security headers to a response on its way out.
///
/// Existing values are overwritten rather than appended to: a handler has no
/// business weakening the policy, and `insert` makes that structural.
pub(crate) async fn apply(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    for (name, value) in SECURITY_HEADERS.iter() {
        headers.insert(name, value.clone());
    }
    headers.insert(CONTENT_SECURITY_POLICY, csp::header());

    response
}

#[cfg(test)]
mod tests {
    use super::PERMISSIONS_POLICY;
    use axum::http::HeaderValue;

    #[test]
    fn the_permissions_policy_is_a_valid_header_value() {
        assert!(HeaderValue::from_str(PERMISSIONS_POLICY).is_ok());
    }

    /// A line continuation in a Rust string literal eats the following
    /// indentation, but only after a backslash. One missing backslash and the
    /// header gains a run of spaces, which is legal and ugly, or a newline,
    /// which is not legal at all.
    #[test]
    fn the_permissions_policy_has_no_newline_or_double_space() {
        assert!(!PERMISSIONS_POLICY.contains('\n'));
        assert!(!PERMISSIONS_POLICY.contains("  "));
    }
}
