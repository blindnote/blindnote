//! Serving the embedded client.
//!
//! Everything is decided at build time: the bytes, the Brotli copy, the media
//! type, the caching rule and the ETag all come out of `build.rs` as a table.
//! Serving a file is a lookup and a header set, with no disk access, no
//! compression and no hashing on the request path. The whole interface is in
//! the binary, which is the point: nothing is fetched from a CDN at runtime,
//! and the bytes a visitor runs are the bytes that were published.

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::extract::{ConnectInfo, Path as UrlPath, State};
use axum::http::StatusCode;
use axum::http::header::{
    ACCEPT_ENCODING, CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE, ETAG, HeaderMap, HeaderName,
    HeaderValue, IF_NONE_MATCH, VARY,
};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use blindnote_store::{Store, UnixTime};

use crate::rate_limit::resolve_client_ip;

/// How long a file may be kept, in the terms the build script decides.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CachePolicy {
    /// Content-hashed filename: the bytes behind this name never change.
    Immutable,
    /// Stable in practice, but the name carries no hash, so revalidate.
    Revalidate,
    /// Never stored.
    Never,
}

impl CachePolicy {
    /// The `Cache-Control` value.
    const fn header(self) -> &'static str {
        match self {
            Self::Immutable => "public, max-age=31536000, immutable",
            Self::Revalidate => "public, max-age=86400, must-revalidate",
            Self::Never => "no-store",
        }
    }
}

/// One embedded file.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Asset {
    /// Path it is served at, with no leading slash.
    pub(crate) path: &'static str,
    /// Media type, from a fixed list rather than a guess.
    pub(crate) content_type: &'static str,
    /// How long it may be kept.
    pub(crate) cache: CachePolicy,
    /// Quoted base64 SHA-256 of the uncompressed bytes.
    pub(crate) etag: &'static str,
    /// The bytes, uncompressed.
    pub(crate) identity: &'static [u8],
    /// The Brotli copy, when compressing it was worth the space.
    pub(crate) brotli: Option<&'static [u8]>,
}

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Builds the routes that serve the interface.
///
/// Two routes rather than a wildcard mounted at the root, so that `/api`
/// cannot be shadowed by a file and an unknown path is a plain 404 instead of
/// the page. There is no single-page fallback because the application has no
/// paths: everything it needs is in the fragment.
pub(crate) fn router(store: Store, trusted_proxy: Option<IpAddr>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/{*path}", get(file))
        .with_state(PageViews {
            store,
            trusted_proxy,
        })
}

/// What the entry-point route needs to count a page view.
#[derive(Clone)]
struct PageViews {
    store: Store,
    trusted_proxy: Option<IpAddr>,
}

impl PageViews {
    /// Counts one view of the interface.
    ///
    /// The client address is resolved the same way the rate limiter resolves
    /// it, used in memory only, and folded into an aggregate sketch -- it is
    /// never stored and cannot be recovered from the sketch. See
    /// [`Store::record_view`].
    fn record(&self, peer: SocketAddr, headers: &HeaderMap) {
        let address = resolve_client_ip(peer.ip(), headers, self.trusted_proxy);
        self.store.record_view(UnixTime::now(), &ip_bytes(address));
    }
}

/// An address's canonical bytes: four for IPv4, sixteen for IPv6.
fn ip_bytes(address: IpAddr) -> Vec<u8> {
    match address {
        IpAddr::V4(v4) => v4.octets().to_vec(),
        IpAddr::V6(v6) => v6.octets().to_vec(),
    }
}

/// Serves `index.html`, counting the view first.
///
/// The one route with state; every other asset is static. `file` shares the
/// router's state type but does not extract it.
async fn index(
    State(views): State<PageViews>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    views.record(peer, &headers);
    serve("index.html", &headers)
}

/// Serves any other embedded file.
async fn file(UrlPath(path): UrlPath<String>, headers: HeaderMap) -> Response {
    serve(&path, &headers)
}

