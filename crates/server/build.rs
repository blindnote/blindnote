//! Bakes the client build into the binary.
//!
//! The browser is handed static files, and a static file server has three
//! questions to answer per request: what are the bytes, what is the type, and
//! has the client already got it. All three are answered here, at build time,
//! so that serving a file at runtime is a table lookup and nothing else -- no
//! disk, no compression, no hashing on the request path.
//!
//! This replaces `rust-embed`. That crate embeds the bytes and stops there,
//! which would leave Brotli compression, content types, ETags and the content
//! security policy's script hashes to be arranged elsewhere. Doing it in one
//! build script is both less code and more reproducible: the generated table
//! carries no timestamps and no paths from this machine.
//!
//! If `client/dist` is missing the table comes out empty, with a warning here
//! and an error logged at startup. That keeps `cargo test` working without a
//! `npm run build` first, while making the gap loud rather than quiet.

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

/// Brotli quality. 11 is the maximum, and this runs once per build.
const BROTLI_QUALITY: u32 = 11;

/// Brotli window size, in bits. 22 is the maximum any browser accepts.
const BROTLI_WINDOW: u32 = 22;

/// The page, which is the only file this script rewrites.
const INDEX: &str = "index.html";

/// Keep the compressed copy only if it comes out below this percentage.
///
/// A woff2 font is already Brotli-compressed internally, so compressing it
/// again gains a fraction of a percent for a doubled footprint. The threshold
/// decides per file rather than per extension, which is one fewer list to keep
/// correct.
const WORTH_COMPRESSING_PERCENT: usize = 95;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets this"));
    let dist = manifest.join("../../client/dist");
    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets this"));

    println!("cargo::rerun-if-changed={}", dist.display());
    println!("cargo::rerun-if-changed=build.rs");

    let sources = collect(&dist);
    if sources.is_empty() {
        println!(
            "cargo::warning=client/dist is missing or empty, so no interface is embedded. \
             Run `npm ci && npm run build` in client/ before building a release."
        );
    }

    let assets_dir = out.join("assets");
    fs::create_dir_all(&assets_dir).expect("the output directory is writable");

    // Read everything first, because the page cannot be finished until the
    // files it references have been hashed.
    let mut contents: BTreeMap<String, Vec<u8>> = sources
        .iter()
        .map(|(served, source)| (served.clone(), fs::read(source).expect("a readable file")))
        .collect();

    let mut hashes: BTreeMap<String, String> = contents
        .iter()
        .map(|(served, bytes)| (served.clone(), base64_standard(&Sha256::digest(bytes))))
        .collect();

    // A content security policy hash only matches an *external* script when
    // the element repeats the hash in an `integrity` attribute. So the page is
    // stamped here, and then re-hashed: its own bytes just changed.
    if let Some(page) = contents.get(INDEX).cloned() {
        let stamped = stamp_integrity(&String::from_utf8_lossy(&page), &hashes);
        hashes.insert(
            INDEX.to_owned(),
            base64_standard(&Sha256::digest(stamped.as_bytes())),
        );
        contents.insert(INDEX.to_owned(), stamped.into_bytes());
    }

    let mut table = String::new();
    let mut script_hashes = Vec::new();
    let mut style_hashes = Vec::new();

    for (served, identity) in &contents {
        let sha256 = &hashes[served];

        // Collected here because this is the only place that has the bytes,
        // and a hash computed anywhere else could disagree with what is served.
        match extension(served) {
            "js" => script_hashes.push(sha256.clone()),
            "css" => style_hashes.push(sha256.clone()),
            _ => {}
        }

        let stored = store(&assets_dir, served, identity);
        let compressed = compress(identity);
        let brotli = match &compressed {
            Some(bytes) => {
                let path = store(&assets_dir, &format!("{served}.br"), bytes);
                format!("Some(include_bytes!(r\"{path}\"))")
            }
            None => "None".to_owned(),
        };

        writeln!(
            table,
            "    Asset {{\n        \
             path: {served:?},\n        \
             content_type: {:?},\n        \
             cache: {},\n        \
             etag: {:?},\n        \
             identity: include_bytes!(r\"{stored}\"),\n        \
             brotli: {brotli},\n    }},",
            content_type(served),
            cache_policy(served),
            format!("\"{sha256}\""),
        )
        .expect("writing to a String cannot fail");
    }

    // Inline scripts are pinned by the hash of their text, not by `integrity`:
    // a browser matches an inline script against a script-src hash directly. The
    // page's theme bootstrap (which must run before first paint, too early to
    // wait for the bundle) is the one inline script, hashed here from the bytes
    // as served so the policy can allow exactly it and nothing else.
    let inline_hashes = contents
        .get(INDEX)
        .map(|page| inline_script_hashes(&String::from_utf8_lossy(page)))
        .unwrap_or_default();

    let generated = format!(
        "// Generated by build.rs. Do not edit.\n\
         pub(crate) static ASSETS: &[Asset] = &[\n{table}];\n\n\
         pub(crate) static SCRIPT_HASHES: &[&str] = &[{}];\n\n\
         pub(crate) static INLINE_SCRIPT_HASHES: &[&str] = &[{}];\n\n\
         #[allow(dead_code, reason = \"used only by tests; style-src is 'self', pinned by SRI\")]\n\
         pub(crate) static STYLE_HASHES: &[&str] = &[{}];\n",
        quoted(&script_hashes),
        quoted(&inline_hashes),
        quoted(&style_hashes),
    );

    fs::write(out.join("assets.rs"), generated).expect("the output directory is writable");
}

