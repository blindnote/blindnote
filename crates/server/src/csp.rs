//! The content security policy.
//!
//! This is the exfiltration defence, and it is the third item in the project's
//! trust hierarchy for a reason: if injected code cannot reach the network, it
//! cannot steal a note. The browser's own encryption key is in memory on the
//! page, so "cannot phone home" is the difference between a compromised script
//! being a nuisance and being the whole secret.
//!
//! Two things make it strict rather than decorative:
//!
//! * **`default-src 'none'`.** Every directive the policy does not name falls
//!   back to this, so a browser feature nobody thought about is denied rather
//!   than permitted. The alternative -- listing what to deny -- is a list that
//!   is wrong every time a browser ships something.
//! * **The script hash is the build's, not a human's.** It comes from
//!   `build.rs`, which computed it from the bytes it embedded and stamped the
//!   same value into the page's `integrity` attribute. The stylesheet is pinned
//!   by that same `integrity` attribute alone -- browsers do not match an
//!   external stylesheet against a `style-src` hash the way they do a script, so
//!   `style-src` is `'self'` and SRI carries the byte guarantee.

use std::fmt::Write as _;
use std::sync::LazyLock;

use axum::http::HeaderValue;

use crate::assets::{INLINE_SCRIPT_HASHES, SCRIPT_HASHES};

/// The policy, built once from the embedded bundle's hashes.
static POLICY: LazyLock<String> = LazyLock::new(build);

/// The header value, built once.
static HEADER: LazyLock<HeaderValue> = LazyLock::new(|| {
    HeaderValue::from_str(&POLICY)
        .unwrap_or_else(|_| HeaderValue::from_static("default-src 'none'"))
});

/// The `Content-Security-Policy` value to send.
pub(crate) fn header() -> HeaderValue {
    HEADER.clone()
}

/// Assembles the policy.
fn build() -> String {
    let mut policy = String::new();

    // Deny by default, then name what the application actually needs.
    directive(&mut policy, "default-src", "'none'");

    // Scripts are pinned by hash. An external script matches a script-src hash
    // when the element repeats it in `integrity`; an inline script (the theme
    // bootstrap) matches by the hash of its text directly. Both lists are the
    // build's, from the bytes it embedded. With nothing embedded there is
    // nothing to allow, and 'none' says so.
    let mut scripts: Vec<&str> =
        Vec::with_capacity(SCRIPT_HASHES.len() + INLINE_SCRIPT_HASHES.len());
    scripts.extend_from_slice(SCRIPT_HASHES);
    scripts.extend_from_slice(INLINE_SCRIPT_HASHES);
    directive(&mut policy, "script-src", &sources(&scripts));

    // Styles cannot be pinned the same way: browsers do not match an external
    // stylesheet against a style-src hash (only inline styles), so a hash here
    // would silently block our own stylesheet. The byte pinning lives in the
    // link's `integrity` attribute instead -- which browsers DO enforce for a
    // stylesheet -- and style-src is 'self' to permit the same-origin sheet.
    // There are no inline styles in the bundle.
    directive(&mut policy, "style-src", "'self'");

    // Self-hosted woff2, from the same connection as everything else. No
    // Google Fonts, no third party, ever -- which is why this is 'self' and
    // not a host list.
    directive(&mut policy, "font-src", "'self'");

    // Favicons, the apple-touch icon and the manifest's icons are all served
    // from this binary. 'self' lets the browser load them and admits nothing
    // wider: a third-party image is a beacon that leaks the page was opened,
    // and 'self' forbids exactly that -- while default-src 'none' would refuse
    // our own icons along with everyone else's.
    directive(&mut policy, "img-src", "'self'");

    // The three API calls, and nothing else. This is the directive that makes
    // injected code unable to send a note anywhere.
    directive(&mut policy, "connect-src", "'self'");

    // The web app manifest, same-origin. Without this it falls back to
    // default-src 'none' and the browser drops it; with it the browser reads
    // the icons and theme colour the manifest declares.
    directive(&mut policy, "manifest-src", "'self'");

    // No plugins, no embedding, no base-tag rewriting of relative urls, and no
    // form may navigate anywhere: every request this application makes goes
    // through `fetch`.
    directive(&mut policy, "object-src", "'none'");
    directive(&mut policy, "frame-ancestors", "'none'");
    directive(&mut policy, "base-uri", "'none'");
    directive(&mut policy, "form-action", "'none'");

    // Note content is rendered as text, never as markup. This is that rule
    // enforced by the browser rather than by review: an assignment to
    // `innerHTML` throws instead of parsing. Browsers without it ignore the
    // directive, so it costs nothing.
    policy.push_str("require-trusted-types-for 'script'");

    policy
}

/// Renders the hashes as a source list, or `'none'` when there are none.
fn sources(hashes: &[&str]) -> String {
    if hashes.is_empty() {
        return "'none'".to_owned();
    }

    let mut list = String::new();
    for hash in hashes {
        if !list.is_empty() {
            list.push(' ');
        }
        let _ = write!(list, "'sha256-{hash}'");
    }
    list
}

/// Appends one directive and its separator.
fn directive(policy: &mut String, name: &str, value: &str) {
    let _ = write!(policy, "{name} {value}; ");
}

#[cfg(test)]
mod tests {
    use super::{build, header, sources};
    use crate::assets::{ASSETS, INLINE_SCRIPT_HASHES, SCRIPT_HASHES};

    #[test]
    fn the_policy_denies_everything_it_does_not_name() {
        assert!(build().starts_with("default-src 'none';"));
    }

    #[test]
    fn the_policy_is_a_valid_header_value() {
        assert!(!header().is_empty());
        assert_eq!(header().to_str().expect("ascii"), build());
    }

    /// Nothing may be loaded from, or sent to, anywhere but this origin.
    #[test]
    fn the_policy_allows_no_third_party() {
        let policy = build();

        for forbidden in [
            "http:",
            "https:",
            "*",
            "'unsafe-inline'",
            "'unsafe-eval'",
            "data:",
        ] {
            assert!(
                !policy.contains(forbidden),
                "the policy contains {forbidden}: {policy}"
            );
        }
    }

    #[test]
    fn an_empty_bundle_pins_nothing_rather_than_allowing_anything() {
        assert_eq!(sources(&[]), "'none'");
    }

    /// The pinned script hash is the hash of the script that is served.
    ///
    /// The two come from the same bytes in `build.rs`, so this cannot drift --
    /// which is exactly why it is worth asserting: if it ever does, the page
    /// stops working and nothing else would say why.
    #[test]
    fn every_pinned_script_hash_belongs_to_a_served_script() {
        if ASSETS.is_empty() {
            return;
        }

        for hash in SCRIPT_HASHES {
            let quoted = format!("\"{hash}\"");
            assert!(
                ASSETS.iter().any(|asset| asset.etag == quoted),
                "the policy pins {hash}, which is not any served file's hash"
            );
        }
    }

    /// The page's inline bootstrap script is pinned by hash, never allowed by
    /// `'unsafe-inline'`. When an interface is embedded there is exactly one
    /// inline script, and the policy carries its hash.
    #[test]
    fn the_inline_script_is_pinned_by_hash() {
        if ASSETS.is_empty() {
            return;
        }

        assert!(
            !INLINE_SCRIPT_HASHES.is_empty(),
            "the page has no inline script hash, so the theme bootstrap is unpinned"
        );
        let policy = build();
        for hash in INLINE_SCRIPT_HASHES {
            assert!(
                policy.contains(&format!("'sha256-{hash}'")),
                "the policy does not pin the inline script hash {hash}"
            );
        }
    }
}
