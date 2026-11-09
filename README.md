<div align="center">

<img src="docs/logo.png" alt="BlindNote" width="104">

<h1>BlindNote</h1>

<strong>One-time-read, end-to-end encrypted, self-destructing notes — that you can actually verify.</strong>

<p>
  <a href="https://blindnote.com"><strong>blindnote.com</strong></a>
  ·
  <a href="SECURITY.md">Security</a>
  ·
  <a href="verify.sh">Verify a release</a>
  ·
  <a href="LICENSE.md">License</a>
</p>

</div>

One-time-read, end-to-end encrypted, self-destructing notes. One Rust binary, a React
client baked into it, one dedicated server.

You write a note, you get a link, you send the link. The first person to open it reads the
note once; then it is gone. The text is encrypted in your browser and decrypted in theirs —
the server never sees it, and cannot.

## Why another one of these

There are already several one-time-secret services. The difference here is not the
encryption — everyone claims that. The difference is that **you can check it.**

Browser-delivered end-to-end encryption has one structural weakness: the server sends the
JavaScript that protects your secret, so a dishonest server could send *different*
JavaScript to one person and read everything. The only real answer is a chain you can
follow yourself:

> published source → a signed release whose signature carries the build workflow's own
> identity (logged in a transparency log) → proof the code your browser runs is built from
> that source, byte for byte → **somebody actually checking.**

So BlindNote publishes its source and signs every release with
[cosign](https://docs.sigstore.dev/) keyless (the signature carries the build workflow's own
identity, not a key we could quietly swap, and is logged in a public transparency log) with
SLSA provenance and an SBOM, and ships [`verify.sh`](./verify.sh) — which checks that
signature and then proves the client your browser actually runs is byte-for-byte the client
built from that release's source. (The server binary is not yet bit-for-bit reproducible; we
verify the code your browser executes, which is what a browser-delivered app turns on.) A
hash nobody checks is theatre; the ask is that you run the check.

The project exists because its 2021 predecessor called itself "end-to-end encrypted" and
**stored every note in the database as plaintext.** It was never public, so the false claim
never reached anyone — but it is the reason this rewrite holds itself to "don't trust us,
verify" instead of "trust us."

## How it works, briefly

- The link is `https://blindnote.com/#<id>/<key>`. The key lives in the URL **fragment**,
  which browsers never send to the server. The server stores only ciphertext it cannot read.
- Reading a note deletes its per-note key — 32 bytes in a small, separate database file.
  The claim is "the key is gone," never "the bytes were physically scrubbed from the SSD,"
  because on an SSD that is not provable.
- Missing, expired, and already-read all return the **identical** response. There is no way
  to ask the server "did this note exist?"
- The note is shown as plain text. Never as HTML, never auto-linkified.
- The whole client is embedded in the server binary; nothing is fetched from a CDN, and a
  strict Content-Security-Policy pins the script hashes.

The deeper reasoning — and the measurements behind each decision — live in
[`docs/devlog.md`](./docs/devlog.md).

## Verifying a release

With `cosign`, `node`, `git` and `curl`, inside a checkout of this repo:

```sh
./verify.sh v0.1.7
```

It checks that the published image was signed by this repository's release workflow at that
exact tag, then builds the client from the source at that tag and confirms the live site
serves those exact bytes — each asset byte-for-byte, with its hash pinned in the page's CSP
and SRI so the browser would refuse anything else. Run it against the release the site is
currently serving.

## What it does *not* protect against

Read [`SECURITY.md`](./SECURITY.md) before you rely on this for anything. The short version:
whoever opens the link first wins (the link *is* the key), a compromised device or a
screenshot defeats everything, and "self-destructing" is about our storage, not the
recipient's screen. We would rather tell you the limits than oversell the guarantee — that
overselling is exactly the failure this project was built to not repeat.

## Source-available, not open source

BlindNote is licensed under [PolyForm Shield 1.0.0](./LICENSE.md): use it for any purpose
except building something that competes with it. Because it restricts a field of use, it is
**not** an OSI open-source licence, and we do not call it "open source" — for a project whose
whole pitch is "we don't lie to you," that word is not available to us. "Source-available"
is the honest phrasing, and it is all the trust chain above needs: verification wants
readable source and a reproducible build, not an OSI licence.

The **name, logo and domain are not licensed** — a copy must rename.

## Contributing

**Bug reports, security reports and rebuild verifications are very welcome. Pull requests are
not accepted.** Under this licence the project may be monetized by its author and a
contributor may not, so taking outside code would leave the project holding patches it is not
clearly licensed to sell. This keeps things honest and costs you nothing: open an issue.

Security issues: see [`SECURITY.md`](./SECURITY.md) or
[`/.well-known/security.txt`](https://blindnote.com/.well-known/security.txt).
