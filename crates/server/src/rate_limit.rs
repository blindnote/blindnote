//! Per-address rate limiting, and the header that makes it work or not work.
//!
//! The application sits behind a reverse proxy, so every request arrives from
//! the proxy's address. Left alone, a per-address limiter buckets the entire
//! internet into one counter and does nothing at all. The fix is to read
//! `X-Forwarded-For` -- and the fix has a trap in it, because that header is
//! something any client can send.
//!
//! RFC 7239's `Forwarded` header is the standardised replacement and is
//! deliberately not read here. The proxy in front of us sends the older
//! header, and accepting two sources for one fact means two parsers and two
//! chances to trust the wrong one. If the proxy changes, change this module
//! rather than adding a fallback.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::http::header::HeaderName;
use tower_governor::errors::GovernorError;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;

use crate::Config;

/// A limiter configuration, with the generics spelled out.
///
/// `tower_governor` leaves the middleware parameter open and gives it no
/// default, so it has to be named here. The no-op middleware is the one that
/// does not add `x-ratelimit-*` headers; a throttled client learns it was
/// throttled from the status code and nothing more.
type Limiter = GovernorConfig<ClientAddress, governor::middleware::NoOpMiddleware>;

/// How often the limiter's per-address state is pruned.
///
/// Without this the map grows by one entry per distinct source address and
/// never shrinks, which turns a limiter meant to bound memory into a way to
/// exhaust it.
const RETAIN_INTERVAL: Duration = Duration::from_secs(60);

/// The forwarding header this application reads. Not a standard header, so
/// `http` has no constant for it.
const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");

/// Attributes a request to a client address.
#[derive(Clone, Debug)]
pub(crate) struct ClientAddress {
    /// The proxy whose forwarding header may be believed, if any.
    trusted_proxy: Option<IpAddr>,
}

impl ClientAddress {
    /// Builds an extractor from the configured trusted proxy.
    pub(crate) const fn new(trusted_proxy: Option<IpAddr>) -> Self {
        Self { trusted_proxy }
    }
}

impl KeyExtractor for ClientAddress {
    type Key = IpAddr;

    /// Named for the library's own tracing output.
    fn name(&self) -> &'static str {
        "client address"
    }

    fn extract<T>(&self, request: &Request<T>) -> Result<Self::Key, GovernorError> {
        let peer = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(address)| address.ip())
            .ok_or(GovernorError::UnableToExtractKey)?;

        // Not the configured proxy, or no proxy configured: whatever the
        // client put in a forwarding header is ignored outright. An
        // application that believes a client-supplied `X-Forwarded-For` has a
        // rate limiter that is bypassed by setting one header.
        if self.trusted_proxy != Some(peer) {
            return Ok(peer);
        }

        Ok(last_forwarded_for(request).unwrap_or(peer))
    }
}

/// The **last** address in `X-Forwarded-For`.
///
/// The header reads `client, proxy1, proxy2`, and each hop appends the address
/// it accepted the connection from. Our proxy therefore appends the only entry
/// it observed itself, and that entry is the last one. Everything before it was
/// supplied by whoever was further out -- which, for the first entry, is the
/// client.
///
/// So the obvious reading, "the client is first, take the first", is precisely
/// the bug: a client that sends `X-Forwarded-For: 1.2.3.4` gets attributed to
/// 1.2.3.4, and a new value per request means no limit at all. Taking the last
/// entry is correct whether the proxy appends to a client's header or replaces
/// it.
fn last_forwarded_for<T>(request: &Request<T>) -> Option<IpAddr> {
    request
        .headers()
        .get(&X_FORWARDED_FOR)?
        .to_str()
        .ok()?
        .rsplit(',')
        .next()?
        .trim()
        .parse()
        .ok()
}

/// The two limiters: a burst limit, and an hourly create quota.
pub(crate) struct Limiters {
    /// Applies to every request.
    pub(crate) requests: Arc<Limiter>,
    /// Applies to note creation only.
    pub(crate) creates: Arc<Limiter>,
}

impl Limiters {
    /// Builds both limiters from the configuration.
    ///
    /// # Errors
    ///
    /// Returns the name of the setting that made a limiter impossible. The
    /// builder rejects a zero rate, and a zero rate configured by accident
    /// would otherwise mean either "no limit" or "no service" depending on
    /// which way the library happened to round.
    pub(crate) fn new(config: &Config) -> Result<Self, &'static str> {
        let extractor = ClientAddress::new(config.trusted_proxy);