/// Adds an `integrity` attribute to every tag that references a hashed asset.
///
/// A content security policy hash matches an inline script by its text, but an
/// *external* one only when the element repeats the same hash in `integrity`.
/// So pinning the bundle takes both halves, and both have to come from the
/// same bytes -- which is why this happens in the build rather than by hand.
///
/// The transformation is deliberately narrow: it walks tags, and for a
/// `<script>` or `<link>` that names a file under `assets/` and has no
/// `integrity` yet, it inserts one. Anything else is copied through untouched,
/// including the font preloads.
fn stamp_integrity(html: &str, hashes: &BTreeMap<String, String>) -> String {
    let mut stamped = String::with_capacity(html.len() + 256);
    let mut rest = html;

    while let Some(open) = rest.find('<') {
        stamped.push_str(&rest[..open]);
        rest = &rest[open..];

        let Some(close) = rest.find('>') else {
            // An unterminated tag is not ours to repair.
            break;
        };

        stamped.push_str(&stamp_tag(&rest[..=close], hashes));
        rest = &rest[close + 1..];
    }

    stamped.push_str(rest);
    stamped
}

/// Stamps one tag, or returns it unchanged.
fn stamp_tag(tag: &str, hashes: &BTreeMap<String, String>) -> String {
    let interesting = tag.starts_with("<script") || tag.starts_with("<link");
    if !interesting || tag.contains("integrity=") {
        return tag.to_owned();
    }

    let Some(referenced) = hashes
        .keys()
        .filter(|path| path.starts_with("assets/"))
        .find(|path| tag.contains(&format!("/{path}\"")))
    else {
        return tag.to_owned();
    };

    // Trim the closing bracket and any self-closing slash, then put them back.
    let inner = tag
        .trim_end_matches('>')
        .trim_end()
        .trim_end_matches('/')
        .trim_end();

    format!("{inner} integrity=\"sha256-{}\">", hashes[referenced])
}

/// The content-security-policy hash of every inline `<script>` in the page.
///
/// A browser hashes an inline script's text to match it against a `script-src`
/// hash -- there is no `integrity` attribute as there is for an external one.
/// So this walks the page, and for every `<script>` with no `src` it hashes the
/// text between the tags exactly as served. An external script (one with a
/// `src`) is skipped: it is pinned by the `integrity` stamped above.
fn inline_script_hashes(html: &str) -> Vec<String> {
    const OPEN: &str = "<script";
    const CLOSE: &str = "</script>";

    let mut hashes = Vec::new();
    let mut rest = html;

    while let Some(start) = rest.find(OPEN) {
        rest = &rest[start..];
        let Some(open_end) = rest.find('>') else {
            break;
        };
        let open_tag = &rest[..=open_end];
        let body = &rest[open_end + 1..];
        let Some(close) = body.find(CLOSE) else {
            break;
        };

        // A script with a source is external, and pinned by its `integrity`.
        if !open_tag.contains("src=") {
            hashes.push(base64_standard(&Sha256::digest(&body.as_bytes()[..close])));
        }
        rest = &body[close + CLOSE.len()..];
    }

    hashes
}