/// Looks a path up and answers.
fn serve(path: &str, headers: &HeaderMap) -> Response {
    let Some(asset) = ASSETS.iter().find(|asset| asset.path == path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // The ETag is the only thing compared, and it is compared as sent. A weak
    // match is not accepted: these bytes never change under a name, so a
    // mismatch means a different file rather than a different encoding of one.
    if headers
        .get_all(IF_NONE_MATCH)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|candidate| candidate.trim() == asset.etag)
    {
        return (StatusCode::NOT_MODIFIED, cache_headers(asset)).into_response();
    }

    let accepts_brotli = headers
        .get(ACCEPT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|encoding| {
                encoding
                    .split(';')
                    .next()
                    .is_some_and(|name| name.trim() == "br")
            })
        });

    let mut response = match (accepts_brotli, asset.brotli) {
        (true, Some(compressed)) => {
            let mut response = compressed.into_response();
            response
                .headers_mut()
                .insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
            response
        }
        _ => asset.identity.into_response(),
    };

    let header_map = response.headers_mut();
    for (name, value) in cache_headers(asset) {
        header_map.insert(name, value);
    }
    header_map.insert(CONTENT_TYPE, header_value(asset.content_type));

    response
}

/// A header value from a build-time string, without a panic if it is ever bad.
///
/// Every value that reaches this comes from a fixed list in `build.rs`, so the
/// fallback is unreachable today. It exists because the alternative,
/// `HeaderValue::from_static`, panics -- and a panic in a static file handler
/// would be a strange way to find out that somebody added an extension with a
/// typo in it.
fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))
}

/// The headers an answer carries whether or not it has a body.
///
/// `Vary` is here rather than only on a compressed answer, because a cache
/// that stored the uncompressed copy without it would then hand that copy to a
/// client that asked for Brotli -- correct, but it would also do the reverse.
fn cache_headers(asset: &Asset) -> [(HeaderName, HeaderValue); 3] {
    [
        (CACHE_CONTROL, header_value(asset.cache.header())),
        (ETAG, header_value(asset.etag)),
        (VARY, HeaderValue::from_static("accept-encoding")),
    ]
}

/// Whether anything was embedded at all.
///
/// An empty table means `client/dist` was missing when this was built. The
/// build script warns about it; the server says so again at startup, because a
/// binary that serves an API and no interface looks healthy from every angle
/// except a browser's.
pub(crate) fn is_empty() -> bool {
    ASSETS.is_empty()
}

#[cfg(test)]
mod tests {
    use super::{ASSETS, SCRIPT_HASHES, STYLE_HASHES};

    /// Every embedded file has to be reachable and typed.
    ///
    /// Skipped when nothing is embedded, which is the case in a checkout where
    /// `client/dist` has not been built. The alternative -- failing -- would
    /// make `cargo test` depend on `npm`.
    #[test]
    fn every_asset_is_typed_and_tagged() {
        for asset in ASSETS {
            assert!(
                !asset.path.starts_with('/'),
                "{}: served paths are relative",
                asset.path
            );
            assert!(!asset.identity.is_empty(), "{}: no bytes", asset.path);
            assert!(
                asset.etag.starts_with('"') && asset.etag.ends_with('"'),
                "{}: an ETag is a quoted string",
                asset.path
            );
            assert_ne!(
                asset.content_type, "application/octet-stream",
                "{}: unknown extension, so it would be served untyped",
                asset.path
            );
        }
    }

    /// A Brotli copy is only kept when it is smaller.
    #[test]
    fn no_brotli_copy_is_larger_than_the_original() {
        for asset in ASSETS {
            if let Some(compressed) = asset.brotli {
                assert!(
                    compressed.len() < asset.identity.len(),
                    "{}: the compressed copy is not smaller",
                    asset.path
                );
            }
        }
    }

    /// When there is an interface, there is a page and a script to pin.
    #[test]
    fn an_embedded_interface_has_an_index_and_a_script() {
        if ASSETS.is_empty() {
            return;
        }

        assert!(
            ASSETS.iter().any(|asset| asset.path == "index.html"),
            "the interface has no entry point"
        );
        assert!(
            !SCRIPT_HASHES.is_empty(),
            "no script hashes, so the policy in step 7 would have nothing to pin"
        );
        assert!(!STYLE_HASHES.is_empty(), "no style hashes");
    }
}