        let requests = GovernorConfigBuilder::default()
            .per_second(config.requests_per_second)
            .burst_size(config.request_burst)
            .key_extractor(extractor.clone())
            .finish()
            .ok_or("BLINDNOTE_REQUESTS_PER_SECOND / BLINDNOTE_REQUEST_BURST")?;

        // An hourly quota expressed as a rate: one token every
        // 3600/N seconds, with N of them available at once. That lets a user
        // create a handful of notes in a row, as people actually do, while
        // still bounding a day's worth of disk.
        let per_create = Duration::from_secs(3600)
            .checked_div(config.creates_per_hour)
            .ok_or("BLINDNOTE_CREATES_PER_HOUR")?;

        let creates = GovernorConfigBuilder::default()
            .period(per_create)
            .burst_size(config.creates_per_hour)
            .key_extractor(extractor)
            .finish()
            .ok_or("BLINDNOTE_CREATES_PER_HOUR")?;

        Ok(Self {
            requests: Arc::new(requests),
            creates: Arc::new(creates),
        })
    }

    /// Prunes both limiters' per-address state until the process ends.
    pub(crate) fn spawn_pruner(&self) -> tokio::task::JoinHandle<()> {
        let requests = Arc::clone(&self.requests);
        let creates = Arc::clone(&self.creates);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(RETAIN_INTERVAL);

            loop {
                ticker.tick().await;
                requests.limiter().retain_recent();
                creates.limiter().retain_recent();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientAddress, last_forwarded_for};
    use axum::extract::ConnectInfo;
    use axum::http::Request;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tower_governor::key_extractor::KeyExtractor as _;

    const PROXY: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    const CLIENT: IpAddr = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7));
    const SPOOFED: &str = "1.2.3.4";

    fn request(peer: IpAddr, forwarded_for: Option<&str>) -> Request<()> {
        let mut builder = Request::builder();
        if let Some(value) = forwarded_for {
            builder = builder.header("x-forwarded-for", value);
        }

        let mut request = builder.body(()).expect("a valid request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(peer, 51234)));
        request
    }

    #[test]
    fn without_a_trusted_proxy_the_peer_address_is_used() {
        let extractor = ClientAddress::new(None);

        let key = extractor
            .extract(&request(CLIENT, None))
            .expect("the peer address");

        assert_eq!(key, CLIENT);
    }

    /// The bypass this module exists to prevent.
    #[test]
    fn a_forwarding_header_from_an_untrusted_peer_is_ignored() {
        let extractor = ClientAddress::new(Some(PROXY));

        let key = extractor
            .extract(&request(CLIENT, Some(SPOOFED)))
            .expect("the peer address");

        assert_eq!(
            key, CLIENT,
            "a client that is not the proxy must not be able to choose its own bucket"
        );
    }

    #[test]
    fn the_trusted_proxys_forwarding_header_is_believed() {
        let extractor = ClientAddress::new(Some(PROXY));

        let key = extractor
            .extract(&request(PROXY, Some("203.0.113.7")))
            .expect("the forwarded address");

        assert_eq!(key, CLIENT);
    }

    /// The other half of the same bug: a client that sends its own header gets
    /// its value prepended by the proxy, so only the last entry is trustworthy.
    #[test]
    fn a_client_supplied_entry_before_the_proxys_is_ignored() {
        let extractor = ClientAddress::new(Some(PROXY));

        let key = extractor
            .extract(&request(PROXY, Some("1.2.3.4, 203.0.113.7")))
            .expect("the forwarded address");

        assert_eq!(
            key, CLIENT,
            "the first entry came from the client; only the last came from the proxy"
        );
    }

    #[test]
    fn a_trusted_proxy_without_the_header_falls_back_to_its_own_address() {
        let extractor = ClientAddress::new(Some(PROXY));

        let key = extractor
            .extract(&request(PROXY, None))
            .expect("the peer address");

        assert_eq!(key, PROXY);
    }

    #[test]
    fn a_garbled_forwarding_header_falls_back_rather_than_failing() {
        let extractor = ClientAddress::new(Some(PROXY));

        let key = extractor
            .extract(&request(PROXY, Some("not-an-address")))
            .expect("the peer address");

        assert_eq!(key, PROXY);
    }

    #[test]
    fn an_empty_forwarding_header_yields_nothing() {
        assert!(last_forwarded_for(&request(PROXY, Some(""))).is_none());
    }
}