/// Every file under `dist`, keyed by the path it is served at.
///
/// A `BTreeMap` so the generated table is in a stable order whatever the
/// filesystem feels like returning -- the point of a reproducible build is
/// that two runs produce the same bytes.
fn collect(dist: &Path) -> BTreeMap<String, PathBuf> {
    let mut files = BTreeMap::new();
    walk(dist, dist, &mut files);
    files
}

/// Recurses into `dir`, recording paths relative to `root`.
fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, files);
            continue;
        }

        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let served = relative.to_string_lossy().replace('\\', "/");
        files.insert(served, path);
    }
}

/// Writes bytes next to the generated table and returns the absolute path.
fn store(dir: &Path, name: &str, bytes: &[u8]) -> String {
    let flat = name.replace('/', "__");
    let path = dir.join(flat);
    fs::write(&path, bytes).expect("the output directory is writable");
    path.to_string_lossy().into_owned()
}

/// Compresses, or returns `None` when compressing is not worth it.
fn compress(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut input = bytes;
    brotli::BrotliCompress(
        &mut input,
        &mut compressed,
        &brotli::enc::BrotliEncoderParams {
            quality: BROTLI_QUALITY.cast_signed(),
            lgwin: BROTLI_WINDOW.cast_signed(),
            ..Default::default()
        },
    )
    .expect("compressing a byte slice in memory cannot fail");

    // Integer comparison rather than a ratio: there is no precision to lose
    // and nothing to round the wrong way.
    let worthwhile = compressed.len() * 100 < bytes.len() * WORTH_COMPRESSING_PERCENT;
    worthwhile.then_some(compressed)
}

/// A served path's extension, lowercased, or the empty string.
fn extension(path: &str) -> &str {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
}

/// The media type for a served path.
///
/// A short list rather than a mime database: this directory holds exactly what
/// Vite produces plus the fonts, and a dependency for seven extensions would
/// be one more thing in the reproducible build.
fn content_type(path: &str) -> &'static str {
    match extension(path) {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "woff2" => "font/woff2",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webp" => "image/webp",
        "ico" => "image/vnd.microsoft.icon",
        "json" => "application/json",
        "webmanifest" => "application/manifest+json",
        "txt" => "text/plain; charset=utf-8",
        // Not guessed. An unknown type served as something plausible is how a
        // file ends up being interpreted as a script.
        _ => "application/octet-stream",
    }
}

/// The caching rule for a served path, as a `CachePolicy` expression.
fn cache_policy(path: &str) -> &'static str {
    if path.starts_with("assets/") {
        // Vite puts a content hash in these filenames, so the bytes behind a
        // name never change and a year is as safe as a minute.
        return "CachePolicy::Immutable";
    }
    if path == "index.html" {
        // Never cached. It names the hashed asset files, and the content
        // security policy pins their hashes -- a stale copy would reference
        // scripts the current policy does not allow. It is 276 bytes
        // compressed, so re-fetching it costs nothing and guarantees that a
        // visitor always runs the page that was published.
        return "CachePolicy::Never";
    }
    // Fonts and licences: stable in practice, but the names carry no hash, so
    // they are revalidated rather than trusted for a year.
    "CachePolicy::Revalidate"
}

/// Standard base64, which is what a content security policy hash and an ETag
/// both want.
fn base64_standard(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);

        encoded.push(ALPHABET[(n >> 18) as usize & 63] as char);
        encoded.push(ALPHABET[(n >> 12) as usize & 63] as char);
        encoded.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    encoded
}

/// Renders a list of strings as Rust source.
fn quoted(values: &[String]) -> String {
    let mut rendered = String::new();
    for value in values {
        write!(rendered, "{value:?}, ").expect("writing to a String cannot fail");
    }
    rendered
}
