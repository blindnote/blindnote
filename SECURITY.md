# Security

## Reporting a vulnerability

Email **ye@deligoz.me** — see [`/.well-known/security.txt`](https://blindnote.com/.well-known/security.txt).
Please report privately and give us a reasonable window to fix before any public disclosure.
Rebuild-verification findings (a published image that does not match its source) are security
reports too, and the most valuable kind this project can get.

What helps: the version or image digest, what you observed, and how to reproduce it. You do
not need a working exploit; a clear description of the class of problem is enough.

## What BlindNote protects

- The plaintext and the encryption key never reach the server, in any code path. The key
  travels in the URL fragment, which is never sent in an HTTP request.
- The server stores only ciphertext it cannot read. Reading a note destroys its per-note key.
- The code your browser runs is verifiable against the published source — see
  [`README.md`](./README.md) and [`verify.sh`](./verify.sh). This is the guarantee that makes
  a dishonest operator *detectable*.
- Missing, expired, and already-read notes are indistinguishable from the outside. Note ids
  and request bodies are never logged — there is no access log at all. The only things the
  server records are **anonymous aggregate counters**: totals and per-day counts of page views
  and note activity, plus an approximate daily distinct-visitor estimate from a salted,
  daily-rotating sketch. A visitor's IP address is used **in memory only** — to rate-limit and
  to update that sketch — and is never written to disk; the sketch cannot be reversed to an
  address.

## What BlindNote does **not** protect against

Write your expectations against this list, not against a stronger claim. Over-promising here
is the same failure as calling something encrypted when it is not — just more sophisticated.

1. **Whoever opens the link first wins.** The link *is* the key. If it leaks, the leaker
   reads the note and the intended recipient sees "not found." We cannot tell those two cases
   apart and do not pretend to. The flip side is a feature: a note that shows up already burned
   to the person who was supposed to read it is **evidence that someone else got there first.**
2. **A compromised endpoint.** A malicious browser extension, a keylogger, a compromised
   device, or a screenshot defeats everything. We assume both ends are honest.
3. **The recipient keeping the plaintext** after reading it. "Self-destructing" is about our
   storage, not their screen or clipboard.
4. **Traffic analysis.** Someone watching the network learns that a note was created and later
   read, and roughly when.
5. **Physical byte-level erasure on an SSD.** Burning a note deletes its 32-byte key; the
   guarantee is cryptographic ("the key is gone"), not forensic ("the bytes were scrubbed").
6. **A hostile operator — including us.** Code integrity (the verifiable-build chain) is what
   makes a hostile operator *detectable*; it does not make one impossible. This is the reason
   the whole verification story exists.

## Scope

In scope: the server, the client, the build and release pipeline, and anything that would let
a note's plaintext or key reach the server or a third party. Out of scope: the limitations
listed above, and denial-of-service by raw volume (rate limits exist; exhausting them is not a
vulnerability report).
